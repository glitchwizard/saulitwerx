//! SHA-256 and the `model_hash` of a document (`docs/specs/FILE_FORMAT.md`,
//! "`manifest.json`").

use crate::canonical::to_canonical_json;
use serde_json::Value;

/// SHA-256 round constants: the first 32 bits of the fractional parts of the
/// cube roots of the first 64 primes (FIPS 180-4, section 4.2.2).
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Initial hash value: the first 32 bits of the fractional parts of the
/// square roots of the first 8 primes (FIPS 180-4, section 5.3.3).
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// The SHA-256 digest of `bytes`.
///
/// Implemented from NIST FIPS 180-4, *Secure Hash Standard* (2015):
/// the functions Ch, Maj, Σ0, Σ1, σ0, σ1 (section 4.1.2), the constants
/// (section 4.2.2), padding (section 5.1.1), the initial hash value
/// (section 5.3.3) and the hash computation (section 6.2.2).
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut h = H0;
    // Padding (5.1.1): a 1 bit, zeros up to 56 mod 64 bytes, then the
    // message length in bits as a 64-bit big-endian integer.
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut tail = Vec::with_capacity(128);
    let full = bytes.len() - bytes.len() % 64;
    tail.extend_from_slice(&bytes[full..]);
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    tail.extend_from_slice(&bit_len.to_be_bytes());
    for block in bytes[..full].chunks_exact(64).chain(tail.chunks_exact(64)) {
        compress(&mut h, block);
    }
    let mut out = [0u8; 32];
    for (o, w) in out.chunks_exact_mut(4).zip(h) {
        o.copy_from_slice(&w.to_be_bytes());
    }
    out
}

/// One 512-bit block of the SHA-256 hash computation (FIPS 180-4, 6.2.2).
fn compress(h: &mut [u32; 8], block: &[u8]) {
    let mut w = [0u32; 64];
    for (t, word) in block.chunks_exact(4).enumerate() {
        w[t] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
    }
    for t in 16..64 {
        let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
        let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
        w[t] = s1
            .wrapping_add(w[t - 7])
            .wrapping_add(s0)
            .wrapping_add(w[t - 16]);
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;
    for t in 0..64 {
        let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ (!e & g);
        let t1 = hh
            .wrapping_add(big_s1)
            .wrapping_add(ch)
            .wrapping_add(K[t])
            .wrapping_add(w[t]);
        let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let t2 = big_s0.wrapping_add(maj);
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    for (x, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
        *x = x.wrapping_add(v);
    }
}

/// Lowercase hex of `bytes`.
pub(crate) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(DIGITS[usize::from(b >> 4)] as char);
        s.push(DIGITS[usize::from(b & 0x0f)] as char);
    }
    s
}

/// The `model_hash` of `model`: `"sha256:"` plus 64 lowercase hex digits of
/// the SHA-256 of exactly the bytes [`crate::write_unpacked`] writes for
/// `model.json` (its canonical JSON).
pub fn model_hash(model: &Value) -> String {
    format!(
        "sha256:{}",
        hex(&sha256(to_canonical_json(model).as_bytes()))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex256(bytes: &[u8]) -> String {
        hex(&sha256(bytes))
    }

    #[test]
    fn fips_180_4_example_vectors() {
        assert_eq!(
            hex256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            hex256(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn padding_boundaries_give_distinct_digests() {
        let digests: Vec<[u8; 32]> = [55, 56, 63, 64, 65]
            .iter()
            .map(|&n| sha256(&vec![b'a'; n]))
            .collect();
        for i in 0..digests.len() {
            for j in i + 1..digests.len() {
                assert_ne!(digests[i], digests[j]);
            }
        }
    }

    #[test]
    fn golden_model_hash_is_the_sha256_of_model_json() {
        let raw = include_str!("../tests/golden/bracket.sltwx/model.json");
        let model: Value = serde_json::from_str(raw).unwrap();
        assert_eq!(
            model_hash(&model),
            format!("sha256:{}", hex256(raw.as_bytes()))
        );
    }
}
