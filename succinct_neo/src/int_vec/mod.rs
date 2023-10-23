pub use self::traits::IntVector;

mod dynamic;
mod fixed;
mod traits;

pub use dynamic::DynamicIntVec;
pub use fixed::FixedIntVec;

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

impl<const T: usize> IntoIterator for FixedIntVec<T> {
    type Item = usize;

    type IntoIter = IntoIter<FixedIntVec<T>>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { i: 0, v: self }
    }
}

impl IntoIterator for DynamicIntVec {
    type Item = usize;

    type IntoIter = IntoIter<DynamicIntVec>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { i: 0, v: self }
    }
}

impl<'a, const T: usize> IntoIterator for &'a FixedIntVec<T> {
    type Item = usize;

    type IntoIter = Iter<'a, FixedIntVec<T>>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { i: 0, v: self }
    }
}

impl<'a> IntoIterator for &'a DynamicIntVec {
    type Item = usize;

    type IntoIter = Iter<'a, DynamicIntVec>;

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

        // SAFETY: Bounds check already happened
        let res = unsafe { self.v.get_unchecked(self.i) };
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

        // SAFETY: Bounds check already happened
        let res = unsafe { self.v.get_unchecked(self.i) };
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
