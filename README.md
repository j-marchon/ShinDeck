# ShinDeck

A fast, lightweight clip manager for **NVIDIA ShadowPlay** recordings on Windows.
Black and NVIDIA green, a gallery organized by game, and a keyboard-driven player.

- **First-launch setup.** Pick your clips folder. ShinDeck pre-fills the folder ShadowPlay actually uses.
- **Organized by game.** ShadowPlay creates one folder per game, and ShinDeck uses that folder as the clip's game.
- **Gallery.** Thumbnails, file name, game name with the game's icon, duration and capture date.
  Sort by date, name, game or favorites, filter by game, and search.
- **Favorites.** Click the star at the top right of any thumbnail, or press `S` in the player.
- **Player.** Time counter, seek bar, play/pause, previous/next clip, slower/faster, volume and fullscreen.
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
| `Esc`         | Back to the gallery       |
| `?`           | Show shortcuts            |

In the gallery, `Ctrl+F` or `/` focuses search.

## Why Tauri + Rust + Svelte

| Concern     | Choice                                                                 |
| ----------- | ---------------------------------------------------------------------- |
| Shell       | **Tauri 2**. Uses the WebView2 runtime that ships with Windows, so the installer is a few MB (an Electron app would be 100 MB or more) and memory use stays low. |
| Backend     | **Rust**. Scans the folders in parallel, extracts thumbnails and icons through Win32, and watches the clips folder. |
| UI          | **Svelte 5**. Compiles to plain DOM updates with no virtual DOM; the whole UI is about 30 KB gzipped. |
| Video       | WebView2's native `<video>` (hardware-decoded H.264/HEVC/AV1), streamed via Tauri's asset protocol with HTTP range requests, so seeking never reads the whole file. |
| Installer   | **NSIS** via the Tauri bundler: per-user install with no admin prompt, Start-menu entry, uninstaller, and the WebView2 bootstrapper embedded in case the runtime is missing. |

### Performance notes

- **Virtualized gallery.** Only the rows near the viewport exist in the DOM, so 10,000 clips scroll as smoothly as 10.
- **Thumbnails come from the Windows Shell** (`IShellItemImageFactory`, the same API Explorer uses).
  - Nothing is decoded in the app, and no ffmpeg is bundled.
  - Clips Explorer has already seen come straight from the system thumbnail cache.
  - Results are re-encoded once to a 512 px JPEG in `%LOCALAPPDATA%\com.shindeck.app\thumbnails`. The cache key includes file size and modification time, so a replaced clip gets a new thumbnail.
  - Thumbnails are produced on a small pool of COM threads, newest request first. When you scroll quickly, the cards on screen are served before the ones you scrolled past.
- **Durations** are read straight from the MP4 `moov/mvhd` box, which takes a few bytes per file, and are reused across rescans.
- **Lazy loading.** Thumbnails and icons are plain `<img loading="lazy">` tags pointing at custom `thumb://` and `gameicon://` protocols, so the webview handles lazy loading and decoding natively.

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
| Settings, favorites     | `%APPDATA%\com.shindeck.app\settings.json`, `favorites.json` |
| Thumbnail & icon cache  | `%LOCALAPPDATA%\com.shindeck.app\`                  |

Favorites are keyed by the clip's path relative to the library root, so they survive moving the whole folder.
ShinDeck never modifies or moves your clips.

### Default clips folder

On first launch the setup screen pre-fills, in order:

1. The path configured in GeForce Experience or the NVIDIA app (registry: `HKCU\Software\NVIDIA Corporation\Global\ShadowPlay\NVSPCAPS\DefaultPathW`).
2. `Videos\NVIDIA` (the NVIDIA app's default), if it exists.
3. Your `Videos` folder (GeForce Experience's default).

You can browse to any other folder, and change it later from the folder button at the bottom of the sidebar.

## Project layout

```
src/                         Svelte UI
  App.svelte                 screen routing (setup / library / player)
  lib/api/                   typed backend interface (Tauri impl + browser mock)
  lib/state/library.svelte.ts  library store: filtering, sorting, favorites
  lib/components/            Gallery (virtualized), ClipCard, Player, Setup, …
src-tauri/                   Rust backend
  src/commands.rs            commands exposed to the UI
  src/library/               folder scanner + MP4 duration reader
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

### Building the installer

```sh
npm run tauri build
# -> src-tauri/target/release/bundle/nsis/ShinDeck_0.1.0_x64-setup.exe
```

The `Windows build` GitHub Actions workflow builds the installer on every push.
It is attached to the run as an artifact, and to a draft release when you push a `v*` tag.

The installer embeds the WebView2 bootstrapper (about 2 MB). WebView2 is already present on Windows 11 and up-to-date Windows 10.
On a machine without it, the bootstrapper installs it, which needs internet access.
For a fully offline installer, set `bundle.windows.webviewInstallMode` to `{"type": "offlineInstaller"}`. This adds about 130 MB.
