// Downloads the ffmpeg build bundled with the Windows app (used by the clip
// editor) into src-tauri/binaries, named the way Tauri's sidecar bundling
// expects. Runs automatically before `tauri dev` / `tauri build`; a no-op
// when the binary is already there or when not building for Windows.
//
// Build: FFmpeg 7.1.1 "essentials" by Gyan Doshi (GPLv3), pinned by hash.
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const VERSION = "7.1.1";
const URL = `https://github.com/GyanD/codexffmpeg/releases/download/${VERSION}/ffmpeg-${VERSION}-essentials_build.zip`;
const SHA256 = "04861d3339c5ebe38b56c19a15cf2c0cc97f5de4fa8910e4d47e5e6404e4a2d4";

const platform = process.env.TAURI_ENV_PLATFORM ?? (process.platform === "win32" ? "windows" : process.platform);
if (platform !== "windows") process.exit(0);

const triple = process.env.TAURI_ENV_TARGET_TRIPLE ?? "x86_64-pc-windows-msvc";
const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const target = join(root, "src-tauri", "binaries", `ffmpeg-${triple}.exe`);
if (existsSync(target)) process.exit(0);

const work = mkdtempSync(join(tmpdir(), "shindeck-ffmpeg-"));
try {
  const zip = join(work, "ffmpeg.zip");
  console.log(`Downloading ffmpeg ${VERSION}…`);
  execFileSync("curl", ["-fsSL", "--retry", "3", "-o", zip, URL], { stdio: "inherit" });

  const hash = createHash("sha256").update(readFileSync(zip)).digest("hex");
  if (hash !== SHA256) throw new Error(`ffmpeg download hash mismatch: ${hash}`);

  const inner = `ffmpeg-${VERSION}-essentials_build/bin/ffmpeg.exe`;
  // Windows' bsdtar reads zip files; elsewhere use unzip.
  if (process.platform === "win32") execFileSync("tar", ["-xf", zip, "-C", work, inner]);
  else execFileSync("unzip", ["-q", zip, inner, "-d", work]);

  mkdirSync(dirname(target), { recursive: true });
  copyFileSync(join(work, inner), target);
  console.log(`ffmpeg ready: ${target}`);
} finally {
  rmSync(work, { recursive: true, force: true });
}
