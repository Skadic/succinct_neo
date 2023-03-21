use super::HashedBytes;

pub trait RollingHash<'a> {
    fn hash(&self) -> u64;
    fn advance(&mut self) -> u64;
    fn hashed_bytes(&self) -> HashedBytes<'a>;
}
