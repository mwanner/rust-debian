//! Helper library for anything Debian related.

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

pub mod changelog;
pub mod control;
pub mod version;

pub use self::changelog::{Changelog, ChangelogEntry};
pub use self::control::{
    ControlEntry, ControlFile, ControlParagraph, ControlValue,
};
pub use self::version::Version;
