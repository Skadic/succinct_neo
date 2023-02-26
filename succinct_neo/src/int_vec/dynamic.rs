use crate::int_vec::{Dynamic, IntAccess, IntVec};

impl IntVec<Dynamic> {
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
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// // 32 * 10 makes 320 bits, requiring 5 * 64bit blocks.
    /// assert_eq!(5, IntVec::num_required_blocks::<u64>(32, 10))
    /// ```
    #[inline]
    pub fn num_required_blocks<T>(num_elements: usize, bit_width: usize) -> usize {
        (num_elements as f64 * bit_width as f64 / (std::mem::size_of::<T>() as f64 * 8.0)).ceil()
            as usize
    }

    #[inline]
    fn recalculate_capacity(&mut self) {
        self.capacity = self.data.capacity() * Self::block_width() / self.width;
    }

    /// Creates an integer vector with a given bit width and a default capacity of 8.
    ///
    /// # Arguments
    ///
    /// * `width` - The bit width for each integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let v = IntVec::new(15);
    ///
    /// // 8 integers of size 15 require 120 bits this in turn requiring 2 * 64 blocks (= 128 bits).
    /// // These can hold 8 integers exactly.
    /// assert_eq!(8, v.capacity());
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub fn new(width: usize) -> Self {
        Self::with_capacity(width, 8)
    }

    /// Creates an integer vector with a given bit width and capacity.
    ///
    /// # Arguments
    ///
    /// * `width` - The bit width for each integer.
    /// * `capacity` - The number of integers which should fit into this vector without
    /// reallocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let v = IntVec::with_capacity(15, 20);
    ///
    /// // 20 integers of size 15 require 300 bits this in turn requiring 5 * 64 blocks (= 320 bits).
    /// // However, 320 bits can hold 21 integers of size 15
    /// assert_eq!(21, v.capacity());
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub fn with_capacity(width: usize, capacity: usize) -> Self {
        let num_blocks = Self::num_required_blocks::<usize>(capacity, width);

        let mut temp = Self {
            data: Vec::with_capacity(num_blocks),
            width,
            capacity: num_blocks * Self::block_width() / width,
            size: 0,
            _marker: Default::default()
        };

        temp.data.push(0);
        temp
    }

    /// Calculates the current offset inside the last used block where the next integer would be
    /// inserted.
    #[inline]
    fn current_offset(&self) -> usize {
        (self.size * self.width) % Self::block_width()
    }

    /// Gets the number of bits each integer is saved with.
    /// In our case, this is the same as the generic type parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let v = IntVec::new(15);
    ///
    /// assert_eq!(15, v.bit_width());
    /// ```
    #[inline]
    pub const fn bit_width(&self) -> usize {
        self.width
    }

    #[inline]
    const fn mask(&self) -> usize {
        (1 << self.width) - 1
    }

    /// Adds an integer to the end of the vector.
    ///
    /// # Arguments
    ///
    /// * `v` - The value to insert.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{IntVec, IntAccess};
    ///
    /// let mut v = IntVec::new(10);
    /// v.push(25);
    /// v.push(8);
    /// v.push(60);
    ///
    /// assert_eq!(25, v.get(0));
    /// assert_eq!(8, v.get(1));
    /// assert_eq!(60, v.get(2));
    /// ```
    pub fn push(&mut self, v: usize) {
        if v >= (1 << self.width) {
            panic!("value too large for {}-bit integer", self.width)
        }
        let offset = self.current_offset();
        let mask = self.mask();
        if offset == 0 {
            *self.data.last_mut().unwrap() |= v & mask;
            self.size += 1;
            return;
        }

        // If we're wrapping into the next block
        if offset + self.width >= Self::block_width() {
            let fitting_bits = Self::block_width() - offset;
            let fitting_mask = (1 << fitting_bits) - 1;
            let mask = (1 << self.width) - 1;
            *self.data.last_mut().unwrap() |= (v & fitting_mask) << offset;
            let hi = (v & mask) >> fitting_bits;
            self.data.push(hi);
            self.recalculate_capacity();
            self.size += 1;
            return;
        }

        *self.data.last_mut().unwrap() |= (v & mask) << offset;
        self.size += 1;
    }

    /// Grants access to the underlying slice where the bits are saved.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{IntVec, IntAccess};
    ///
    /// let mut v = IntVec::new(32);
    /// v.push(125);
    /// v.push(1231);
    ///
    /// assert_eq!((1231 << 32) | 125, v.raw_data()[0]);
    /// ```
    #[inline]
    pub fn raw_data(&self) -> &[usize] {
        &self.data
    }

    #[inline]
    /// Checks whether this has no integers saved.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::{IntVec, IntAccess};
    ///
    /// let mut v = IntVec::new(32);
    ///
    /// assert!(v.is_empty());
    ///
    /// v.push(125);
    ///
    /// assert!(!v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    /// The amount of integers saved in this vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let mut v = IntVec::new(10);
    /// for i in 0..20 {
    ///     v.push(i);
    /// }
    ///
    /// assert_eq!(20, v.len());
    ///
    /// ```
    pub fn len(&self) -> usize {
        self.size
    }

    /// An iterator over this vector's saved integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let mut v = IntVec::new(10);
    /// for i in (0..20).rev() {
    ///     v.push(i);
    /// }
    ///
    /// assert!(Iterator::eq((0..20).rev(), v.iter()))
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter { i: 0, v: self }
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

    /// Modifies this vector to require the minimum amount of bits per saved element.
    ///
    /// This searches for the largest element in the vector. It then saves all saved all integers
    /// with its number of required bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let mut v = IntVec::with_capacity(9, 25);
    ///
    /// // All these numbers should take 3 bits to save
    /// for i in (0..50).step_by(2) {
    ///     v.push(i % 8)
    /// }
    ///
    /// v.bit_compress();
    ///
    /// assert_eq!(3, v.bit_width());
    /// ```
    pub fn bit_compress(&mut self) {
        let Some(min_required_bits) = self.iter().reduce(|acc, v| { acc.max(v) }).map(|min| if min > 1 { (min - 1).ilog2() as usize + 1 } else { 1 }) else {
            // No elements in here
            return;
        };

        debug_assert!(min_required_bits <= self.width, "minimum required bits for the elements in this vector greater than previous word width");

        let old_width = self.width;
        self.width = min_required_bits;
        self.recalculate_capacity();

        for i in 0..self.len() {
            // SAFETY: we know the amount of values in this bitvector, so there's no problem
            unsafe {
                let v = self.get_unchecked_with_width(i, old_width);
                self.set_unchecked_with_width(i, v, self.width)
            }
        }
    }

    /// Shrinks the allocated backing storage behind this int vector to fit the amount of saved
    /// integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::int_vec::IntVec;
    ///
    /// let mut v = IntVec::with_capacity(5, 200);
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
        let required_blocks = Self::num_required_blocks::<usize>(self.size, self.width);
        self.data.truncate(required_blocks);
        self.data.shrink_to_fit();
        self.recalculate_capacity();
    }
}
impl IntAccess for IntVec<Dynamic> {
    unsafe fn get_unchecked(&self, index: usize) -> usize {
        self.get_unchecked_with_width(index, self.width)
    }

    fn get(&self, index: usize) -> usize {
        if index >= self.len() {
            panic!("length is {} but index is {index}", self.len())
        }

        unsafe { self.get_unchecked(index) }
    }

    unsafe fn set_unchecked(&mut self, index: usize, value: usize) {
        self.set_unchecked_with_width(index, value, self.width)
    }

    fn set(&mut self, index: usize, value: usize) {
        if index >= self.len() {
            panic!("length is {} but index is {index}", self.len())
        }
        if value >= (1 << self.width) {
            panic!("value {value} too large for {}-bit integer", self.width)
        }
        unsafe { self.set_unchecked(index, value) }
    }
}

impl IntoIterator for IntVec<Dynamic> {
    type Item = usize;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { i: 0, v: self }
    }
}

impl<'a> IntoIterator for &'a IntVec<Dynamic> {
    type Item = usize;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { i: 0, v: self }
    }
}

pub struct IntoIter {
    i: usize,
    v: IntVec<Dynamic>,
}

impl Iterator for IntoIter {
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

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.v.len() - self.i
    }
}

pub struct Iter<'a> {
    i: usize,
    v: &'a IntVec<Dynamic>,
}

impl Iterator for Iter<'_> {
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

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.v.len() - self.i
    }
}
