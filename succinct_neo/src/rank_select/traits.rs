
pub trait RankSupport {
    fn rank<const TARGET: bool>(&self, index: usize) -> usize;
}
