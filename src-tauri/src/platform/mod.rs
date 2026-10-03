//! OS integration. Windows is the real target; the fallback keeps the app
//! compiling and usable (without thumbnails/icons) on other platforms for
//! development.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;

#[cfg(not(windows))]
mod fallback;
#[cfg(not(windows))]
pub use self::fallback::*;
