//! Bounds-checked binary reader and writer for secret key blobs. Hand-written (instead of
//! protobuf) so every secret passes through buffers this crate controls and zeroizes: the writer
//! is created with its exact final capacity, so it never reallocates and leaves no stale copy.

use thiserror::Error;
use zeroize::{Zeroize, Zeroizing};

use crate::secret::SecretBytes;
use crate::x25519::X25519SecretKey;

/// A blob is truncated, has trailing bytes, or is otherwise malformed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("malformed encoding")]
pub struct CodecError;

/// Reads fields from a blob; every read is bounds-checked and returns an error, never panics.
pub struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    /// Start reading `bytes`.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    /// Take the next `n` bytes.
    ///
    /// # Errors
    /// [`CodecError`] if fewer than `n` bytes remain.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], CodecError> {
        let (head, tail) = self.rest.split_at_checked(n).ok_or(CodecError)?;
        self.rest = tail;
        Ok(head)
    }

    /// Read one byte.
    ///
    /// # Errors
    /// [`CodecError`] if the input is exhausted.
    pub fn u8(&mut self) -> Result<u8, CodecError> {
        self.take(1)?.first().copied().ok_or(CodecError)
    }

    /// Read a big-endian `u16`.
    ///
    /// # Errors
    /// [`CodecError`] if fewer than 2 bytes remain.
    pub fn u16(&mut self) -> Result<u16, CodecError> {
        Ok(u16::from_be_bytes(
            <[u8; 2]>::try_from(self.take(2)?).map_err(|_| CodecError)?,
        ))
    }

    /// Read a big-endian `u32`.
    ///
    /// # Errors
    /// [`CodecError`] if fewer than 4 bytes remain.
    pub fn u32(&mut self) -> Result<u32, CodecError> {
        Ok(u32::from_be_bytes(
            <[u8; 4]>::try_from(self.take(4)?).map_err(|_| CodecError)?,
        ))
    }

    /// Read a big-endian `u64`.
    ///
    /// # Errors
    /// [`CodecError`] if fewer than 8 bytes remain.
    pub fn u64(&mut self) -> Result<u64, CodecError> {
        Ok(u64::from_be_bytes(
            <[u8; 8]>::try_from(self.take(8)?).map_err(|_| CodecError)?,
        ))
    }

    /// Read 32 bytes of non-secret data.
    ///
    /// # Errors
    /// [`CodecError`] if fewer than 32 bytes remain.
    pub fn array32(&mut self) -> Result<[u8; 32], CodecError> {
        <[u8; 32]>::try_from(self.take(32)?).map_err(|_| CodecError)
    }

    /// Read 32 secret bytes into a zeroizing container and wipe the temporary copy.
    ///
    /// # Errors
    /// [`CodecError`] if fewer than 32 bytes remain.
    pub fn secret32(&mut self) -> Result<SecretBytes<32>, CodecError> {
        let mut tmp = self.array32()?;
        let secret = SecretBytes::new(tmp);
        tmp.zeroize();
        Ok(secret)
    }

    /// Read an X25519 secret key.
    ///
    /// # Errors
    /// [`CodecError`] if fewer than 32 bytes remain.
    pub fn x25519_secret(&mut self) -> Result<X25519SecretKey, CodecError> {
        let secret = self.secret32()?;
        Ok(X25519SecretKey::from_bytes(*secret.expose()))
    }

    /// Read 64 secret bytes.
    ///
    /// # Errors
    /// [`CodecError`] if fewer than 64 bytes remain.
    pub fn secret64(&mut self) -> Result<SecretBytes<64>, CodecError> {
        let mut tmp = <[u8; 64]>::try_from(self.take(64)?).map_err(|_| CodecError)?;
        let secret = SecretBytes::new(tmp);
        tmp.zeroize();
        Ok(secret)
    }

    /// Fail unless every byte was consumed.
    ///
    /// # Errors
    /// [`CodecError`] if bytes remain.
    pub fn finish(self) -> Result<(), CodecError> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(CodecError)
        }
    }
}

/// Builds a blob in a buffer of exactly the final size, zeroized on drop.
pub struct Writer {
    buf: Zeroizing<Vec<u8>>,
    capacity: usize,
}

impl Writer {
    /// Create a writer whose buffer is allocated once at `capacity` bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Zeroizing::new(Vec::with_capacity(capacity)),
            capacity,
        }
    }

    /// Append raw bytes.
    pub fn put(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// Append one byte.
    pub fn u8(&mut self, v: u8) {
        self.put(&[v]);
    }

    /// Append a big-endian `u16`.
    pub fn u16(&mut self, v: u16) {
        self.put(&v.to_be_bytes());
    }

    /// Append a big-endian `u32`.
    pub fn u32(&mut self, v: u32) {
        self.put(&v.to_be_bytes());
    }

    /// Append a big-endian `u64`.
    pub fn u64(&mut self, v: u64) {
        self.put(&v.to_be_bytes());
    }

    /// Finish. In debug builds this asserts that the capacity estimate was exact, which is what
    /// guarantees the buffer never reallocated.
    #[must_use]
    pub fn finish(self) -> Zeroizing<Vec<u8>> {
        debug_assert_eq!(
            self.buf.len(),
            self.capacity,
            "blob capacity estimate must be exact"
        );
        self.buf
    }
}
