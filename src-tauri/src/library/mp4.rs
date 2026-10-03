//! Minimal ISO-BMFF reader that pulls the duration out of `moov/mvhd`.
//!
//! Reading a few box headers is orders of magnitude cheaper than asking a
//! media framework, and works the same whether the `moov` box sits at the
//! start of the file or at the end (as with ShadowPlay recordings).

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

pub fn duration_ms(path: &Path) -> Option<u64> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if !matches!(ext.as_str(), "mp4" | "m4v" | "mov") {
        return None;
    }
    let mut file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    read_duration(&mut file, len).ok().flatten()
}

struct BoxHeader {
    kind: [u8; 4],
    /// Offset of the box payload.
    body: u64,
    /// Offset just past the end of the box.
    end: u64,
}

fn read_header<R: Read + Seek>(r: &mut R, at: u64, limit: u64) -> io::Result<Option<BoxHeader>> {
    if at + 8 > limit {
        return Ok(None);
    }
    r.seek(SeekFrom::Start(at))?;
    let mut head = [0u8; 8];
    r.read_exact(&mut head)?;
    let size32 = u32::from_be_bytes(head[0..4].try_into().unwrap()) as u64;
    let kind = head[4..8].try_into().unwrap();
    let (size, header_len) = match size32 {
        0 => (limit - at, 8),
        1 => {
            let mut large = [0u8; 8];
            r.read_exact(&mut large)?;
            (u64::from_be_bytes(large), 16)
        }
        n => (n, 8),
    };
    if size < header_len || at + size > limit {
        return Ok(None);
    }
    Ok(Some(BoxHeader { kind, body: at + header_len, end: at + size }))
}

fn find_child<R: Read + Seek>(
    r: &mut R,
    mut at: u64,
    limit: u64,
    kind: &[u8; 4],
) -> io::Result<Option<BoxHeader>> {
    while let Some(header) = read_header(r, at, limit)? {
        if &header.kind == kind {
            return Ok(Some(header));
        }
        at = header.end;
    }
    Ok(None)
}

fn read_duration<R: Read + Seek>(r: &mut R, len: u64) -> io::Result<Option<u64>> {
    let Some(moov) = find_child(r, 0, len, b"moov")? else { return Ok(None) };
    let Some(mvhd) = find_child(r, moov.body, moov.end, b"mvhd")? else { return Ok(None) };

    r.seek(SeekFrom::Start(mvhd.body))?;
    let mut version = [0u8; 4];
    r.read_exact(&mut version)?;
    let (timescale, duration) = if version[0] == 1 {
        // creation(8) modification(8) timescale(4) duration(8)
        let mut b = [0u8; 28];
        r.read_exact(&mut b)?;
        (
            u32::from_be_bytes(b[16..20].try_into().unwrap()) as u64,
            u64::from_be_bytes(b[20..28].try_into().unwrap()),
        )
    } else {
        // creation(4) modification(4) timescale(4) duration(4)
        let mut b = [0u8; 16];
        r.read_exact(&mut b)?;
        (
            u32::from_be_bytes(b[8..12].try_into().unwrap()) as u64,
            u32::from_be_bytes(b[12..16].try_into().unwrap()) as u64,
        )
    };
    if timescale == 0 || duration == 0 || duration == u32::MAX as u64 {
        return Ok(None);
    }
    Ok(Some(duration.saturating_mul(1000) / timescale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn boxed(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut v = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        v.extend_from_slice(kind);
        v.extend_from_slice(body);
        v
    }

    fn mvhd_v0(timescale: u32, duration: u32) -> Vec<u8> {
        let mut body = vec![0u8; 4 + 8];
        body.extend_from_slice(&timescale.to_be_bytes());
        body.extend_from_slice(&duration.to_be_bytes());
        body.extend_from_slice(&[0u8; 80]);
        boxed(b"mvhd", &body)
    }

    #[test]
    fn reads_duration_when_moov_is_last() {
        let mut file = boxed(b"ftyp", b"isom0000");
        file.extend(boxed(b"mdat", &[0u8; 64]));
        file.extend(boxed(b"moov", &mvhd_v0(1000, 61_500)));
        let len = file.len() as u64;
        assert_eq!(read_duration(&mut Cursor::new(file), len).unwrap(), Some(61_500));
    }

    #[test]
    fn reads_v1_duration() {
        let mut body = vec![1u8, 0, 0, 0];
        body.extend_from_slice(&[0u8; 16]);
        body.extend_from_slice(&90_000u32.to_be_bytes());
        body.extend_from_slice(&(90_000u64 * 30).to_be_bytes());
        let file = boxed(b"moov", &boxed(b"mvhd", &body));
        let len = file.len() as u64;
        assert_eq!(read_duration(&mut Cursor::new(file), len).unwrap(), Some(30_000));
    }

    #[test]
    fn missing_moov_is_none() {
        let file = boxed(b"mdat", &[0u8; 32]);
        let len = file.len() as u64;
        assert_eq!(read_duration(&mut Cursor::new(file), len).unwrap(), None);
    }
}
