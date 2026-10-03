use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use image::codecs::jpeg::JpegEncoder;
use image::imageops::{self, FilterType};
use image::RgbImage;

use super::{stable_hash, Worker};
use crate::library::IndexEntry;

/// Wide enough to stay crisp on large cards at 150% display scaling.
const THUMB_WIDTH: u32 = 512;
const JPEG_QUALITY: u8 = 82;

/// Disk-cached video thumbnails.
pub struct Thumbnails {
    dir: PathBuf,
    worker: Worker,
}

impl Thumbnails {
    pub fn new(dir: PathBuf, worker: Worker) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir, worker }
    }

    /// The cache key includes size and mtime, so a replaced or re-encoded
    /// clip automatically gets a fresh thumbnail.
    fn cache_path(&self, entry: &IndexEntry) -> PathBuf {
        let hash = stable_hash(&[
            entry.path.as_os_str().as_encoded_bytes(),
            &entry.size.to_le_bytes(),
            &entry.modified.to_le_bytes(),
        ]);
        self.dir.join(format!("{hash:016x}.jpg"))
    }

    /// Calls `done` with JPEG bytes, or `None` if no thumbnail can be made.
    /// Must be called off the UI thread (it may read from disk).
    pub fn get(&self, entry: IndexEntry, done: impl FnOnce(Option<Vec<u8>>) + Send + 'static) {
        let cache = self.cache_path(&entry);
        if let Ok(bytes) = fs::read(&cache) {
            return done(Some(bytes));
        }
        self.worker.spawn(move || {
            let bytes = crate::platform::video_thumbnail(&entry.path, THUMB_WIDTH).and_then(encode);
            if let Some(bytes) = &bytes {
                let _ = fs::write(&cache, bytes);
            }
            done(bytes)
        });
    }
}

fn encode(mut image: RgbImage) -> Option<Vec<u8>> {
    if image.width() > THUMB_WIDTH {
        let height = (image.height() as u64 * THUMB_WIDTH as u64 / image.width() as u64) as u32;
        image = imageops::resize(&image, THUMB_WIDTH, height.max(1), FilterType::Triangle);
    }
    let mut out = Cursor::new(Vec::with_capacity(48 * 1024));
    JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY).encode_image(&image).ok()?;
    Some(out.into_inner())
}
