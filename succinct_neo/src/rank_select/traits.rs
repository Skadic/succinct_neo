pub trait RankSupport<T = u8>
where
    T: Eq,
{
    fn rank(index: usize, value: T);
}

pub trait SelectSupport<T = u8>
where
    T: Eq,
{
    fn select(rank: usize, value: T);
}
