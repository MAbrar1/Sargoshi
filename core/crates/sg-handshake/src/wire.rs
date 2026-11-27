//! Canonical wire encodings of bundles and the initial header: `version || protobuf(body)`.
//! The decoder re-encodes what it parsed and rejects the input unless the bytes are identical,
//! so unknown fields and non-minimal encodings are refused. Parsing a bundle verifies every
//! signature in it, so a parsed value is always a verified one.

use prost::Message;
use sg_common::constants::MAX_BUNDLE_BYTES;
use sg_common::{PROTOCOL_VERSION, ProtoError, check_version};
use sg_crypto::ed25519::Signature;
use sg_crypto::kem::{MlKem768Ciphertext, MlKem768PublicKey};
use sg_crypto::x25519::X25519PublicKey;

use crate::error::HandshakeError;
use crate::handshake::InitialHeader;
use crate::identity::IdentityPublic;
use crate::prekeys::{
    OneTimePreKeyPublic, PqPreKeyPublic, PreKeyBundle, SignedPreKeyPublic, UploadBundle,
};

#[derive(Clone, PartialEq, Message)]
struct IdentityWire {
    #[prost(bytes = "vec", tag = "1")]
    signing: Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    dh: Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    dh_signature: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct SignedPreKeyWire {
    #[prost(uint32, tag = "1")]
    id: u32,
    #[prost(uint64, tag = "2")]
    created_at: u64,
    #[prost(bytes = "vec", tag = "3")]
    public: Vec<u8>,
    #[prost(bytes = "vec", tag = "4")]
    signature: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct PqPreKeyWire {
    #[prost(uint32, tag = "1")]
    id: u32,
    #[prost(bytes = "vec", tag = "2")]
    encapsulation_key: Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    signature: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct OneTimePreKeyWire {
    #[prost(uint32, tag = "1")]
    id: u32,
    #[prost(bytes = "vec", tag = "2")]
    public: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct PreKeyBundleWire {
    #[prost(message, optional, tag = "1")]
    identity: Option<IdentityWire>,
    #[prost(message, optional, tag = "2")]
    signed_prekey: Option<SignedPreKeyWire>,
    #[prost(message, optional, tag = "3")]
    pq_prekey: Option<PqPreKeyWire>,
    #[prost(message, optional, tag = "4")]
    one_time_prekey: Option<OneTimePreKeyWire>,
}

#[derive(Clone, PartialEq, Message)]
struct UploadBundleWire {
    #[prost(message, optional, tag = "1")]
    signed_prekey: Option<SignedPreKeyWire>,
    #[prost(message, optional, tag = "2")]
    pq_prekey: Option<PqPreKeyWire>,
    #[prost(message, repeated, tag = "3")]
    one_time_prekeys: Vec<OneTimePreKeyWire>,
}

#[derive(Clone, PartialEq, Message)]
struct InitialHeaderWire {
    #[prost(message, optional, tag = "1")]
    identity: Option<IdentityWire>,
    #[prost(bytes = "vec", tag = "2")]
    ephemeral: Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    kem_ciphertext: Vec<u8>,
    #[prost(uint32, optional, tag = "4")]
    signed_prekey_id: Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pq_prekey_id: Option<u32>,
    #[prost(uint32, optional, tag = "6")]
    one_time_prekey_id: Option<u32>,
}

fn malformed(what: &'static str) -> HandshakeError {
    HandshakeError::Proto(ProtoError::Malformed(what))
}

fn encode_versioned<M: Message>(message: &M) -> Result<Vec<u8>, HandshakeError> {
    let mut out = Vec::with_capacity(1 + message.encoded_len());
    out.push(PROTOCOL_VERSION);
    message.encode(&mut out).map_err(|_| malformed("encode"))?;
    if out.len() > MAX_BUNDLE_BYTES {
        return Err(HandshakeError::Proto(ProtoError::TooLarge {
            limit: MAX_BUNDLE_BYTES,
            actual: out.len(),
        }));
    }
    Ok(out)
}

fn decode_canonical<M: Message + Default>(bytes: &[u8]) -> Result<M, HandshakeError> {
    if bytes.is_empty() {
        return Err(HandshakeError::Proto(ProtoError::Truncated));
    }
    if bytes.len() > MAX_BUNDLE_BYTES {
        return Err(HandshakeError::Proto(ProtoError::TooLarge {
            limit: MAX_BUNDLE_BYTES,
            actual: bytes.len(),
        }));
    }
    let (version, rest) = bytes
        .split_first()
        .ok_or(HandshakeError::Proto(ProtoError::Truncated))?;
    check_version(*version)?;
    let message = M::decode(rest).map_err(|_| malformed("message body"))?;
    if message.encode_to_vec() != rest {
        return Err(HandshakeError::Proto(ProtoError::NonCanonical));
    }
    Ok(message)
}

fn identity_to_wire(identity: &IdentityPublic) -> IdentityWire {
    IdentityWire {
        signing: identity.signing_key().to_bytes().to_vec(),
        dh: identity.dh_key().as_bytes().to_vec(),
        dh_signature: identity.dh_signature().as_bytes().to_vec(),
    }
}

fn identity_from_wire(wire: Option<IdentityWire>) -> Result<IdentityPublic, HandshakeError> {
    let wire = wire.ok_or_else(|| malformed("identity"))?;
    IdentityPublic::new(&wire.signing, &wire.dh, &wire.dh_signature)
}

fn spk_to_wire(spk: &SignedPreKeyPublic) -> SignedPreKeyWire {
    SignedPreKeyWire {
        id: spk.id,
        created_at: spk.created_at,
        public: spk.public.as_bytes().to_vec(),
        signature: spk.signature.as_bytes().to_vec(),
    }
}

fn spk_from_wire(wire: Option<SignedPreKeyWire>) -> Result<SignedPreKeyPublic, HandshakeError> {
    let wire = wire.ok_or_else(|| malformed("signed prekey"))?;
    Ok(SignedPreKeyPublic {
        id: wire.id,
        created_at: wire.created_at,
        public: X25519PublicKey::from_slice(&wire.public)
            .map_err(|_| malformed("signed prekey key"))?,
        signature: Signature::from_slice(&wire.signature)
            .map_err(|_| malformed("signed prekey signature"))?,
    })
}

fn pq_to_wire(pq: &PqPreKeyPublic) -> PqPreKeyWire {
    PqPreKeyWire {
        id: pq.id,
        encapsulation_key: pq.encapsulation_key.as_bytes().to_vec(),
        signature: pq.signature.as_bytes().to_vec(),
    }
}

fn pq_from_wire(wire: Option<PqPreKeyWire>) -> Result<PqPreKeyPublic, HandshakeError> {
    let wire = wire.ok_or_else(|| malformed("post-quantum prekey"))?;
    Ok(PqPreKeyPublic {
        id: wire.id,
        encapsulation_key: MlKem768PublicKey::from_bytes(&wire.encapsulation_key)
            .map_err(|_| malformed("post-quantum prekey key"))?,
        signature: Signature::from_slice(&wire.signature)
            .map_err(|_| malformed("post-quantum prekey signature"))?,
    })
}

fn opk_to_wire(opk: &OneTimePreKeyPublic) -> OneTimePreKeyWire {
    OneTimePreKeyWire {
        id: opk.id,
        public: opk.public.as_bytes().to_vec(),
    }
}

fn opk_from_wire(wire: OneTimePreKeyWire) -> Result<OneTimePreKeyPublic, HandshakeError> {
    Ok(OneTimePreKeyPublic {
        id: wire.id,
        public: X25519PublicKey::from_slice(&wire.public)
            .map_err(|_| malformed("one-time prekey key"))?,
    })
}

impl PreKeyBundle {
    /// Verify the signed prekey and post-quantum prekey signatures. The identity inside is
    /// verified when it is constructed.
    ///
    /// # Errors
    /// The specific signature error for the first signature that fails.
    pub fn verify(&self) -> Result<(), HandshakeError> {
        self.signed_prekey.verify(self.identity.signing_key())?;
        self.pq_prekey.verify(self.identity.signing_key())
    }

    /// Serialize to `version || protobuf`.
    ///
    /// # Errors
    /// A protocol error if the result would exceed the size limit.
    pub fn to_bytes(&self) -> Result<Vec<u8>, HandshakeError> {
        encode_versioned(&PreKeyBundleWire {
            identity: Some(identity_to_wire(&self.identity)),
            signed_prekey: Some(spk_to_wire(&self.signed_prekey)),
            pq_prekey: Some(pq_to_wire(&self.pq_prekey)),
            one_time_prekey: self.one_time_prekey.as_ref().map(opk_to_wire),
        })
    }

    /// Parse strictly and verify every signature.
    ///
    /// # Errors
    /// A protocol error for malformed or non-canonical input, or the specific signature error.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HandshakeError> {
        let wire: PreKeyBundleWire = decode_canonical(bytes)?;
        let bundle = Self {
            identity: identity_from_wire(wire.identity)?,
            signed_prekey: spk_from_wire(wire.signed_prekey)?,
            pq_prekey: pq_from_wire(wire.pq_prekey)?,
            one_time_prekey: wire.one_time_prekey.map(opk_from_wire).transpose()?,
        };
        bundle.verify()?;
        Ok(bundle)
    }
}

impl UploadBundle {
    /// Verify the signatures against the identity the server has on record for the device.
    ///
    /// # Errors
    /// The specific signature error for the first signature that fails.
    pub fn verify(&self, identity: &IdentityPublic) -> Result<(), HandshakeError> {
        self.signed_prekey.verify(identity.signing_key())?;
        self.pq_prekey.verify(identity.signing_key())
    }

    /// Serialize to `version || protobuf`.
    ///
    /// # Errors
    /// A protocol error if the result would exceed the size limit.
    pub fn to_bytes(&self) -> Result<Vec<u8>, HandshakeError> {
        encode_versioned(&UploadBundleWire {
            signed_prekey: Some(spk_to_wire(&self.signed_prekey)),
            pq_prekey: Some(pq_to_wire(&self.pq_prekey)),
            one_time_prekeys: self.one_time_prekeys.iter().map(opk_to_wire).collect(),
        })
    }

    /// Parse strictly and verify the signatures under `identity`.
    ///
    /// # Errors
    /// A protocol error for malformed or non-canonical input, [`HandshakeError::TooManyKeys`] for
    /// too many one-time prekeys, or the specific signature error.
    pub fn from_bytes(bytes: &[u8], identity: &IdentityPublic) -> Result<Self, HandshakeError> {
        let wire: UploadBundleWire = decode_canonical(bytes)?;
        if wire.one_time_prekeys.len() > sg_common::constants::MAX_ONE_TIME_PREKEYS_PER_UPLOAD {
            return Err(HandshakeError::TooManyKeys);
        }
        let bundle = Self {
            signed_prekey: spk_from_wire(wire.signed_prekey)?,
            pq_prekey: pq_from_wire(wire.pq_prekey)?,
            one_time_prekeys: wire
                .one_time_prekeys
                .into_iter()
                .map(opk_from_wire)
                .collect::<Result<_, _>>()?,
        };
        bundle.verify(identity)?;
        Ok(bundle)
    }
}

impl InitialHeader {
    /// Serialize to `version || protobuf`.
    ///
    /// # Errors
    /// A protocol error if the result would exceed the size limit.
    pub fn to_bytes(&self) -> Result<Vec<u8>, HandshakeError> {
        encode_versioned(&InitialHeaderWire {
            identity: Some(identity_to_wire(&self.identity)),
            ephemeral: self.ephemeral.as_bytes().to_vec(),
            kem_ciphertext: self.kem_ciphertext.as_bytes().to_vec(),
            signed_prekey_id: Some(self.signed_prekey_id),
            pq_prekey_id: Some(self.pq_prekey_id),
            one_time_prekey_id: self.one_time_prekey_id,
        })
    }

    /// Parse strictly; the initiator's identity signature is verified.
    ///
    /// # Errors
    /// A protocol error for malformed or non-canonical input, or an identity error.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HandshakeError> {
        let wire: InitialHeaderWire = decode_canonical(bytes)?;
        Ok(Self {
            identity: identity_from_wire(wire.identity)?,
            ephemeral: X25519PublicKey::from_slice(&wire.ephemeral)
                .map_err(|_| malformed("ephemeral key"))?,
            kem_ciphertext: MlKem768Ciphertext::from_slice(&wire.kem_ciphertext)
                .map_err(|_| malformed("kem ciphertext"))?,
            signed_prekey_id: wire
                .signed_prekey_id
                .ok_or_else(|| malformed("signed prekey id"))?,
            pq_prekey_id: wire
                .pq_prekey_id
                .ok_or_else(|| malformed("post-quantum prekey id"))?,
            one_time_prekey_id: wire.one_time_prekey_id,
        })
    }
}
