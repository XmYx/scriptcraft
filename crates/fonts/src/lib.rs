#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
//! The screenplay typeface.
//!
//! Scripts are set in 12 pt fixed-pitch type at 10 characters per inch. ScriptCraft uses
//! Liberation Mono (SIL Open Font License 1.1, metric-compatible with the common Courier-style
//! faces) from [craft-fonts](https://github.com/storytold/craft-fonts), embedded at build time when
//! `CRAFT_FONTS_DIR` names a checkout. Without it [`SCREENPLAY_FONTS`] is empty and callers fall
//! back to another fixed-pitch face; pagination does not depend on the face.

pub use craft_fonts_build::EmbeddedFace;

include!(concat!(env!("OUT_DIR"), "/craft_fonts.rs"));

/// The embedded face with this style (`Regular`, `Bold`, `Italic`, `Bold Italic`).
pub fn face(style: &str) -> Option<&'static EmbeddedFace> {
    SCREENPLAY_FONTS.iter().find(|f| f.style.eq_ignore_ascii_case(style))
}

#[cfg(test)]
mod tests {
    #[test]
    fn faces_are_liberation_mono_when_present() {
        for f in super::SCREENPLAY_FONTS {
            assert_eq!(f.family, "Liberation Mono");
            assert!(f.bytes.len() > 1000);
        }
        assert_eq!(super::face("Regular").is_some(), !super::SCREENPLAY_FONTS.is_empty());
    }
}
