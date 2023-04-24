use multimap::MultiMap;
use nohash_hasher::{BuildNoHashHasher, IntMap, IntSet, IsEnabled};

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

mod cyclic_polynomial;
mod rabin_karp;
mod traits;

pub use cyclic_polynomial::CyclicPolynomial;
pub use rabin_karp::RabinKarp;
pub use traits::*;

pub type HashedByteMap<'a, V = HashedBytes<'a>> = IntMap<HashedBytes<'a>, V>;
pub type HashedByteSet<'a> = IntSet<HashedBytes<'a>>;
pub type HashedByteMultiMap<'a, V = HashedBytes<'a>> =
    MultiMap<HashedBytes<'a>, V, BuildNoHashHasher<HashedBytes<'a>>>;
pub type HashedByteMultiSet<'a> = HashedByteMultiMap<'a, ()>;

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

impl IsEnabled for HashedBytes<'_> {}
