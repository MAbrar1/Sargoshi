//! Fixed-size secret byte strings: zeroized on drop, constant-time equality, and a `Debug`
//! implementation that never prints the contents.

use core::fmt;

use subtle::ConstantTimeEq;
use zeroize::Zeroize;

/// `N` secret bytes. There is deliberately no `Clone`, no `Display` and no serialization.
pub struct SecretBytes<const N: usize>([u8; N]);

impl<const N: usize> SecretBytes<N> {
    /// Take ownership of secret bytes. The caller should zeroize its own copy if it keeps one.
    #[must_use]
    pub const fn new(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    /// Borrow the secret bytes. Callers must not copy them into long-lived or logged values.
    #[must_use]
    pub const fn expose(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> Drop for SecretBytes<N> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl<const N: usize> fmt::Debug for SecretBytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretBytes<{N}>(redacted)")
    }
}

impl<const N: usize> PartialEq for SecretBytes<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ct_eq(&other.0).into()
    }
}

impl<const N: usize> Eq for SecretBytes<N> {}

impl<const N: usize> From<[u8; N]> for SecretBytes<N> {
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes)
    }
}
