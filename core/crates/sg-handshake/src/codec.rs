//! Bounds-checked binary reader and writer for the secret key blobs. Hand-written (instead of
//! protobuf) so every secret passes through buffers this crate controls and zeroizes: the writer
//! is created with its exact final capacity, so it never reallocates and leaves no stale copy.

use sg_crypto::SecretBytes;
use sg_crypto::x25519::X25519SecretKey;
use zeroize::{Zeroize, Zeroizing};

use crate::error::HandshakeError;

/// Reads fields from a blob; every read is bounds-checked and returns an error, never panics.
pub(crate) struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8], HandshakeError> {
        let (head, tail) = self
            .rest
            .split_at_checked(n)
            .ok_or(HandshakeError::MalformedBlob)?;
        self.rest = tail;
        Ok(head)
    }

    pub(crate) fn u8(&mut self) -> Result<u8, HandshakeError> {
        self.take(1)?
            .first()
            .copied()
            .ok_or(HandshakeError::MalformedBlob)
    }

    pub(crate) fn u16(&mut self) -> Result<u16, HandshakeError> {
        let bytes =
            <[u8; 2]>::try_from(self.take(2)?).map_err(|_| HandshakeError::MalformedBlob)?;
        Ok(u16::from_be_bytes(bytes))
    }

    pub(crate) fn u32(&mut self) -> Result<u32, HandshakeError> {
        let bytes =
            <[u8; 4]>::try_from(self.take(4)?).map_err(|_| HandshakeError::MalformedBlob)?;
        Ok(u32::from_be_bytes(bytes))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, HandshakeError> {
        let bytes =
            <[u8; 8]>::try_from(self.take(8)?).map_err(|_| HandshakeError::MalformedBlob)?;
        Ok(u64::from_be_bytes(bytes))
    }

    pub(crate) fn array32(&mut self) -> Result<[u8; 32], HandshakeError> {
        <[u8; 32]>::try_from(self.take(32)?).map_err(|_| HandshakeError::MalformedBlob)
    }

    /// Read 32 secret bytes into a zeroizing container and wipe the temporary copy.
    pub(crate) fn secret32(&mut self) -> Result<SecretBytes<32>, HandshakeError> {
        let mut tmp = self.array32()?;
        let secret = SecretBytes::new(tmp);
        tmp.zeroize();
        Ok(secret)
    }

    pub(crate) fn x25519_secret(&mut self) -> Result<X25519SecretKey, HandshakeError> {
        let secret = self.secret32()?;
        Ok(X25519SecretKey::from_bytes(*secret.expose()))
    }

    pub(crate) fn secret64(&mut self) -> Result<SecretBytes<64>, HandshakeError> {
        let mut tmp =
            <[u8; 64]>::try_from(self.take(64)?).map_err(|_| HandshakeError::MalformedBlob)?;
        let secret = SecretBytes::new(tmp);
        tmp.zeroize();
        Ok(secret)
    }

    /// Fail unless every byte was consumed.
    pub(crate) fn finish(self) -> Result<(), HandshakeError> {
        if self.rest.is_empty() {
            Ok(())
        } else {
            Err(HandshakeError::MalformedBlob)
        }
    }
}

/// Builds a blob in a buffer of exactly the final size, zeroized on drop.
pub(crate) struct Writer {
    buf: Zeroizing<Vec<u8>>,
    capacity: usize,
}

impl Writer {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Zeroizing::new(Vec::with_capacity(capacity)),
            capacity,
        }
    }

    pub(crate) fn put(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub(crate) fn u8(&mut self, v: u8) {
        self.put(&[v]);
    }

    pub(crate) fn u16(&mut self, v: u16) {
        self.put(&v.to_be_bytes());
    }

    pub(crate) fn u32(&mut self, v: u32) {
        self.put(&v.to_be_bytes());
    }

    pub(crate) fn u64(&mut self, v: u64) {
        self.put(&v.to_be_bytes());
    }

    /// Finish. In debug builds this asserts that the capacity estimate was exact, which is what
    /// guarantees the buffer never reallocated.
    pub(crate) fn finish(self) -> Zeroizing<Vec<u8>> {
        debug_assert_eq!(
            self.buf.len(),
            self.capacity,
            "blob capacity estimate must be exact"
        );
        self.buf
    }
}
