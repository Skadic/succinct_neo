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
