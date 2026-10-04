//! Clip editing: rename, trim & cut, and compression to a size target.

pub mod ffmpeg;
mod files;
mod filmstrip;
pub mod plan;

pub use filmstrip::Filmstrips;

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::library::{self, Clip, Marks};
use crate::state::AppState;
use plan::{EditSpec, Encoder};

/// Where an edit's result goes. `Ask` never reaches the backend: the UI
/// resolves it with the user first.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Destination {
    New,
    Replace,
}

/// Runs one ffmpeg job at a time and lets the UI cancel it.
#[derive(Default)]
pub struct Editor {
    busy: AtomicBool,
    cancelled: AtomicBool,
    child: Mutex<Option<Child>>,
}

impl Editor {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            let _ = child.kill();
        }
    }

    fn run(&self, args: &[String], duration: f64, progress: &dyn Fn(f64)) -> Result<()> {
        let mut child =
            ffmpeg::command()?.args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        let stdout = child.stdout.take().expect("piped");
        let mut stderr = child.stderr.take().expect("piped");
        let errors = thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            text
        });
        *self.child.lock().unwrap() = Some(child);

        // `-progress pipe:1` prints key=value lines; out_time_us is the
        // position in the output.
        for line in BufReader::new(stdout).lines().map_while(|l| l.ok()) {
            let micros = line.strip_prefix("out_time_us=").or(line.strip_prefix("out_time_ms="));
            if let Some(us) = micros.and_then(|v| v.parse::<f64>().ok()) {
                progress((us / 1e6 / duration).clamp(0.0, 1.0));
            }
        }
        let status = self.child.lock().unwrap().take().map(|mut c| c.wait()).transpose()?;
        let errors = errors.join().unwrap_or_default();

        if self.cancelled.load(Ordering::SeqCst) {
            return Err(Error::Message("Cancelled".into()));
        }
        match status {
            Some(s) if s.success() => Ok(()),
            _ => {
                let detail = errors.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("");
                Err(Error::Message(format!("Export failed. {detail}").trim().to_owned()))
            }
        }
    }
}

/// Clears the busy flag however the export ends.
struct BusyGuard<'a>(&'a AtomicBool);

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

fn library_root(state: &AppState) -> Result<PathBuf> {
    state
        .settings
        .lock()
        .unwrap()
        .value
        .library_path
        .clone()
        .ok_or_else(|| Error::Message("No clips folder configured".into()))
}

fn clip_path(state: &AppState, id: &str) -> Result<PathBuf> {
    state
        .index
        .read()
        .unwrap()
        .get(id)
        .map(|e| e.path.clone())
        .ok_or_else(|| Error::Message("Clip not found; it may have been moved or deleted".into()))
}

/// Carries favorite/edited marks to a new id and refreshes the index.
fn commit(state: &AppState, root: &Path, old_id: &str, path: &Path, edited: bool) -> Result<Clip> {
    let new_id = {
        let rel = path.strip_prefix(root).unwrap_or(path);
        rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
    };
    let mut favorites = state.favorites.lock().unwrap();
    let mut marks = state.edited.lock().unwrap();
    if favorites.value.rename(old_id, &new_id) {
        favorites.save()?;
    }
    let mut marks_changed = marks.value.rename(old_id, &new_id);
    if edited {
        marks_changed |= marks.value.set(&new_id, true);
    }
    if marks_changed {
        marks.save()?;
    }

    let mut index = state.index.write().unwrap();
    let clip = library::clip_at(
        root,
        path,
        &Marks { favorites: &favorites.value, edited: &marks.value },
        &index,
    )?;
    if old_id != clip.id {
        index.remove(old_id);
    }
    index.upsert(&clip);
    Ok(clip)
}

pub fn rename(state: &AppState, id: &str, name: &str) -> Result<Clip> {
    let root = library_root(state)?;
    let path = clip_path(state, id)?;
    let name = files::validate_name(name)?;
    let ext = path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    let target = path.with_file_name(format!("{name}.{ext}"));

    if target != path {
        // Allow pure case changes ("clip" -> "Clip") on case-insensitive disks.
        let same_file =
            target.to_string_lossy().to_lowercase() == path.to_string_lossy().to_lowercase();
        if target.exists() && !same_file {
            return Err(Error::Message("Another clip already has that name".into()));
        }
        files::rename(&path, &target)?;
    }
    commit(state, &root, id, &target, false)
}

/// Moves a clip to the Recycle Bin and drops its favorite/edited marks and
/// index entry. A clip that is already gone counts as removed.
pub fn delete(state: &AppState, id: &str) -> Result<()> {
    let path = clip_path(state, id)?;
    if path.exists() {
        crate::platform::move_to_trash(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                Error::Message("The file is in use by another program".into())
            }
            _ => Error::from(e),
        })?;
    }

    let mut favorites = state.favorites.lock().unwrap();
    if favorites.value.set(id, false) {
        favorites.save()?;
    }
    let mut marks = state.edited.lock().unwrap();
    if marks.value.set(id, false) {
        marks.save()?;
    }
    state.index.write().unwrap().remove(id);
    Ok(())
}

pub fn export(
    state: &AppState,
    id: &str,
    spec: &EditSpec,
    destination: Destination,
    progress: &dyn Fn(f64),
) -> Result<Clip> {
    let editor = &state.editor;
    if editor.busy.swap(true, Ordering::SeqCst) {
        return Err(Error::Message("Another edit is still running".into()));
    }
    let _guard = BusyGuard(&editor.busy);
    editor.cancelled.store(false, Ordering::SeqCst);

    let root = library_root(state)?;
    let source = clip_path(state, id)?;
    let stamps = files::Stamps::of(&source);
    let probe = ffmpeg::probe(&source)?;
    let encoder = if ffmpeg::has_nvenc() { Encoder::Nvenc } else { Encoder::X264 };
    let temp = files::temp_path(&source, "export");

    let result = (|| -> Result<()> {
        let plan = plan::build(&source, &temp, &probe, spec, encoder, 1.0)?;
        editor.run(&plan.args, plan.duration, progress)?;
        // Bitrate control is approximate; if we overshot, tighten once.
        if let Some(target) = spec.target_bytes {
            let size = fs::metadata(&temp)?.len();
            if size > target {
                let scale = 0.92 * target as f64 / size as f64;
                let plan = plan::build(&source, &temp, &probe, spec, encoder, scale)?;
                editor.run(&plan.args, plan.duration, progress)?;
            }
        }
        Ok(())
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&temp);
        return Err(e);
    }

    let dir = source.parent().unwrap_or(&root);
    let stem = source.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let is_mp4 = source.extension().is_some_and(|e| e.eq_ignore_ascii_case("mp4"));
    let final_path = match destination {
        Destination::New => {
            let label: String = spec.label.chars().filter(|c| !c.is_control()).collect();
            let label = files::validate_name(&label).unwrap_or("edited");
            files::unique_path(dir, &format!("{stem} ({label})"), "mp4")
        }
        Destination::Replace if is_mp4 => source.clone(),
        // The result is always MP4; an .mkv/.mov original gets a sibling.
        Destination::Replace => files::unique_path(dir, &stem, "mp4"),
    };

    let finalize = if final_path == source {
        files::replace(&source, &temp)
    } else {
        files::rename(&temp, &final_path).and_then(|_| match destination {
            Destination::Replace => files::remove(&source),
            Destination::New => Ok(()),
        })
    };
    if let Err(e) = finalize {
        let _ = fs::remove_file(&temp);
        return Err(e);
    }

    // The export is a brand-new file; give it the original's recording date.
    stamps.apply(&final_path);

    let old_id = match destination {
        Destination::Replace => id,
        Destination::New => "",
    };
    commit(state, &root, old_id, &final_path, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// End-to-end check against a real ffmpeg; skipped when none is installed.
    #[test]
    fn exports_trimmed_and_compressed_clips() {
        if ffmpeg::path().is_none() {
            eprintln!("ffmpeg not found; skipping");
            return;
        }
        let dir = std::env::temp_dir().join(format!("shindeck-export-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let source = dir.join("source.mp4");
        // 20 s of 720p60 video with a game track and a separate mic track,
        // like ShadowPlay's "separate both tracks" option.
        let status = Command::new(ffmpeg::path().unwrap())
            .args(["-hide_banner", "-loglevel", "error", "-y"])
            .args(["-f", "lavfi", "-i", "testsrc2=size=1280x720:rate=60:duration=20"])
            .args(["-f", "lavfi", "-i", "sine=frequency=440:duration=20"])
            .args(["-f", "lavfi", "-i", "sine=frequency=880:duration=20"])
            .args([
                "-map",
                "0",
                "-map",
                "1",
                "-map",
                "2",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
            ])
            .args(["-c:a", "aac", "-shortest"])
            .arg(&source)
            .status()
            .unwrap();
        assert!(status.success());
        let probe = ffmpeg::probe(&source).unwrap();
        assert_eq!(probe.audio_streams, 2);

        let editor = Editor::default();
        let run = |spec: &EditSpec, out: &Path| {
            let plan = plan::build(&source, out, &probe, spec, Encoder::X264, 1.0).unwrap();
            let last = std::cell::Cell::new(0.0);
            editor.run(&plan.args, plan.duration, &|p| last.set(p)).unwrap();
            assert!(last.get() > 0.5, "progress was reported");
        };

        // Keep 2-8 s and 12-18 s: 12 s total, audio tracks mixed into one.
        let trimmed = dir.join("trimmed.mp4");
        let spec = EditSpec {
            keep: Some(vec![(2.0, 8.0), (12.0, 18.0)]),
            target_bytes: None,
            label: "trimmed".into(),
        };
        run(&spec, &trimmed);
        let out = ffmpeg::probe(&trimmed).unwrap();
        assert!((out.duration - 12.0).abs() < 0.2, "duration {}", out.duration);
        assert_eq!(out.audio_streams, 1);

        // Compress the 20 s clip to 2 MB.
        let small = dir.join("small.mp4");
        let target = 2_000_000;
        let spec = EditSpec { keep: None, target_bytes: Some(target), label: "2 MB".into() };
        run(&spec, &small);
        let size = fs::metadata(&small).unwrap().len();
        assert!(size <= target, "{size} bytes exceeds the {target} byte target");
        assert!(ffmpeg::probe(&small).unwrap().height <= 540);

        // Film strip for the trim bar: one JPEG with 16 frames side by side.
        let strips = Filmstrips::new(dir.join("strips"));
        let entry = crate::library::IndexEntry { path: source.clone(), size: 1, modified: 1 };
        let jpeg = strips.get(&entry).expect("film strip");
        assert_eq!(&jpeg[..2], &[0xFF, 0xD8]);
        let image = image::load_from_memory(&jpeg).unwrap();
        assert_eq!((image.width(), image.height()), (16 * 192, 108));

        fs::remove_dir_all(&dir).unwrap();
    }
}
