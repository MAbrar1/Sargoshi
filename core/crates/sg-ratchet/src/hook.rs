//! Extension point for ongoing post-quantum rekeying. Sargoshi does not implement it (see
//! docs/architecture/PQ-RATCHET.md for the design note); the default does nothing, so the
//! ratchet behaves exactly as specified. A future implementation supplies an extra secret that
//! is mixed into the root key derivation at each Diffie-Hellman ratchet step.

use sg_crypto::SecretBytes;

/// Which Diffie-Hellman ratchet step is being performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatchetDirection {
    /// Deriving a receiving chain from a newly seen remote ratchet key.
    Receiving,
    /// Deriving a sending chain from a freshly generated local ratchet key.
    Sending,
}

/// What a hook is told about the step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatchetStep<'a> {
    /// The kind of step.
    pub direction: RatchetDirection,
    /// The local ratchet public key used in this step.
    pub local_ratchet_public: &'a [u8; 32],
    /// The remote ratchet public key used in this step.
    pub remote_ratchet_public: &'a [u8; 32],
}

/// Supplies an optional extra secret for a ratchet step. Implementations must be pure functions
/// of their own state and the [`RatchetStep`]: a decryption that fails authentication is rolled
/// back, so the hook may be asked about the same step again. Both sides must supply the same
/// secret for a session to keep working. This trait is unstable until the post-quantum ratchet
/// is designed.
pub trait PqRatchetHook {
    /// The extra secret to mix into `KDF_RK` for this step, or `None` (the default).
    fn extra_root_secret(&self, _step: &RatchetStep<'_>) -> Option<SecretBytes<32>> {
        None
    }
}

/// The default hook: never supplies a secret.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoPqRatchet;

impl PqRatchetHook for NoPqRatchet {}
