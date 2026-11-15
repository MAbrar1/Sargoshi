//! Message size padding. Hides exact plaintext length from the server.
//!
//! Layout: 4-byte big-endian length, plaintext, zero fill up to the bucket size.
//! Buckets: powers of two from 256 up to 16 KiB, then multiples of 16 KiB.

const HEADER: usize = 4;
const MIN_BUCKET: usize = 256;
const STEP: usize = 16 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum PadError {
    TooLarge,
    Malformed,
}

fn bucket_for(total: usize) -> usize {
    if total <= STEP {
        total.next_power_of_two().max(MIN_BUCKET)
    } else {
        total.div_ceil(STEP) * STEP
    }
}

pub fn pad(plain: &[u8]) -> Result<Vec<u8>, PadError> {
    let len = u32::try_from(plain.len()).map_err(|_| PadError::TooLarge)?;
    let total = HEADER
        .checked_add(plain.len())
        .ok_or(PadError::TooLarge)?;
    let mut out = Vec::with_capacity(bucket_for(total));
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(plain);
    out.resize(bucket_for(total), 0);
    Ok(out)
}

pub fn unpad(padded: &[u8]) -> Result<&[u8], PadError> {
    if padded.len() < HEADER {
        return Err(PadError::Malformed);
    }
    let len = u32::from_be_bytes([padded[0], padded[1], padded[2], padded[3]]) as usize;
    let end = HEADER.checked_add(len).ok_or(PadError::Malformed)?;
    if end > padded.len() || padded.len() != bucket_for(end) {
        return Err(PadError::Malformed);
    }
    if padded[end..].iter().any(|b| *b != 0) {
        return Err(PadError::Malformed);
    }
    Ok(&padded[HEADER..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_various_sizes() {
        for n in [0, 1, 251, 252, 253, 1000, 16380, 16381, 16384, 40000] {
            let data = vec![7u8; n];
            let p = pad(&data).unwrap();
            assert_eq!(unpad(&p).unwrap(), &data[..]);
        }
    }

    #[test]
    fn lengths_fall_in_buckets() {
        assert_eq!(pad(&[]).unwrap().len(), 256);
        assert_eq!(pad(&[0u8; 252]).unwrap().len(), 256);
        assert_eq!(pad(&[0u8; 253]).unwrap().len(), 512);
        assert_eq!(pad(&vec![0u8; 16381]).unwrap().len(), 32768);
        assert_eq!(pad(&vec![0u8; 40000]).unwrap().len(), 49152);
    }

    #[test]
    fn rejects_nonzero_fill_and_wrong_size() {
        let mut p = pad(b"hi").unwrap();
        *p.last_mut().unwrap() = 1;
        assert_eq!(unpad(&p), Err(PadError::Malformed));
        let mut q = pad(b"hi").unwrap();
        q.push(0);
        assert_eq!(unpad(&q), Err(PadError::Malformed));
        assert_eq!(unpad(&[1, 2]), Err(PadError::Malformed));
    }
}
