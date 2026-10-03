//! Thumbnails, game icons and the custom URI protocols that serve them.

mod icons;
pub mod protocol;
mod thumbnails;
mod worker;

pub use icons::GameIcons;
pub use thumbnails::Thumbnails;
pub use worker::Worker;

/// FNV-1a: tiny, fast and stable across builds (unlike `DefaultHasher`),
/// which matters because hashes name files in the on-disk cache.
pub(crate) fn stable_hash(parts: &[&[u8]]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for &byte in *part {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
