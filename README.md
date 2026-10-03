# ShinDeck

A fast, lightweight clip manager and editor for **NVIDIA ShadowPlay** recordings on Windows.
Black and NVIDIA green, a gallery organized by game, a keyboard-driven player and a built-in editor.

##Build

To build an installer:
git clone https://github.com/j-marchon/ShinDeck.git
cd ShinDeck
npm install
npm run tauri build

## Where data lives

| What                    | Where                                               |
| ----------------------- | --------------------------------------------------- |
| Settings, favorites, edit marks | `%APPDATA%\com.shindeck.app\settings.json`, `favorites.json`, `edited.json` |
| Thumbnail, icon & film-strip cache | `%LOCALAPPDATA%\com.shindeck.app\`                  |

## Third-party software

The Windows build bundles [FFmpeg](https://ffmpeg.org) 7.1.1 ("essentials" build by Gyan Doshi), licensed under the GPLv3.
FFmpeg runs as a separate program; its source is available from ffmpeg.org.
