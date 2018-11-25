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

extern crate chrono;
#[macro_use]
extern crate log;

pub mod package;
pub mod version;
pub use self::version::Version;
