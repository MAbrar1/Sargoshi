//! C ABI over `sargoshi_core` for `dart:ffi`. This is the only crate that uses
//! `unsafe`; the core crate forbids it. Rules for every exported function:
//! never unwind across the boundary, never trust a pointer without checking it,
//! return a code instead of a message, and hand ownership of results to the
//! caller through `SgBuf` (freed with `sg_buf_free`, which zeroes first).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use zeroize::Zeroize;

use sargoshi_core::{aead, keywrap, padding, recovery, vault};

pub const SG_OK: i32 = 0;
pub const SG_ERR_ARG: i32 = 1;
pub const SG_ERR_FORMAT: i32 = 2;
pub const SG_ERR_DECRYPT: i32 = 3;
pub const SG_ERR_WEAK: i32 = 4;
pub const SG_ERR_PARAMS: i32 = 5;
pub const SG_ERR_INTERNAL: i32 = 6;
pub const SG_ERR_TOO_LARGE: i32 = 7;
pub const SG_ERR_PIN_SHORT: i32 = 8;
pub const SG_ERR_PEPPER: i32 = 9;
/// Returned by `sg_vault_verify` for a well-formed record and a wrong PIN.
pub const SG_NO_MATCH: i32 = 10;
pub const SG_ERR_KEY: i32 = 11;
pub const SG_ERR_OPEN: i32 = 12;

#[repr(C)]
pub struct SgBuf {
    pub ptr: *mut u8,
    pub len: usize,
}

impl SgBuf {
    const EMPTY: SgBuf = SgBuf {
        ptr: ptr::null_mut(),
        len: 0,
    };

    fn from_vec(v: Vec<u8>) -> SgBuf {
        let b = v.into_boxed_slice();
        let len = b.len();
        SgBuf {
            ptr: Box::into_raw(b) as *mut u8,
            len,
        }
    }
}

/// Borrow `len` bytes at `p`. Null is accepted only for an empty slice.
unsafe fn input<'a>(p: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    if p.is_null() || len > isize::MAX as usize {
        return None;
    }
    // SAFETY: caller guarantees `p` is valid for `len` bytes; checked non-null above.
    Some(unsafe { std::slice::from_raw_parts(p, len) })
}

fn guard(out: *mut SgBuf, f: impl FnOnce() -> Result<Vec<u8>, i32>) -> i32 {
    if out.is_null() {
        return SG_ERR_ARG;
    }
    // SAFETY: `out` is non-null and the caller provides writable storage for one SgBuf.
    unsafe { out.write(SgBuf::EMPTY) };
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => {
            // SAFETY: as above.
            unsafe { out.write(SgBuf::from_vec(v)) };
            SG_OK
        }
        Ok(Err(code)) => code,
        Err(_) => SG_ERR_INTERNAL,
    }
}

fn recovery_code(e: recovery::RecoveryError) -> i32 {
    use recovery::RecoveryError::*;
    match e {
        WeakPassphrase => SG_ERR_WEAK,
        BadParams => SG_ERR_PARAMS,
        BadFormat => SG_ERR_FORMAT,
        Decrypt => SG_ERR_DECRYPT,
        Random | Kdf => SG_ERR_INTERNAL,
    }
}

/// Pads `data` into its size bucket.
///
/// # Safety
/// `data` must be valid for `len` bytes and `out` must point to writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_pad(data: *const u8, len: usize, out: *mut SgBuf) -> i32 {
    guard(out, || {
        let d = unsafe { input(data, len) }.ok_or(SG_ERR_ARG)?;
        padding::pad(d).map_err(|_| SG_ERR_TOO_LARGE)
    })
}

/// # Safety
/// Same contract as [`sg_pad`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_unpad(data: *const u8, len: usize, out: *mut SgBuf) -> i32 {
    guard(out, || {
        let d = unsafe { input(data, len) }.ok_or(SG_ERR_ARG)?;
        padding::unpad(d)
            .map(<[u8]>::to_vec)
            .map_err(|_| SG_ERR_FORMAT)
    })
}

/// Creates a recovery file with the default Argon2 parameters. `pass` is UTF-8.
///
/// # Safety
/// Pointers must be valid for their lengths; `out` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_recovery_create(
    keys: *const u8,
    keys_len: usize,
    pass: *const u8,
    pass_len: usize,
    out: *mut SgBuf,
) -> i32 {
    guard(out, || {
        let k = unsafe { input(keys, keys_len) }.ok_or(SG_ERR_ARG)?;
        let p = unsafe { input(pass, pass_len) }.ok_or(SG_ERR_ARG)?;
        let p = std::str::from_utf8(p).map_err(|_| SG_ERR_ARG)?;
        recovery::create(k, p, recovery::KdfParams::DEFAULT).map_err(recovery_code)
    })
}

/// # Safety
/// Same contract as [`sg_recovery_create`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_recovery_restore(
    file: *const u8,
    file_len: usize,
    pass: *const u8,
    pass_len: usize,
    out: *mut SgBuf,
) -> i32 {
    guard(out, || {
        let f = unsafe { input(file, file_len) }.ok_or(SG_ERR_ARG)?;
        let p = unsafe { input(pass, pass_len) }.ok_or(SG_ERR_ARG)?;
        let p = std::str::from_utf8(p).map_err(|_| SG_ERR_ARG)?;
        recovery::restore(f, p)
            .map(|z| z.to_vec())
            .map_err(recovery_code)
    })
}

/// Zeroes and frees a buffer returned by this library. Safe on an empty buffer.
///
/// # Safety
/// `buf` must come from this library and must not be freed twice.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_buf_free(buf: SgBuf) {
    if buf.ptr.is_null() {
        return;
    }
    // SAFETY: ptr/len came from Box<[u8]> in `SgBuf::from_vec`.
    let mut b = unsafe { Box::from_raw(ptr::slice_from_raw_parts_mut(buf.ptr, buf.len)) };
    b.zeroize();
}

fn vault_code(e: vault::VaultError) -> i32 {
    use vault::VaultError::*;
    match e {
        PinTooShort => SG_ERR_PIN_SHORT,
        BadPepper => SG_ERR_PEPPER,
        BadParams => SG_ERR_PARAMS,
        BadFormat => SG_ERR_FORMAT,
        Random | Kdf => SG_ERR_INTERNAL,
    }
}

/// Creates a vault verifier record for a PIN (UTF-8) keyed with a 32-byte pepper.
///
/// # Safety
/// Pointers must be valid for their lengths; `out` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_vault_enroll(
    pin: *const u8,
    pin_len: usize,
    pepper: *const u8,
    pepper_len: usize,
    out: *mut SgBuf,
) -> i32 {
    guard(out, || {
        let p = unsafe { input(pin, pin_len) }.ok_or(SG_ERR_ARG)?;
        let p = std::str::from_utf8(p).map_err(|_| SG_ERR_ARG)?;
        let k = unsafe { input(pepper, pepper_len) }.ok_or(SG_ERR_ARG)?;
        vault::enroll(p, k, vault::VaultParams::DEFAULT).map_err(vault_code)
    })
}

/// Returns `SG_OK` on a match, `SG_NO_MATCH` for a wrong PIN, another code on error.
///
/// # Safety
/// Pointers must be valid for their lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_vault_verify(
    record: *const u8,
    record_len: usize,
    pin: *const u8,
    pin_len: usize,
    pepper: *const u8,
    pepper_len: usize,
) -> i32 {
    let run = || -> i32 {
        let Some(r) = (unsafe { input(record, record_len) }) else {
            return SG_ERR_ARG;
        };
        let Some(p) = (unsafe { input(pin, pin_len) }) else {
            return SG_ERR_ARG;
        };
        let Ok(p) = std::str::from_utf8(p) else {
            return SG_ERR_ARG;
        };
        let Some(k) = (unsafe { input(pepper, pepper_len) }) else {
            return SG_ERR_ARG;
        };
        match vault::verify(r, p, k) {
            Ok(true) => SG_OK,
            Ok(false) => SG_NO_MATCH,
            Err(e) => vault_code(e),
        }
    };
    catch_unwind(AssertUnwindSafe(run)).unwrap_or(SG_ERR_INTERNAL)
}

/// Generates a random 32-byte key.
///
/// # Safety
/// `out` must point to writable storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_key_generate(out: *mut SgBuf) -> i32 {
    guard(out, || {
        aead::generate_key()
            .map(|k| k.to_vec())
            .map_err(|_| SG_ERR_INTERNAL)
    })
}

fn aead_code(e: aead::AeadError) -> i32 {
    match e {
        aead::AeadError::BadKey => SG_ERR_KEY,
        aead::AeadError::Random => SG_ERR_INTERNAL,
        aead::AeadError::Open => SG_ERR_OPEN,
    }
}

/// Seals `plain` under a 32-byte `key`, binding `aad`. Output is nonce||ciphertext.
///
/// # Safety
/// Pointers must be valid for their lengths; `out` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_seal(
    key: *const u8,
    key_len: usize,
    aad: *const u8,
    aad_len: usize,
    plain: *const u8,
    plain_len: usize,
    out: *mut SgBuf,
) -> i32 {
    guard(out, || {
        let k = unsafe { input(key, key_len) }.ok_or(SG_ERR_ARG)?;
        let a = unsafe { input(aad, aad_len) }.ok_or(SG_ERR_ARG)?;
        let p = unsafe { input(plain, plain_len) }.ok_or(SG_ERR_ARG)?;
        aead::seal(k, a, p).map_err(aead_code)
    })
}

/// # Safety
/// Same contract as [`sg_seal`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_open(
    key: *const u8,
    key_len: usize,
    aad: *const u8,
    aad_len: usize,
    sealed: *const u8,
    sealed_len: usize,
    out: *mut SgBuf,
) -> i32 {
    guard(out, || {
        let k = unsafe { input(key, key_len) }.ok_or(SG_ERR_ARG)?;
        let a = unsafe { input(aad, aad_len) }.ok_or(SG_ERR_ARG)?;
        let c = unsafe { input(sealed, sealed_len) }.ok_or(SG_ERR_ARG)?;
        aead::open(k, a, c).map(|z| z.to_vec()).map_err(aead_code)
    })
}

fn wrap_code(e: keywrap::WrapError) -> i32 {
    use keywrap::WrapError::*;
    match e {
        PinTooShort => SG_ERR_PIN_SHORT,
        BadPepper => SG_ERR_PEPPER,
        BadMaster => SG_ERR_KEY,
        BadParams => SG_ERR_PARAMS,
        BadFormat => SG_ERR_FORMAT,
        Unwrap => SG_ERR_DECRYPT,
        Random | Kdf => SG_ERR_INTERNAL,
    }
}

/// Wraps a 32-byte master key under the PIN and pepper.
///
/// # Safety
/// Pointers must be valid for their lengths; `out` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_key_wrap(
    master: *const u8,
    master_len: usize,
    pin: *const u8,
    pin_len: usize,
    pepper: *const u8,
    pepper_len: usize,
    out: *mut SgBuf,
) -> i32 {
    guard(out, || {
        let m = unsafe { input(master, master_len) }.ok_or(SG_ERR_ARG)?;
        let p = unsafe { input(pin, pin_len) }.ok_or(SG_ERR_ARG)?;
        let p = std::str::from_utf8(p).map_err(|_| SG_ERR_ARG)?;
        let k = unsafe { input(pepper, pepper_len) }.ok_or(SG_ERR_ARG)?;
        keywrap::wrap(m, p, k, keywrap::WrapParams::DEFAULT).map_err(wrap_code)
    })
}

/// # Safety
/// Same contract as [`sg_key_wrap`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_key_unwrap(
    blob: *const u8,
    blob_len: usize,
    pin: *const u8,
    pin_len: usize,
    pepper: *const u8,
    pepper_len: usize,
    out: *mut SgBuf,
) -> i32 {
    guard(out, || {
        let b = unsafe { input(blob, blob_len) }.ok_or(SG_ERR_ARG)?;
        let p = unsafe { input(pin, pin_len) }.ok_or(SG_ERR_ARG)?;
        let p = std::str::from_utf8(p).map_err(|_| SG_ERR_ARG)?;
        let k = unsafe { input(pepper, pepper_len) }.ok_or(SG_ERR_ARG)?;
        keywrap::unwrap(b, p, k)
            .map(|z| z.to_vec())
            .map_err(wrap_code)
    })
}

/// Frees the buffer behind `buf` (zeroing it) and resets `buf` to empty.
///
/// # Safety
/// `buf` must be null or point to an `SgBuf` produced by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_buf_release(buf: *mut SgBuf) {
    if buf.is_null() {
        return;
    }
    // SAFETY: non-null, caller guarantees it is a valid SgBuf.
    unsafe {
        let b = buf.read();
        buf.write(SgBuf::EMPTY);
        sg_buf_free(b);
    }
}

const HDR: usize = 8;

/// Allocates `len` zeroed bytes for the caller to fill with input. Free with
/// `sg_dealloc`. Lets Dart pass inputs without extra packages.
#[unsafe(no_mangle)]
pub extern "C" fn sg_alloc(len: usize) -> *mut u8 {
    let Some(total) = len.checked_add(HDR) else {
        return ptr::null_mut();
    };
    let mut v = vec![0u8; total];
    v[..HDR].copy_from_slice(&(len as u64).to_le_bytes());
    let raw = Box::into_raw(v.into_boxed_slice()) as *mut u8;
    // SAFETY: raw points to `total` bytes, HDR <= total.
    unsafe { raw.add(HDR) }
}

/// Zeroes and frees memory from `sg_alloc`. Null is ignored.
///
/// # Safety
/// `p` must come from `sg_alloc` and must not be freed twice.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sg_dealloc(p: *mut u8) {
    if p.is_null() {
        return;
    }
    // SAFETY: p was returned by sg_alloc, so HDR bytes before it hold the length.
    unsafe {
        let raw = p.sub(HDR);
        let mut hdr = [0u8; HDR];
        ptr::copy_nonoverlapping(raw, hdr.as_mut_ptr(), HDR);
        let len = u64::from_le_bytes(hdr) as usize;
        let mut b = Box::from_raw(ptr::slice_from_raw_parts_mut(raw, len + HDR));
        b.zeroize();
    }
}

/// ABI version so Dart can refuse a mismatched library.
#[unsafe(no_mangle)]
pub extern "C" fn sg_abi_version() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe fn take(b: SgBuf) -> Vec<u8> {
        let v = if b.ptr.is_null() {
            vec![]
        } else {
            unsafe { std::slice::from_raw_parts(b.ptr, b.len) }.to_vec()
        };
        unsafe { sg_buf_free(b) };
        v
    }

    #[test]
    fn pad_roundtrip() {
        unsafe {
            let mut o = SgBuf::EMPTY;
            assert_eq!(sg_pad(b"hello".as_ptr(), 5, &mut o), SG_OK);
            let padded = take(o);
            assert_eq!(padded.len(), 256);
            let mut o2 = SgBuf::EMPTY;
            assert_eq!(sg_unpad(padded.as_ptr(), padded.len(), &mut o2), SG_OK);
            assert_eq!(take(o2), b"hello");
        }
    }

    #[test]
    fn rejects_null_and_bad_input() {
        unsafe {
            let mut o = SgBuf::EMPTY;
            assert_eq!(sg_pad(ptr::null(), 3, &mut o), SG_ERR_ARG);
            assert!(o.ptr.is_null());
            assert_eq!(sg_pad(b"x".as_ptr(), 1, ptr::null_mut()), SG_ERR_ARG);
            assert_eq!(sg_unpad([1u8, 2].as_ptr(), 2, &mut o), SG_ERR_FORMAT);
            let bad = [0xffu8, 0xfe];
            assert_eq!(
                sg_recovery_create(b"k".as_ptr(), 1, bad.as_ptr(), 2, &mut o),
                SG_ERR_ARG
            );
            sg_buf_free(SgBuf::EMPTY);
        }
    }

    #[test]
    fn recovery_roundtrip_and_errors() {
        unsafe {
            let pass = b"correct horse battery";
            let keys = b"identity-keys";
            let mut o = SgBuf::EMPTY;
            assert_eq!(
                sg_recovery_create(keys.as_ptr(), keys.len(), pass.as_ptr(), pass.len(), &mut o),
                SG_OK
            );
            let file = take(o);
            let mut r = SgBuf::EMPTY;
            assert_eq!(
                sg_recovery_restore(file.as_ptr(), file.len(), pass.as_ptr(), pass.len(), &mut r),
                SG_OK
            );
            assert_eq!(take(r), keys);
            let wrong = b"wrong horse battery";
            let mut r2 = SgBuf::EMPTY;
            assert_eq!(
                sg_recovery_restore(
                    file.as_ptr(),
                    file.len(),
                    wrong.as_ptr(),
                    wrong.len(),
                    &mut r2
                ),
                SG_ERR_DECRYPT
            );
            let mut w = SgBuf::EMPTY;
            assert_eq!(
                sg_recovery_create(keys.as_ptr(), keys.len(), b"short".as_ptr(), 5, &mut w),
                SG_ERR_WEAK
            );
        }
    }

    #[test]
    fn alloc_and_release_roundtrip() {
        unsafe {
            let p = sg_alloc(5);
            assert!(!p.is_null());
            p.copy_from_nonoverlapping(b"hello".as_ptr(), 5);
            let mut o = SgBuf::EMPTY;
            assert_eq!(sg_pad(p, 5, &mut o), SG_OK);
            sg_dealloc(p);
            sg_buf_release(&mut o);
            assert!(o.ptr.is_null());
            sg_buf_release(&mut o);
            sg_dealloc(ptr::null_mut());
        }
    }

    #[test]
    fn vault_enroll_and_verify() {
        unsafe {
            let pepper = [3u8; 32];
            let mut o = SgBuf::EMPTY;
            assert_eq!(
                sg_vault_enroll(b"2468".as_ptr(), 4, pepper.as_ptr(), 32, &mut o),
                SG_OK
            );
            let rec = take(o);
            let v = |pin: &[u8], pep: &[u8]| {
                sg_vault_verify(
                    rec.as_ptr(),
                    rec.len(),
                    pin.as_ptr(),
                    pin.len(),
                    pep.as_ptr(),
                    pep.len(),
                )
            };
            assert_eq!(v(b"2468", &pepper), SG_OK);
            assert_eq!(v(b"2469", &pepper), SG_NO_MATCH);
            assert_eq!(v(b"2468", &[3u8; 5]), SG_ERR_PEPPER);
            let mut w = SgBuf::EMPTY;
            assert_eq!(
                sg_vault_enroll(b"12".as_ptr(), 2, pepper.as_ptr(), 32, &mut w),
                SG_ERR_PIN_SHORT
            );
        }
    }

    #[test]
    fn seal_open_and_wrap_roundtrip() {
        unsafe {
            let mut k = SgBuf::EMPTY;
            assert_eq!(sg_key_generate(&mut k), SG_OK);
            let key = take(k);
            assert_eq!(key.len(), 32);
            let mut s = SgBuf::EMPTY;
            assert_eq!(
                sg_seal(
                    key.as_ptr(),
                    32,
                    b"id:1".as_ptr(),
                    4,
                    b"hi".as_ptr(),
                    2,
                    &mut s
                ),
                SG_OK
            );
            let sealed = take(s);
            let mut o = SgBuf::EMPTY;
            assert_eq!(
                sg_open(
                    key.as_ptr(),
                    32,
                    b"id:1".as_ptr(),
                    4,
                    sealed.as_ptr(),
                    sealed.len(),
                    &mut o
                ),
                SG_OK
            );
            assert_eq!(take(o), b"hi");
            let mut bad = SgBuf::EMPTY;
            assert_eq!(
                sg_open(
                    key.as_ptr(),
                    32,
                    b"id:2".as_ptr(),
                    4,
                    sealed.as_ptr(),
                    sealed.len(),
                    &mut bad
                ),
                SG_ERR_OPEN
            );

            let pepper = [6u8; 32];
            let mut w = SgBuf::EMPTY;
            assert_eq!(
                sg_key_wrap(
                    key.as_ptr(),
                    32,
                    b"2468".as_ptr(),
                    4,
                    pepper.as_ptr(),
                    32,
                    &mut w
                ),
                SG_OK
            );
            let wrapped = take(w);
            let mut u = SgBuf::EMPTY;
            assert_eq!(
                sg_key_unwrap(
                    wrapped.as_ptr(),
                    wrapped.len(),
                    b"2468".as_ptr(),
                    4,
                    pepper.as_ptr(),
                    32,
                    &mut u
                ),
                SG_OK
            );
            assert_eq!(take(u), key);
            let mut u2 = SgBuf::EMPTY;
            assert_eq!(
                sg_key_unwrap(
                    wrapped.as_ptr(),
                    wrapped.len(),
                    b"1111".as_ptr(),
                    4,
                    pepper.as_ptr(),
                    32,
                    &mut u2
                ),
                SG_ERR_DECRYPT
            );
        }
    }

    #[test]
    fn abi_version_is_three() {
        assert_eq!(sg_abi_version(), 3);
    }
}
