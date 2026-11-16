//! A minimal ZIP reader and writer for the `.sltwx` container
//! (`docs/specs/FILE_FORMAT.md`, "Container").
//!
//! Source: PKWARE, *ZIP File Format Specification* (APPNOTE.TXT) version
//! 6.3.x, section 4.3.7 (local file header), 4.3.12 (central directory file
//! header) and 4.3.16 (end of central directory record). CRC-32 is the
//! IEEE 802.3 polynomial (`crc32fast`); deflate is RFC 1951 (`flate2`).
//!
//! Supported: methods 0 (stored) and 8 (deflate), CRC-32, UTF-8 names
//! (general purpose bit 11). Not supported: ZIP64, encryption, multi-disk.

use std::io::{Read, Write};

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;

use crate::FormatError;

const LOCAL_SIG: u32 = 0x0403_4b50;
const CENTRAL_SIG: u32 = 0x0201_4b50;
const EOCD_SIG: u32 = 0x0605_4b50;
const LOCAL_LEN: usize = 30;
const CENTRAL_LEN: usize = 46;
const EOCD_LEN: usize = 22;
/// General purpose bit 0: the entry is encrypted.
const FLAG_ENCRYPTED: u16 = 1;
/// General purpose bit 11: the entry name is UTF-8.
const FLAG_UTF8: u16 = 1 << 11;
const METHOD_STORED: u16 = 0;
const METHOD_DEFLATE: u16 = 8;
/// DOS date 1980-01-01: `(year - 1980) << 9 | month << 5 | day`. The DOS
/// time is 0 (00:00:00), so written archives are reproducible.
const DOS_DATE: u16 = (1 << 5) | 1;

fn bad(why: impl Into<String>) -> FormatError {
    FormatError::MalformedArchive(why.into())
}

/// Builds a ZIP archive holding `entries` (name, data), in the given order,
/// each deflated, with fixed timestamps.
pub(crate) fn write_zip(entries: &[(&str, &[u8])]) -> Result<Vec<u8>, FormatError> {
    struct Written {
        name: Vec<u8>,
        crc: u32,
        csize: u32,
        usize: u32,
        offset: u32,
    }
    let too_big = || bad("archive too large (ZIP64 is not supported)");
    let mut out: Vec<u8> = Vec::new();
    let mut written: Vec<Written> = Vec::new();
    for (name, data) in entries {
        let offset = u32::try_from(out.len()).map_err(|_| too_big())?;
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
        enc.write_all(data).map_err(|e| bad(e.to_string()))?;
        let packed = enc.finish().map_err(|e| bad(e.to_string()))?;
        let w = Written {
            name: name.as_bytes().to_vec(),
            crc: crc32fast::hash(data),
            csize: u32::try_from(packed.len()).map_err(|_| too_big())?,
            usize: u32::try_from(data.len()).map_err(|_| too_big())?,
            offset,
        };
        let name_len = u16::try_from(w.name.len()).map_err(|_| bad("entry name too long"))?;
        put_u32(&mut out, LOCAL_SIG);
        put_u16(&mut out, 20); // version needed to extract
        put_u16(&mut out, FLAG_UTF8);
        put_u16(&mut out, METHOD_DEFLATE);
        put_u16(&mut out, 0); // DOS time
        put_u16(&mut out, DOS_DATE);
        put_u32(&mut out, w.crc);
        put_u32(&mut out, w.csize);
        put_u32(&mut out, w.usize);
        put_u16(&mut out, name_len);
        put_u16(&mut out, 0); // extra field length
        out.extend_from_slice(&w.name);
        out.extend_from_slice(&packed);
        written.push(w);
    }
    let cd_start = u32::try_from(out.len()).map_err(|_| too_big())?;
    for w in &written {
        put_u32(&mut out, CENTRAL_SIG);
        put_u16(&mut out, 20); // version made by
        put_u16(&mut out, 20); // version needed to extract
        put_u16(&mut out, FLAG_UTF8);
        put_u16(&mut out, METHOD_DEFLATE);
        put_u16(&mut out, 0);
        put_u16(&mut out, DOS_DATE);
        put_u32(&mut out, w.crc);
        put_u32(&mut out, w.csize);
        put_u32(&mut out, w.usize);
        put_u16(&mut out, w.name.len() as u16); // checked above
        put_u16(&mut out, 0); // extra field length
        put_u16(&mut out, 0); // comment length
        put_u16(&mut out, 0); // disk number start
        put_u16(&mut out, 0); // internal attributes
        put_u32(&mut out, 0); // external attributes
        put_u32(&mut out, w.offset);
        out.extend_from_slice(&w.name);
    }
    let cd_size = u32::try_from(out.len())
        .map_err(|_| too_big())?
        .wrapping_sub(cd_start);
    let count = u16::try_from(written.len()).map_err(|_| too_big())?;
    put_u32(&mut out, EOCD_SIG);
    put_u16(&mut out, 0); // this disk
    put_u16(&mut out, 0); // disk with the central directory
    put_u16(&mut out, count);
    put_u16(&mut out, count);
    put_u32(&mut out, cd_size);
    put_u32(&mut out, cd_start);
    put_u16(&mut out, 0); // comment length
    Ok(out)
}

/// One entry of an archive's central directory.
pub(crate) struct Entry {
    name: String,
    flags: u16,
    method: u16,
    crc: u32,
    csize: usize,
    usize: usize,
    offset: usize,
}

impl Entry {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

/// Reads the central directory of the archive `bytes`.
///
/// Only the directory is checked here; an entry's data is checked when it
/// is extracted with [`extract`].
pub(crate) fn list(bytes: &[u8]) -> Result<Vec<Entry>, FormatError> {
    let eocd = find_eocd(bytes).ok_or_else(|| bad("no end-of-central-directory record"))?;
    let disk = u16_at(bytes, eocd + 4)?;
    let cd_disk = u16_at(bytes, eocd + 6)?;
    let on_disk = u16_at(bytes, eocd + 8)?;
    let total = u16_at(bytes, eocd + 10)?;
    let cd_size = u32_at(bytes, eocd + 12)?;
    let cd_offset = u32_at(bytes, eocd + 16)?;
    if disk != 0 || cd_disk != 0 || on_disk != total {
        return Err(bad("multi-disk archives are not supported"));
    }
    if total == u16::MAX || cd_size == u32::MAX || cd_offset == u32::MAX {
        return Err(bad("ZIP64 archives are not supported"));
    }
    let cd_end = (cd_offset as usize)
        .checked_add(cd_size as usize)
        .filter(|end| *end <= eocd)
        .ok_or_else(|| bad("central directory lies outside the file"))?;
    let mut pos = cd_offset as usize;
    let mut entries = Vec::with_capacity(total as usize);
    for _ in 0..total {
        if pos + CENTRAL_LEN > cd_end || u32_at(bytes, pos)? != CENTRAL_SIG {
            return Err(bad("bad central directory entry"));
        }
        let flags = u16_at(bytes, pos + 8)?;
        let method = u16_at(bytes, pos + 10)?;
        let crc = u32_at(bytes, pos + 16)?;
        let csize = u32_at(bytes, pos + 20)? as usize;
        let usize = u32_at(bytes, pos + 24)? as usize;
        let name_len = u16_at(bytes, pos + 28)? as usize;
        let extra_len = u16_at(bytes, pos + 30)? as usize;
        let comment_len = u16_at(bytes, pos + 32)? as usize;
        let offset = u32_at(bytes, pos + 42)? as usize;
        let name_start = pos + CENTRAL_LEN;
        let next = name_start + name_len + extra_len + comment_len;
        if next > cd_end {
            return Err(bad("central directory entry is truncated"));
        }
        let name = String::from_utf8_lossy(&bytes[name_start..name_start + name_len]).into_owned();
        entries.push(Entry {
            name,
            flags,
            method,
            crc,
            csize,
            usize,
            offset,
        });
        pos = next;
    }
    Ok(entries)
}

/// Returns the uncompressed data of `entry`, checking its size and CRC-32.
pub(crate) fn extract(bytes: &[u8], entry: &Entry) -> Result<Vec<u8>, FormatError> {
    let name = &entry.name;
    if entry.flags & FLAG_ENCRYPTED != 0 {
        return Err(bad(format!("entry `{name}` is encrypted")));
    }
    let h = entry.offset;
    if !h
        .checked_add(LOCAL_LEN)
        .is_some_and(|end| end <= bytes.len())
        || u32_at(bytes, h)? != LOCAL_SIG
    {
        return Err(bad(format!("entry `{name}` has a bad local header")));
    }
    let name_len = u16_at(bytes, h + 26)? as usize;
    let extra_len = u16_at(bytes, h + 28)? as usize;
    let start = h + LOCAL_LEN + name_len + extra_len;
    let data = start
        .checked_add(entry.csize)
        .and_then(|end| bytes.get(start..end))
        .ok_or_else(|| bad(format!("entry `{name}` is truncated")))?;
    let plain = match entry.method {
        METHOD_STORED => {
            if entry.csize != entry.usize {
                return Err(bad(format!("stored entry `{name}` has inconsistent sizes")));
            }
            data.to_vec()
        }
        METHOD_DEFLATE => {
            let mut plain = Vec::new();
            // Read at most one byte past the declared size, so a lying
            // header cannot make us allocate without bound.
            DeflateDecoder::new(data)
                .take(entry.usize as u64 + 1)
                .read_to_end(&mut plain)
                .map_err(|e| bad(format!("entry `{name}` has corrupt deflate data: {e}")))?;
            plain
        }
        method => {
            return Err(FormatError::UnsupportedMethod {
                entry: name.clone(),
                method,
            })
        }
    };
    if plain.len() != entry.usize {
        return Err(bad(format!(
            "entry `{name}` is {} bytes, its header says {}",
            plain.len(),
            entry.usize
        )));
    }
    if crc32fast::hash(&plain) != entry.crc {
        return Err(FormatError::CrcMismatch(name.clone()));
    }
    Ok(plain)
}

/// The offset of the end-of-central-directory record: the last occurrence
/// of its signature in the final `22 + 65535` bytes (the record may be
/// followed by a comment of up to 65535 bytes).
fn find_eocd(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < EOCD_LEN {
        return None;
    }
    let last = bytes.len() - EOCD_LEN;
    let first = last.saturating_sub(usize::from(u16::MAX));
    (first..=last)
        .rev()
        .find(|&i| bytes[i..i + 4] == EOCD_SIG.to_le_bytes())
}

fn u16_at(bytes: &[u8], at: usize) -> Result<u16, FormatError> {
    at.checked_add(2)
        .and_then(|end| bytes.get(at..end))
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| bad("unexpected end of file"))
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, FormatError> {
    at.checked_add(4)
        .and_then(|end| bytes.get(at..end))
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| bad("unexpected end of file"))
}

fn put_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
