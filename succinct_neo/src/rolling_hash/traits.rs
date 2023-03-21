use super::HashedBytes;

/// A trait for rolling hash functions which allow advancing through the text they hash.
///
/// # Examples
///
/// ```
/// use succinct_neo::rolling_hash::{RabinKarp, RollingHash};
///
/// let s = "hashhash";
///
/// // Create a new Rabin-Karp hasher with a window size of 4.
/// let mut rk = RabinKarp::new(s, 4);
///
/// // Get the hashed bytes at the current position (0)
/// let hash_0 = rk.hashed_bytes();
///
/// // Move forward 4 steps
/// rk.advance();
/// rk.advance();
/// rk.advance();
/// rk.advance();
///
/// // Get the hashed bytes at the current position (4)
/// let hash_4 = rk.hashed_bytes();
///
/// // The hashes at indices 0 and 4 should be the same!
/// assert_eq!(hash_0, hash_4);
/// ```
pub trait RollingHash<'a> {
    /// Gets the current hash value of this hasher.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::rolling_hash::{RabinKarp, RollingHash};
    ///
    /// let s = "hashhash";
    ///
    /// // Create a new Rabin-Karp hasher with a window size of 4.
    /// let rk = RabinKarp::new(s, 4);
    ///
    /// // The hash value for s[0..4]
    /// let hash = rk.hash();
    /// ```
    fn hash(&self) -> u64;

    /// Advances the hash function by one character in the text and returns the resulting hash
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::rolling_hash::{RabinKarp, RollingHash};
    ///
    /// let s = "hashhash";
    ///
    /// // Create a new Rabin-Karp hasher with a window size of 4.
    /// let mut rk = RabinKarp::new(s, 4);
    ///
    /// // Move forward by one character and return the hash value for s[1..5]
    /// let hash = rk.advance();
    ///
    /// assert_eq!(hash, rk.hash());
    /// ```
    fn advance(&mut self) -> u64;

    /// Advance the hasher n times. This is equivalent to calling [`RollingHash::advance`] n times
    /// and returning the result of the last call of advance.
    /// If n is zero, this just returns the current hash value.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of times to advance.
    fn advance_n(&mut self, n: usize) -> u64 {
        let mut hash = self.hash();
        for _ in 0..n {
            hash = self.advance();
        }
        hash
    }

    /// Returns the current hash value augmented with the slice of the text that was hashed.
    ///
    /// # Examples
    ///
    /// ```
    /// use succinct_neo::rolling_hash::{RabinKarp, RollingHash};
    ///
    /// let s = "hashhash";
    ///
    /// // Create a new Rabin-Karp hasher with a window size of 4.
    /// let mut rk = RabinKarp::new(s, 4);
    /// rk.advance();
    ///
    /// // The hash value for s[1..5] augmented with b"ashh"
    /// let hash = rk.hashed_bytes();
    ///
    /// assert_eq!(hash.bytes(), b"ashh");
    /// assert_eq!(hash.hash(), rk.hash());
    /// ```
    fn hashed_bytes(&self) -> HashedBytes<'a>;
}

#[cfg(test)]
mod test {
    use crate::rolling_hash::{RabinKarp, RollingHash};

    #[test]
    fn test_advance_n() {
        let s = "hashhash";

        // Create a new Rabin-Karp hasher with a window size of 4.
        let mut rk1 = RabinKarp::new(s, 4);
        rk1.advance();
        rk1.advance();
        rk1.advance();

        let mut rk2 = RabinKarp::new(s, 4);
        rk2.advance_n(3);
        
        assert_eq!(rk1.hashed_bytes(), rk2.hashed_bytes(), "advance different to advance_n");
    }
}
