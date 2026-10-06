//! Turns an edit request into ffmpeg arguments. Pure, so it is unit tested.
//!
//! Every edit re-encodes: cuts are frame-accurate (stream copy can only cut
//! on keyframes, which ShadowPlay places seconds apart) and the output is
//! always H.264 + AAC MP4, which Discord, WhatsApp and Instagram all play.

use std::path::Path;

use serde::Deserialize;

use super::ffmpeg::Probe;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditSpec {
    /// Time ranges (seconds) to keep, in order. `None` keeps the whole clip.
    pub keep: Option<Vec<(f64, f64)>>,
    /// Compress so the file fits in this many bytes.
    pub target_bytes: Option<u64>,
    /// Short tag for the new file name, e.g. "trimmed" or "Discord".
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoder {
    Nvenc,
    X264,
}

#[derive(Debug)]
pub struct Plan {
    pub args: Vec<String>,
    /// Length of the result, for progress reporting.
    pub duration: f64,
}

/// Ignore slivers shorter than a frame or two.
const MIN_SEGMENT: f64 = 0.05;
/// Room for the MP4 container and bitrate overshoot.
const SIZE_MARGIN: f64 = 0.94;
const MIN_VIDEO_KBPS: f64 = 100.0;

/// Clamps, sorts and merges the ranges to keep.
pub fn normalize_segments(keep: Option<&[(f64, f64)]>, duration: f64) -> Vec<(f64, f64)> {
    let mut ranges: Vec<(f64, f64)> = match keep {
        None => vec![(0.0, duration)],
        Some(ranges) => ranges
            .iter()
            .map(|&(a, b)| (a.clamp(0.0, duration), b.clamp(0.0, duration)))
            .filter(|(a, b)| b - a >= MIN_SEGMENT)
            .collect(),
    };
    ranges.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::with_capacity(ranges.len());
    for (a, b) in ranges {
        match merged.last_mut() {
            Some(last) if a <= last.1 + MIN_SEGMENT => last.1 = last.1.max(b),
            _ => merged.push((a, b)),
        }
    }
    merged
}

/// Picks bitrates (kbps) and output height for a size target.
fn size_budget(target_bytes: u64, duration: f64, probe: &Probe) -> Result<(f64, f64, u32, bool)> {
    let total_kbps = target_bytes as f64 * 8.0 / 1000.0 * SIZE_MARGIN / duration;
    let audio_kbps = match (probe.audio_streams, total_kbps) {
        (0, _) => 0.0,
        (_, t) if t > 1500.0 => 128.0,
        (_, t) if t > 600.0 => 96.0,
        _ => 64.0,
    };
    let video_kbps = total_kbps - audio_kbps;
    if video_kbps < MIN_VIDEO_KBPS {
        return Err(Error::Message(
            "That size is too small for a clip this long. Trim it first or pick a larger size."
                .into(),
        ));
    }
    // Fewer pixels look far better than a starved bitrate.
    let height = match video_kbps {
        v if v >= 4500.0 => 1080,
        v if v >= 2000.0 => 720,
        v if v >= 900.0 => 540,
        _ => 360,
    };
    let cap_fps = video_kbps < 1500.0 && probe.fps > 31.0;
    Ok((video_kbps.floor(), audio_kbps, height.min(probe.height), cap_fps))
}

/// Near-transparent quality when there is no size target.
fn quality_video_args(encoder: Encoder) -> Vec<String> {
    let args = match encoder {
        Encoder::Nvenc => "-c:v h264_nvenc -preset p5 -tune hq -rc vbr -cq 19 -b:v 0",
        Encoder::X264 => "-c:v libx264 -preset veryfast -crf 18",
    };
    args.split(' ').map(String::from).collect()
}

const OUTPUT_ARGS: [&str; 7] =
    ["-movflags", "+faststart", "-f", "mp4", "-progress", "pipe:1", "-nostats"];

fn secs(t: f64) -> String {
    format!("{t:.3}")
}

/// `bitrate_scale` < 1 tightens the budget for a retry after an overshoot.
pub fn build(
    input: &Path,
    output: &Path,
    probe: &Probe,
    spec: &EditSpec,
    encoder: Encoder,
    bitrate_scale: f64,
) -> Result<Plan> {
    let segments = normalize_segments(spec.keep.as_deref(), probe.duration);
    let duration: f64 = segments.iter().map(|(a, b)| b - a).sum();
    if segments.is_empty() || duration < 0.25 {
        return Err(Error::Message("Nothing left to save: keep at least a moment of video".into()));
    }

    let mut args: Vec<String> =
        ["-hide_banner", "-loglevel", "error", "-y"].map(String::from).to_vec();
    for &(a, b) in &segments {
        args.extend(["-ss".into(), secs(a), "-t".into(), secs(b - a), "-i".into()]);
        args.push(input.to_string_lossy().into_owned());
    }

    // --- filter graph: (mix audio tracks) -> concat -> scale/fps ------------
    let has_audio = probe.audio_streams > 0;
    let mut graph = Vec::new();
    let mut concat_inputs = String::new();
    for i in 0..segments.len() {
        concat_inputs.push_str(&format!("[{i}:v:0]"));
        if probe.audio_streams >= 2 {
            // Game and microphone tracks become one, as players expect.
            graph.push(format!("[{i}:a:0][{i}:a:1]amix=inputs=2:duration=first:normalize=0[a{i}]"));
            concat_inputs.push_str(&format!("[a{i}]"));
        } else if has_audio {
            concat_inputs.push_str(&format!("[{i}:a:0]"));
        }
    }
    graph.push(format!(
        "{concat_inputs}concat=n={}:v=1:a={}[vc]{}",
        segments.len(),
        u8::from(has_audio),
        if has_audio { "[ac]" } else { "" }
    ));

    let budget = match spec.target_bytes {
        Some(bytes) => Some(size_budget((bytes as f64 * bitrate_scale) as u64, duration, probe)?),
        None => None,
    };
    let mut post = Vec::new();
    if let Some((_, _, height, cap_fps)) = budget {
        if height < probe.height {
            post.push(format!("scale=-2:{height}:flags=lanczos"));
        }
        if cap_fps {
            post.push("fps=30".to_owned());
        }
    }
    // Also converts 10-bit HEVC sources to something every player decodes.
    post.push("format=yuv420p".to_owned());
    graph.push(format!("[vc]{}[v]", post.join(",")));

    args.extend(["-filter_complex".into(), graph.join(";"), "-map".into(), "[v]".into()]);
    if has_audio {
        args.extend(["-map".into(), "[ac]".into()]);
    }

    // --- encoders -----------------------------------------------------------
    let video: Vec<String> = match (encoder, budget) {
        (_, None) => quality_video_args(encoder),
        (Encoder::Nvenc, Some((v, ..))) => vec![
            "-c:v".into(),
            "h264_nvenc".into(),
            "-preset".into(),
            "p5".into(),
            "-tune".into(),
            "hq".into(),
            "-rc".into(),
            "cbr".into(),
            "-multipass".into(),
            "qres".into(),
            "-b:v".into(),
            format!("{v}k"),
            "-bufsize".into(),
            format!("{}k", v * 2.0),
        ],
        (Encoder::X264, Some((v, ..))) => vec![
            "-c:v".into(),
            "libx264".into(),
            "-preset".into(),
            "medium".into(),
            "-b:v".into(),
            format!("{v}k"),
            "-maxrate".into(),
            format!("{v}k"),
            "-bufsize".into(),
            format!("{}k", v * 2.0),
        ],
    };
    args.extend(video);
    if has_audio {
        let audio_kbps = budget.map_or(192.0, |(_, a, ..)| a);
        args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), format!("{audio_kbps}k")]);
    }
    args.extend(OUTPUT_ARGS.map(String::from));
    args.push(output.to_string_lossy().into_owned());
    Ok(Plan { args, duration })
}

/// Joins whole clips end to end, in order. Clips may differ in resolution,
/// frame rate and audio tracks: every part is fitted (letterboxed if needed)
/// to the first clip's frame at the highest frame rate, and a clip without
/// audio gets silence so the soundtrack stays in sync.
pub fn build_merge(inputs: &[(&Path, &Probe)], output: &Path, encoder: Encoder) -> Result<Plan> {
    if inputs.len() < 2 {
        return Err(Error::Message("Pick at least two clips to merge".into()));
    }
    let duration: f64 = inputs.iter().map(|(_, p)| p.duration).sum();
    let (_, first) = inputs[0];
    // Encoders need even dimensions.
    let (w, h) = (first.width & !1, first.height & !1);
    let fps = inputs.iter().map(|(_, p)| p.fps).fold(0.0, f64::max).clamp(1.0, 240.0);
    let has_audio = inputs.iter().any(|(_, p)| p.audio_streams > 0);

    let mut args: Vec<String> =
        ["-hide_banner", "-loglevel", "error", "-y"].map(String::from).to_vec();
    for (path, _) in inputs {
        args.extend(["-i".into(), path.to_string_lossy().into_owned()]);
    }

    let mut graph = Vec::new();
    let mut concat_inputs = String::new();
    let mut silence_input = inputs.len();
    for (i, (_, probe)) in inputs.iter().enumerate() {
        graph.push(format!(
            "[{i}:v:0]scale={w}:{h}:force_original_aspect_ratio=decrease,\
             pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps={fps},format=yuv420p[v{i}]"
        ));
        concat_inputs.push_str(&format!("[v{i}]"));
        if !has_audio {
            continue;
        }
        let source = match probe.audio_streams {
            0 => {
                args.extend(["-f".into(), "lavfi".into(), "-t".into(), secs(probe.duration)]);
                args.extend(["-i".into(), "anullsrc=r=48000:cl=stereo".into()]);
                silence_input += 1;
                format!("[{}:a:0]", silence_input - 1)
            }
            1 => format!("[{i}:a:0]"),
            // Game and microphone tracks become one, as players expect.
            _ => format!("[{i}:a:0][{i}:a:1]amix=inputs=2:duration=first:normalize=0,"),
        };
        let source = if source.ends_with(',') { source } else { format!("{source}anull,") };
        // Same format for every part, padded/cut to the video's length.
        let d = secs(probe.duration);
        graph.push(format!(
            "{source}aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,\
             apad=whole_dur={d},atrim=end={d}[a{i}]"
        ));
        concat_inputs.push_str(&format!("[a{i}]"));
    }
    graph.push(format!(
        "{concat_inputs}concat=n={}:v=1:a={}[v]{}",
        inputs.len(),
        u8::from(has_audio),
        if has_audio { "[a]" } else { "" }
    ));

    args.extend(["-filter_complex".into(), graph.join(";"), "-map".into(), "[v]".into()]);
    if has_audio {
        args.extend(["-map".into(), "[a]".into()]);
    }
    args.extend(quality_video_args(encoder));
    if has_audio {
        args.extend(["-c:a", "aac", "-b:a", "192k"].map(String::from));
    }
    args.extend(OUTPUT_ARGS.map(String::from));
    args.push(output.to_string_lossy().into_owned());
    Ok(Plan { args, duration })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn probe(audio_streams: usize) -> Probe {
        Probe { duration: 60.0, width: 2560, height: 1440, fps: 60.0, audio_streams }
    }

    fn spec(keep: Option<Vec<(f64, f64)>>, target_bytes: Option<u64>) -> EditSpec {
        EditSpec { keep, target_bytes, label: "x".into() }
    }

    fn arg_after<'a>(args: &'a [String], flag: &str) -> &'a str {
        let i = args.iter().position(|a| a == flag).unwrap();
        &args[i + 1]
    }

    #[test]
    fn merges_and_clamps_segments() {
        let keep = [(50.0, 70.0), (-3.0, 10.0), (9.0, 20.0), (30.0, 30.01)];
        assert_eq!(normalize_segments(Some(&keep), 60.0), vec![(0.0, 20.0), (50.0, 60.0)]);
        assert_eq!(normalize_segments(None, 60.0), vec![(0.0, 60.0)]);
    }

    #[test]
    fn trim_and_cut_uses_one_input_per_kept_range() {
        let s = spec(Some(vec![(5.0, 20.0), (30.0, 55.0)]), None);
        let plan =
            build(Path::new("in.mp4"), Path::new("out.mp4"), &probe(1), &s, Encoder::X264, 1.0)
                .unwrap();
        assert_eq!(plan.duration, 40.0);
        assert_eq!(plan.args.iter().filter(|a| *a == "-i").count(), 2);
        assert_eq!(
            arg_after(&plan.args, "-filter_complex"),
            "[0:v:0][0:a:0][1:v:0][1:a:0]concat=n=2:v=1:a=1[vc][ac];[vc]format=yuv420p[v]"
        );
        assert_eq!(arg_after(&plan.args, "-crf"), "18");
    }

    #[test]
    fn mixes_microphone_track() {
        let s = spec(None, None);
        let plan =
            build(Path::new("in.mp4"), Path::new("o.mp4"), &probe(2), &s, Encoder::Nvenc, 1.0)
                .unwrap();
        assert!(arg_after(&plan.args, "-filter_complex").starts_with("[0:a:0][0:a:1]amix"));
    }

    #[test]
    fn compression_fits_the_budget_and_downscales() {
        // 10 MB for 60 s: ~1253 kbps total -> 96k audio, 540p, 30 fps.
        let s = spec(None, Some(10_000_000));
        let plan =
            build(Path::new("in.mp4"), Path::new("o.mp4"), &probe(1), &s, Encoder::X264, 1.0)
                .unwrap();
        let video: f64 = arg_after(&plan.args, "-b:v").trim_end_matches('k').parse().unwrap();
        let audio: f64 = arg_after(&plan.args, "-b:a").trim_end_matches('k').parse().unwrap();
        assert_eq!(audio, 96.0);
        assert!((video + audio) * 60.0 / 8.0 * 1000.0 <= 10_000_000.0);
        assert!(arg_after(&plan.args, "-filter_complex").contains("scale=-2:540"));
        assert!(arg_after(&plan.args, "-filter_complex").contains("fps=30"));
    }

    #[test]
    fn rejects_impossible_targets() {
        let s = spec(None, Some(200_000));
        assert!(build(Path::new("i"), Path::new("o"), &probe(1), &s, Encoder::X264, 1.0).is_err());
    }

    #[test]
    fn merge_fits_every_part_to_the_first_clip() {
        let a = probe(2);
        let b = Probe { duration: 30.0, width: 1920, height: 1080, fps: 30.0, audio_streams: 0 };
        let plan = build_merge(
            &[(Path::new("a.mp4"), &a), (Path::new("b.mp4"), &b)],
            Path::new("o.mp4"),
            Encoder::X264,
        )
        .unwrap();
        assert_eq!(plan.duration, 90.0);
        // Two clips plus generated silence for the one without audio.
        assert_eq!(plan.args.iter().filter(|a| *a == "-i").count(), 3);
        assert!(plan.args.contains(&"anullsrc=r=48000:cl=stereo".to_string()));
        let graph = arg_after(&plan.args, "-filter_complex");
        assert!(graph.contains("[1:v:0]scale=2560:1440:force_original_aspect_ratio=decrease"));
        assert!(graph.contains("fps=60"));
        assert!(graph.contains("[0:a:0][0:a:1]amix"));
        assert!(graph.contains("[2:a:0]anull,"));
        assert!(graph.ends_with("[v0][a0][v1][a1]concat=n=2:v=1:a=1[v][a]"));
    }

    #[test]
    fn merge_without_any_audio() {
        let a = probe(0);
        let plan = build_merge(
            &[(Path::new("a"), &a), (Path::new("b"), &a)],
            Path::new("o"),
            Encoder::Nvenc,
        )
        .unwrap();
        assert!(!plan.args.contains(&"-c:a".to_string()));
        assert!(arg_after(&plan.args, "-filter_complex").ends_with("concat=n=2:v=1:a=0[v]"));
        assert!(build_merge(&[(Path::new("a"), &a)], Path::new("o"), Encoder::X264).is_err());
    }

    #[test]
    fn handles_clips_without_audio() {
        let s = spec(None, None);
        let plan =
            build(Path::new("i"), Path::new("o"), &probe(0), &s, Encoder::X264, 1.0).unwrap();
        assert!(!plan.args.contains(&"-c:a".to_string()));
        assert!(arg_after(&plan.args, "-filter_complex").contains("concat=n=1:v=1:a=0[vc];"));
    }
}
