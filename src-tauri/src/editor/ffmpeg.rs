//! Locating and talking to the bundled ffmpeg.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use crate::error::{Error, Result};

const EXE: &str = if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" };

/// ffmpeg ships next to the app executable (Tauri sidecar). During
/// development, or on other platforms, one on `PATH` is used instead.
pub fn path() -> Option<&'static Path> {
    static PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
    PATH.get_or_init(|| {
        let bundled = std::env::current_exe().ok()?.parent()?.join(EXE);
        if bundled.is_file() {
            return Some(bundled);
        }
        std::env::split_paths(&std::env::var_os("PATH")?)
            .map(|dir| dir.join(EXE))
            .find(|p| p.is_file())
    })
    .as_deref()
}

fn require() -> Result<&'static Path> {
    path().ok_or_else(|| Error::Message("ffmpeg was not found next to ShinDeck".into()))
}

/// A command that never flashes a console window on Windows.
pub fn command() -> Result<Command> {
    let mut cmd = Command::new(require()?);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.stdin(Stdio::null());
    Ok(cmd)
}

/// What the editor needs to know about a source clip.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    /// ShadowPlay can record the microphone on a separate track.
    pub audio_streams: usize,
}

pub fn probe(input: &Path) -> Result<Probe> {
    // `ffmpeg -i` with no output prints the stream summary and exits non-zero;
    // that is expected and avoids shipping ffprobe as well.
    let output = command()?.arg("-hide_banner").arg("-i").arg(input).output()?;
    parse_probe(&String::from_utf8_lossy(&output.stderr))
        .ok_or_else(|| Error::Message("Could not read this clip's video stream".into()))
}

fn parse_timestamp(s: &str) -> Option<f64> {
    let mut parts = s.trim().split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let sec: f64 = parts.next()?.parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + sec)
}

pub(crate) fn parse_probe(text: &str) -> Option<Probe> {
    let mut duration = None;
    let mut video = None;
    let mut audio_streams = 0;
    for line in text.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("Duration:") {
            duration = parse_timestamp(rest.split(',').next()?);
        } else if line.starts_with("Stream #") && line.contains(": Video:") && video.is_none() {
            let (mut size, mut fps) = (None, 30.0);
            let tokens: Vec<&str> = line.split([' ', ',']).filter(|t| !t.is_empty()).collect();
            for (i, token) in tokens.iter().enumerate() {
                if size.is_none() {
                    if let Some((w, h)) = token.split_once('x') {
                        if let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) {
                            if w >= 16 && h >= 16 {
                                size = Some((w, h));
                            }
                        }
                    }
                }
                if *token == "fps" && i > 0 {
                    fps = tokens[i - 1].parse().unwrap_or(30.0);
                }
            }
            video = size.map(|s| (s, fps));
        } else if line.starts_with("Stream #") && line.contains(": Audio:") {
            audio_streams += 1;
        }
    }
    let ((width, height), fps) = video?;
    Some(Probe { duration: duration?, width, height, fps, audio_streams })
}

/// NVENC is available on practically every ShadowPlay user's machine and is
/// many times faster than software encoding; fall back to x264 otherwise.
pub fn has_nvenc() -> bool {
    static NVENC: OnceLock<bool> = OnceLock::new();
    *NVENC.get_or_init(|| {
        let Ok(mut cmd) = command() else { return false };
        cmd.args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i"])
            .arg("color=black:s=256x144:d=0.2")
            .args(["-c:v", "h264_nvenc", "-f", "null", "-"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHADOWPLAY: &str = r#"
Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'clip.mp4':
  Metadata:
    major_brand     : mp42
  Duration: 00:01:15.03, start: 0.000000, bitrate: 15342 kb/s
  Stream #0:0[0x1](und): Video: h264 (High) (avc1 / 0x31637661), yuv420p(tv, bt709, progressive), 2560x1440 [SAR 1:1 DAR 16:9], 15020 kb/s, 59.94 fps, 60 tbr, 90k tbn (default)
  Stream #0:1[0x2](und): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, stereo, fltp, 192 kb/s (default)
  Stream #0:2[0x3](und): Audio: aac (LC) (mp4a / 0x6134706D), 48000 Hz, mono, fltp, 96 kb/s
At least one output file must be specified
"#;

    #[test]
    fn parses_ffmpeg_stream_summary() {
        let probe = parse_probe(SHADOWPLAY).unwrap();
        assert_eq!(
            probe,
            Probe { duration: 75.03, width: 2560, height: 1440, fps: 59.94, audio_streams: 2 }
        );
    }
}
