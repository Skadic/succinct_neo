use std::fmt::Debug;
use std::hash::{Hash, Hasher};

mod rabin_karp;

pub use rabin_karp::RabinKarp;

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
    pub fn new(bytes: &'a [u8], hash: u64) -> Self {
        Self { bytes, hash }
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }

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
