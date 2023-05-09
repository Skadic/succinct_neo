/// Trait adding support for rank queries over bit vectors or similar data structures.
pub trait BitRankSupport {
    /// Calculates the number of zeroes or ones up to and not including a given index.
    ///
    /// This version uses const generics in hopes that the compiler can optimize the code better
    /// and should be preferred over [`BitRankSupport::rank_dyn`] if possible.
    ///
    /// # Generic Arguments
    ///
    /// * `TARGET` - `true` if ones should be ranked, `false` if zeroes should be counted.
    ///
    /// # Arguments
    ///
    /// * `index` - The index whose rank to calculate.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::{
    ///     bit_vec::BitVec,
    ///     bit_vec::rank_select::{
    ///         FlatPopcount,
    ///         BitRankSupport
    ///     }
    /// };
    ///
    /// let mut bv = BitVec::new(64);
    ///
    /// bv.flip(10);
    /// bv.flip(15);
    /// bv.flip(20);
    ///
    /// let rank_ds = FlatPopcount::<_, ()>::new(&bv);
    ///
    /// assert_eq!(0, rank_ds.rank::<true>(5));
    /// assert_eq!(0, rank_ds.rank::<true>(10));
    /// assert_eq!(1, rank_ds.rank::<true>(11));
    /// assert_eq!(3, rank_ds.rank::<true>(25));
    ///
    /// assert_eq!(5, rank_ds.rank::<false>(5));
    /// assert_eq!(10, rank_ds.rank::<false>(10));
    /// assert_eq!(10, rank_ds.rank::<false>(11));
    /// assert_eq!(22, rank_ds.rank::<false>(25));
    /// ```
    fn rank<const TARGET: bool>(&self, index: usize) -> usize;

    /// Calculates the number of zeroes or ones up to and not including a given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index whose rank to calculate.
    /// * `value` - `true` if ones should be ranked, `false` if zeroes should be counted.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::{
    ///     bit_vec::BitVec,
    ///     bit_vec::rank_select::{
    ///         FlatPopcount,
    ///         BitRankSupport
    ///     }
    /// };
    ///
    /// let mut bv = BitVec::new(64);
    ///
    /// bv.flip(10);
    /// bv.flip(15);
    /// bv.flip(20);
    ///
    /// let rank_ds = FlatPopcount::<_, ()>::new(&bv);
    ///
    /// assert_eq!(0, rank_ds.rank_dyn(5, true));
    /// assert_eq!(0, rank_ds.rank_dyn(10, true));
    /// assert_eq!(1, rank_ds.rank_dyn(11, true));
    /// assert_eq!(3, rank_ds.rank_dyn(25, true));
    ///
    /// assert_eq!(5, rank_ds.rank_dyn(5, false));
    /// assert_eq!(10, rank_ds.rank_dyn(10, false));
    /// assert_eq!(10, rank_ds.rank_dyn(11, false));
    /// assert_eq!(22, rank_ds.rank_dyn(25, false));
    /// ```
    fn rank_dyn(&self, index: usize, value: bool) -> usize {
        if value {
            self.rank::<true>(index)
        } else {
            self.rank::<false>(index)
        }
    }
}

/// Trait adding support for rank queries over bit vectors or similar data structures.
/// The `TARGET` parameter determines, whether this data structure supports select for `1` bits
/// (`TARGET` is `true`) or `0` bits (`TARGET` is `false`).
pub trait BitSelectSupport<const TARGET: bool> {
    /// Calculates the index of the nth time the given value shows up.
    ///
    /// If `TARGET` is `true`, this will search for the nth one, if it is `false`, this will
    /// search for zeroes.
    ///
    /// # Arguments
    ///
    /// * `rank` - The rank of the zero/one to find.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::{
    ///     bit_vec::BitVec,
    ///     bit_vec::rank_select::{
    ///         flat_popcount::LinearSearch,
    ///         FlatPopcount,
    ///         BitSelectSupport
    ///     }
    /// };
    ///
    /// let mut bv = BitVec::new(64);
    ///
    /// bv.flip(10);
    /// bv.flip(15);
    /// bv.flip(20);
    ///
    /// // This implements BitSelectSupport<true>
    /// let rank_ds = FlatPopcount::<_, LinearSearch>::new(&bv);
    ///
    ///
    /// assert_eq!(Some(10), rank_ds.select(0));
    /// assert_eq!(Some(15), rank_ds.select(1));
    /// assert_eq!(Some(20), rank_ds.select(2));
    /// assert_eq!(None, rank_ds.select(3));
    /// ```
    fn select(&self, rank: usize) -> Option<usize>;
}
