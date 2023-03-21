use std::collections::{HashSet, HashMap};
use std::fmt::Debug;
use std::hash::{Hash, Hasher, BuildHasherDefault};

mod cyclic_polynomial;
mod rabin_karp;
mod traits;

pub use cyclic_polynomial::CyclicPolynomial;
pub use rabin_karp::RabinKarp;
pub use traits::*;

pub type HashedByteSet<'a> = HashSet<HashedBytes<'a>, HashedBytesBuildHasher>;
pub type HashedByteMap<'a, V=HashedBytes<'a>> = HashMap<HashedBytes<'a>, V, HashedBytesBuildHasher>;

/// A slice of a string augmented with its hash value.
/// Get instances of this through a call to [`RollingHash::hashed_bytes`].
/// This is mostly used in as a key for [`HashSet`] or [`HashMap`] using a [`HashedBytesBuildHasher`], 
/// allowing the set or map to directly use the stored `hash` field as a hash value. 
///
/// # Examples 
///
/// ```
/// use succinct_neo::rolling_hash::{HashedBytes, HashedByteSet, RabinKarp, RollingHash};
///
/// let s = "hashhash";
/// let mut rk = RabinKarp::new(s, 4);
///
/// let mut set = HashedByteSet::default();
///
/// // Insert the hash for s[0..4] = "hash"
/// let hb: HashedBytes = rk.hashed_bytes();
/// set.insert(hb);
///
/// // Advance by 4 characters
/// rk.advance();
/// rk.advance();
/// rk.advance();
/// rk.advance();
///
/// // assure that the set contains the hash for s[4..8] = "hash"
/// assert!(set.contains(&rk.hashed_bytes()));
/// ````
#[derive(Clone, Copy)]
pub struct HashedBytes<'a> {
    bytes: &'a [u8],
    hash: u64,
}

impl Debug for HashedBytes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashedBytes")
            .field("bytes", &String::from_utf8_lossy(self.bytes))
            .field("hash", &self.hash)
            .finish()
    }
}

impl<'a> HashedBytes<'a> {
    /// A new slice with an associated hash value.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The slice.
    /// * `hash` - An associated hash value, usually created by a rolling hash function.
    pub fn new(bytes: &'a [u8], hash: u64) -> Self {
        Self { bytes, hash }
    }

    /// Returns the byte slice.
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }

    /// Returns the hash value.
    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl Hash for HashedBytes<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
    }
}

impl PartialEq for HashedBytes<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for HashedBytes<'_> {}

/// Hasher only for HashedBytes
#[derive(Default)]
pub struct HashedBytesHasher(u64);
impl Hasher for HashedBytesHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = u64::from_ne_bytes(bytes.try_into().unwrap());
    }
}

pub type HashedBytesBuildHasher = BuildHasherDefault<HashedBytesHasher>;
