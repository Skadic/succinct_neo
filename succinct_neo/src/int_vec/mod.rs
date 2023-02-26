pub use self::traits::IntVector;
use std::marker::PhantomData;

mod dynamic;
mod fixed;
mod traits;

#[derive(Debug)]
pub struct Dynamic;
#[derive(Debug)]
pub struct Fixed<const WIDTH: usize>;

#[derive(Debug)]
pub struct IntVec<IntWidth> {
    data: Vec<usize>,
    width: usize,
    capacity: usize,
    size: usize,
    _marker: PhantomData<IntWidth>,
}

impl<T> IntVec<T>
where
    IntVec<T>: IntVector,
{
    #[inline]
    const fn block_width() -> usize {
        std::mem::size_of::<usize>() * 8
    }

    /// Returns the amount of integers would fit into the currently allocated memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Dynamic, IntVec};
    ///
    /// let v = IntVec::<Dynamic>::with_capacity(5, 50);
    ///
    /// // 50 integers of 5 bit each, would fit into 250 bits in total which would make 4 * 64 bit
    /// // blocks, making 256 bits in total. However, 256 bits fit 51 integers of size 5.
    /// assert_eq!(51, v.capacity());
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    fn recalculate_capacity(&mut self) {
        self.capacity = self.data.capacity() * Self::block_width() / self.width;
    }

    /// Grants access to the underlying slice where the bits are saved.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{IntVec, IntVector, Fixed};
    ///
    /// let mut v = IntVec::<Fixed<32>>::new();
    /// v.push(125);
    /// v.push(1231);
    ///
    /// assert_eq!((1231 << 32) | 125, v.raw_data()[0]);
    /// ```
    #[inline]
    pub fn raw_data(&self) -> &[usize] {
        &self.data
    }

    /// Gets an integer of the given bit width from an index.
    ///
    /// This gets the `width` bits at bit index `index width`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to read from.
    /// * `width` - The bit width of integers.
    ///
    /// # Safety
    ///
    /// If `n` is the amount of bits stored in this vector, then for `index` and `width`
    /// `(index + 1) * width < n` must hold.
    ///
    unsafe fn get_unchecked_with_width(&self, index: usize, width: usize) -> usize {
        let index_block = (index * width) / Self::block_width();
        let index_offset = (index * width) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + width >= Self::block_width() {
            let fitting_bits = Self::block_width() - index_offset;
            let remaining_bits = width - fitting_bits;
            let lo = self.data[index_block] >> index_offset;
            let mask = (1 << remaining_bits) - 1;
            let hi = self.data[index_block + 1] & mask;
            return (hi << fitting_bits) | lo;
        }

        let mask = (1 << width) - 1;
        (self.data[index_block] >> index_offset) & mask
    }

    /// Sets an integer of the given bit width at an index.
    ///
    /// This replaces the `width` bits at bit index `index width`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to write to.
    /// * `width` - The bit width of integers.
    ///
    /// # Safety
    ///
    /// If `n` is the amount of bits stored in this vector, then for `index` and `width`
    /// `(index + 1) * width < n` must hold.
    /// In addition, `value` must fit into `width` bits.
    ///
    unsafe fn set_unchecked_with_width(&mut self, index: usize, value: usize, width: usize) {
        let mask = (1 << width) - 1;
        let value = value & mask;
        let index_block = (index * width) / Self::block_width();
        let index_offset = (index * width) % Self::block_width();

        // If we're on the border between blocks
        if index_offset + width >= Self::block_width() {
            let fitting_bits = Self::block_width() - index_offset;
            unsafe {
                let lower_block = self.data.get_unchecked_mut(index_block);
                *lower_block &= !(mask << index_offset);
                *lower_block |= value << index_offset;
                let higher_block = self.data.get_unchecked_mut(index_block + 1);
                *higher_block &= !(mask >> fitting_bits);
                *higher_block |= value >> fitting_bits;
            }
            return;
        }

        self.data[index_block] &= !(mask << index_offset);
        self.data[index_block] |= value << index_offset;
    }

    /// Shrinks the allocated backing storage behind this int vector to fit the amount of saved
    /// integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Dynamic, IntVec};
    ///
    /// let mut v = IntVec::<Dynamic>::with_capacity(5, 200);
    ///
    /// // All these numbers should take 3 bits to save
    /// for i in (0..50) {
    ///     v.push(i % 8)
    /// }
    ///
    /// v.shrink_to_fit();
    ///
    /// // 50 numbers each using 5 bits is 250 bits of storage.
    /// // This fits into 4 * 64 bit blocks (= 256 bits), which in total would fit 51 integers.
    /// assert_eq!(51, v.capacity());
    /// ```
    pub fn shrink_to_fit(&mut self) {
        let required_blocks = num_required_blocks::<usize>(self.size, self.width);
        self.data.truncate(required_blocks);
        self.data.shrink_to_fit();
        self.recalculate_capacity();
    }

    /// An iterator over this vector's saved integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{Dynamic, IntVec};
    ///
    /// let mut v = IntVec::<Dynamic>::new(10);
    /// for i in (0..20).rev() {
    ///     v.push(i);
    /// }
    ///
    /// assert!(Iterator::eq((0..20).rev(), v.iter()))
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<Self> {
        Iter { i: 0, v: self }
    }
}

/// Gets the number of required blocks of the given type to contain the specified number of
/// elements of a given width.
///
/// # Arguments
///
/// * `num_elements` - The number of elements intended to be saved.
/// * `bit_width` - The bit width of each element.
///
/// # Examples
///
/// ```
/// use succinct_neo::int_vec::num_required_blocks;
///
/// // 32 * 10 makes 320 bits, requiring 5 * 64bit blocks.
/// assert_eq!(5, num_required_blocks::<u64>(32, 10))
/// ```
#[inline]
pub fn num_required_blocks<T>(num_elements: usize, bit_width: usize) -> usize {
    (num_elements as f64 * bit_width as f64 / (std::mem::size_of::<T>() as f64 * 8.0)).ceil()
        as usize
}

impl<T> IntoIterator for IntVec<T>
where
    IntVec<T>: IntVector,
{
    type Item = usize;

    type IntoIter = IntoIter<IntVec<T>>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { i: 0, v: self }
    }
}

impl<'a, T> IntoIterator for &'a IntVec<T>
where
    IntVec<T>: IntVector,
{
    type Item = usize;

    type IntoIter = Iter<'a, IntVec<T>>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { i: 0, v: self }
    }
}

pub struct IntoIter<T> {
    i: usize,
    v: T,
}

impl<T> Iterator for IntoIter<T>
where
    T: IntVector,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.v.len() {
            return None;
        }

        let res = self.v.get(self.i);
        self.i += 1;
        Some(res)
    }
}

impl<T: IntVector> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.v.len() - self.i
    }
}

pub struct Iter<'a, T> {
    i: usize,
    v: &'a T,
}

impl<T> Iterator for Iter<'_, T>
where
    T: IntVector,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.v.len() {
            return None;
        }

        let res = self.v.get(self.i);
        self.i += 1;
        Some(res)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T>
where
    T: IntVector,
{
    fn len(&self) -> usize {
        self.v.len() - self.i
    }
}

#[cfg(test)]
mod test {
    use crate::int_vec::num_required_blocks;

    #[test]
    fn packs_required_test() {
        assert_eq!(2, num_required_blocks::<usize>(20, 5));
    }
}
