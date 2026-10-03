//! The film-roll strip shown while trimming: evenly spaced frames rendered
//! side by side into one cached JPEG, in a single ffmpeg run.

use std::fs;
use std::path::PathBuf;
use std::process::Stdio;

use super::ffmpeg;
use crate::library::IndexEntry;
use crate::media::stable_hash;

const FRAMES: usize = 16;
const FRAME_W: u32 = 192;
const FRAME_H: u32 = 108;

pub struct Filmstrips {
    dir: PathBuf,
}

impl Filmstrips {
    pub fn new(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    /// Blocking: may run ffmpeg for a second or two on first request.
    pub fn get(&self, entry: &IndexEntry) -> Option<Vec<u8>> {
        let hash = stable_hash(&[
            entry.path.as_os_str().as_encoded_bytes(),
            &entry.size.to_le_bytes(),
            &entry.modified.to_le_bytes(),
        ]);
        let cache = self.dir.join(format!("{hash:016x}.jpg"));
        if let Ok(bytes) = fs::read(&cache) {
            return Some(bytes);
        }

        let duration = ffmpeg::probe(&entry.path).ok()?.duration;
        let mut cmd = ffmpeg::command().ok()?;
        cmd.args(["-hide_banner", "-loglevel", "error", "-y"]);
        // One fast keyframe seek per frame instead of decoding the whole clip.
        let mut graph = Vec::new();
        let mut stack = String::new();
        for i in 0..FRAMES {
            let t = ((i as f64 + 0.5) * duration / FRAMES as f64).min((duration - 0.1).max(0.0));
            cmd.args(["-ss", &format!("{t:.3}"), "-i"]).arg(&entry.path);
            graph.push(format!(
                "[{i}:v:0]scale={FRAME_W}:{FRAME_H}:force_original_aspect_ratio=increase,\
                 crop={FRAME_W}:{FRAME_H},setsar=1,format=yuvj420p[f{i}]"
            ));
            stack.push_str(&format!("[f{i}]"));
        }
        graph.push(format!("{stack}hstack=inputs={FRAMES}[strip]"));
        cmd.arg("-filter_complex")
            .arg(graph.join(";"))
            .args(["-map", "[strip]", "-frames:v", "1", "-q:v", "5", "-f", "mjpeg"])
            .arg(&cache)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let ok = cmd.status().is_ok_and(|s| s.success());
        if !ok {
            let _ = fs::remove_file(&cache);
            return None;
        }
        fs::read(&cache).ok()
    }
}
