# ShinDeck

A fast, lightweight clip manager and editor for **NVIDIA ShadowPlay** recordings on Windows.
Black and NVIDIA green, a gallery organized by game, a keyboard-driven player and a built-in editor.

- **Custom window.** ShinDeck draws its own title bar. Minimize and maximize glow green, close turns red, and the thin window border is tinted to match.
- **First-launch setup.** Pick your clips folder. ShinDeck pre-fills the folder ShadowPlay actually uses.
- **Organized by game.** ShadowPlay creates one folder per game, and ShinDeck uses that folder as the clip's game.
- **Gallery.**
  - Each card shows the thumbnail, file name, and the game's name and icon, plus file size, date (`21 Aug '26`) and duration.
  - Clips edited in ShinDeck get a green pencil before their name.
- **Filters in one bar.** All of them combine:
  - game: a searchable menu, with *All games* pinned at the top
  - date range
  - favorites only
  - name search
  The sort menu orders by date, name, game or size.
- **Inline rename.** Click a clip's title and type a new name. The file is renamed on disk, and favorites and edit marks follow it.
- **Favorites and edit shortcut.** Hover a thumbnail for the star and the pencil. The pencil opens the clip with the editor ready.
- **Player.**
  - Opens in a large panel over the dimmed gallery, with a green glow.
  - Controls: time counter, seek bar, play/pause, previous/next clip, slower/faster, volume and fullscreen.
- **Editor.** The highlighted **Edit** button slides in a side panel with:
  - **Rename.**
  - **Trim & Cut.** The seek bar turns into a film strip:
    - Drag the green handles to trim the start and end.
    - Right-click twice to mark a section to cut out. You can cut as many sections as you like.
    - Everything except the video and the film strip dims while you work.
    - Playback previews the result, skipping the cut parts.
  - **Compress** to fit Discord (10 MB), WhatsApp (16 MB), Instagram (50 MB) or any custom size.
  - **Save choice.** The first edit asks whether to save a **new clip** or **replace the original**. You can remember the choice or be asked every time, and change it later in Settings or at the bottom of the edit panel.
- **Live updates.** New clips appear as soon as ShadowPlay saves them.

## Keyboard shortcuts (player)

| Key           | Action                    |
| ------------- | ------------------------- |
| `Space` / `K` | Play / pause              |
| `←` / `→`     | Back / forward 5 seconds  |
| `N` / `P`     | Next / previous clip      |
| `[` / `]`     | Slower / faster (0.25×–4×)|
| `0`           | Normal speed              |
| `↑` / `↓`     | Volume                    |
| `M`           | Mute                      |
| `F`           | Fullscreen                |
| `S`           | Toggle favorite           |
| `E`           | Open/close the edit panel |
| `Enter`       | Save the trim (trim mode) |
| `Esc`         | Leave trim mode / back to the gallery |
| `?`           | Show shortcuts            |

In the gallery, `Ctrl+F` or `/` focuses search.

## Why Tauri + Rust + Svelte

| Concern     | Choice                                                                 |
| ----------- | ---------------------------------------------------------------------- |
| Shell       | **Tauri 2**. Uses the WebView2 runtime that ships with Windows, so the app itself is a few MB (an Electron app would be 100 MB or more) and memory use stays low. The installer is about 24 MB, almost all of it the bundled ffmpeg used by the editor. |
| Backend     | **Rust**. Scans the folders in parallel, extracts thumbnails and icons through Win32, and watches the clips folder. |
| UI          | **Svelte 5**. Compiles to plain DOM updates with no virtual DOM; the whole UI is about 50 KB gzipped. |
| Video       | WebView2's native `<video>` (hardware-decoded H.264/HEVC/AV1), streamed via Tauri's asset protocol with HTTP range requests, so seeking never reads the whole file. |
| Installer   | **NSIS** via the Tauri bundler: per-user install with no admin prompt, Start-menu entry, uninstaller, and the WebView2 bootstrapper embedded in case the runtime is missing. |

### Performance notes

- **Virtualized gallery.** Only the rows near the viewport exist in the DOM, so 10,000 clips scroll as smoothly as 10.
- **Thumbnails come from the Windows Shell** (`IShellItemImageFactory`, the same API Explorer uses).
  - Nothing is decoded in the app; ffmpeg is used only by the editor.
  - Clips Explorer has already seen come straight from the system thumbnail cache.
  - Results are re-encoded once to a 512 px JPEG in `%LOCALAPPDATA%\com.shindeck.app\thumbnails`. The cache key includes file size and modification time, so a replaced clip gets a new thumbnail.
  - Thumbnails are produced on a small pool of COM threads, newest request first. When you scroll quickly, the cards on screen are served before the ones you scrolled past.
- **Durations** are read straight from the MP4 `moov/mvhd` box, which takes a few bytes per file, and are reused across rescans.
- **Lazy loading.** Thumbnails and icons are plain `<img loading="lazy">` tags pointing at custom `thumb://` and `gameicon://` protocols, so the webview handles lazy loading and decoding natively.

## How editing works

Editing uses a bundled **ffmpeg 7.1.1** (`src-tauri/src/editor`).
It is installed next to `ShinDeck.exe` and never runs while you are only browsing.

- **Every edit re-encodes to H.264 + AAC MP4,** which Discord, WhatsApp and Instagram all play.
  - Cuts are frame-accurate. A stream copy can only cut on keyframes, which ShadowPlay places seconds apart.
  - On NVIDIA GPUs the encode runs on **NVENC**, so it is fast. Without NVENC it falls back to x264.
- **Trim & Cut** turns each range to keep into its own fast-seeking input, then joins them with ffmpeg's `concat` filter, so removed parts are never decoded.
  - ShadowPlay's separate microphone track is mixed into the game audio.
- **Compression** picks the video bitrate that fits the target size, keeping a 6% safety margin.
  - It lowers resolution (1080p, 720p, 540p or 360p) and frame rate (60 to 30 fps) when the bitrate gets low, because that looks far better than a starved bitrate.
  - If the file still overshoots, it re-encodes once with a tighter budget.
- **Saving safely.**
  - Exports are written to a hidden temporary file next to the clip first.
  - When replacing, the original is kept as a backup until the swap succeeds.
  - A failed or cancelled edit never touches your clip.
- The **film strip** in trim mode is 16 frames rendered into one cached image by a single ffmpeg run.

The platform size limits live in `PRESETS` in `src/lib/components/player/EditPanel.svelte`.

## How game icons work

ShadowPlay only records the game's **name**, as the folder name (for example `Counter-strike 2`).
It doesn't record where the game is installed. ShinDeck matches that name against sources that do know where the game is installed
(`src-tauri/src/media/icons.rs`):

1. **Start menu and desktop shortcuts** (`.lnk` and `.url`).
   - Steam, Epic, Battle.net, EA, Ubisoft and Riot all create these.
   - The Windows Shell resolves each shortcut to the game executable's own high-resolution icon.
   - Steam's `.url` shortcuts point `IconFile=` at a `.ico` in `Steam\steam\games`, which is used directly.
2. **Steam's library cache**, used when there is no shortcut.
   - `steamapps\libraryfolders.vdf` lists every Steam library.
   - Each `appmanifest_*.acf` maps a game name to its app id.
   - `appcache\librarycache\<appid>` contains the icon Steam itself displays.

Names are compared after normalization: lowercase, letters and digits only, with ™, ® and punctuation dropped.
So `Counter-strike 2` matches `Counter-Strike® 2`.
An exact match wins. Otherwise ShinDeck uses a containment match between names of similar length, so `Halo` doesn't match `Halo Infinite Multiplayer Editor`.
Shortcuts such as "Uninstall …" and "Readme" are ignored.

Found icons are cached as 64 px PNGs. A game without a match gets a colored monogram tile instead.
ShadowPlay's `Desktop` folder and loose clips get built-in glyphs.

## Where data lives

| What                    | Where                                               |
| ----------------------- | --------------------------------------------------- |
| Settings, favorites, edit marks | `%APPDATA%\com.shindeck.app\settings.json`, `favorites.json`, `edited.json` |
| Thumbnail, icon & film-strip cache | `%LOCALAPPDATA%\com.shindeck.app\`                  |

Favorites are keyed by the clip's path relative to the library root, so they survive moving the whole folder.
ShinDeck only changes your clips when you rename or edit them.

### Default clips folder

On first launch the setup screen pre-fills, in order:

1. The path configured in GeForce Experience or the NVIDIA app (registry: `HKCU\Software\NVIDIA Corporation\Global\ShadowPlay\NVSPCAPS\DefaultPathW`).
2. `Videos\NVIDIA` (the NVIDIA app's default), if it exists.
3. Your `Videos` folder (GeForce Experience's default).

You can browse to any other folder, and change it later in **Settings** (the gear in the top bar).

## Project layout

```
src/                         Svelte UI
  App.svelte                 screen routing (setup / library / player)
  lib/api/                   typed backend interface (Tauri impl + browser mock)
  lib/state/library.svelte.ts  library store: filtering, sorting, favorites
  lib/state/editor.svelte.ts   export jobs, progress and the save prompt
  lib/components/            Gallery (virtualized), ClipCard, Toolbar, GameFilter, TitleBar, …
  lib/components/player/     PlayerModal, EditPanel, TrimBar (film strip), SavePrompt
src-tauri/                   Rust backend
  src/commands.rs            commands exposed to the UI
  src/library/               folder scanner + MP4 duration reader
  src/editor/                rename, ffmpeg trim/cut/compress, film strips
  src/media/                 thumbnail & icon services, custom protocols, worker pool
  src/platform/              Windows Shell / registry integration (+ non-Windows stubs)
  src/watcher.rs             live folder watching
  tauri.conf.json            window, security (CSP, asset scope) and installer config
```

To add a feature:

1. Add a command in `commands.rs` and register it in `lib.rs`.
2. Add it to the `Backend` interface in `src/lib/api/types.ts`.
3. Implement it in `tauri.ts`. TypeScript will then point at the mock that needs it too.

## Development

Prerequisites: [Node 20+](https://nodejs.org), [Rust](https://rustup.rs), and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (on Windows: MSVC Build Tools and WebView2).

```sh
npm install
npm run tauri dev      # run the app with hot reload
npm run dev            # UI only, in a browser, with generated mock data
npm run check          # type-check the UI
cargo test --manifest-path src-tauri/Cargo.toml
```

ShinDeck targets Windows. The app also builds and runs on Linux and macOS for development, with two limits:

- Thumbnails and game icons come from the Windows Shell, so other platforms show placeholders.
- On Linux, WebKitGTK cannot play `<video>` from Tauri's custom asset scheme.

The first `npm run tauri dev` or `npm run tauri build` for Windows downloads ffmpeg into `src-tauri/binaries/`. This is `scripts/fetch-ffmpeg.mjs`: a pinned version, verified by SHA-256, and git-ignored.

### Building the installer

```sh
npm run tauri build
# -> src-tauri/target/release/bundle/nsis/ShinDeck_0.2.0_x64-setup.exe
```

The `Windows build` GitHub Actions workflow builds the installer on every push.
It is attached to the run as an artifact, and to a draft release when you push a `v*` tag.

The installer embeds the WebView2 bootstrapper (about 2 MB). WebView2 is already present on Windows 11 and up-to-date Windows 10.
On a machine without it, the bootstrapper installs it, which needs internet access.
For a fully offline installer, set `bundle.windows.webviewInstallMode` to `{"type": "offlineInstaller"}`. This adds about 130 MB.

## Third-party software

The Windows build bundles [FFmpeg](https://ffmpeg.org) 7.1.1 ("essentials" build by Gyan Doshi), licensed under the GPLv3.
FFmpeg runs as a separate program; its source is available from ffmpeg.org.
