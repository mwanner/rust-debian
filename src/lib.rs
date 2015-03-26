#![feature(core)]  // for unstable from_str_radix in std::num

extern crate chrono;
// extern crate email;
#[macro_use] extern crate log;

pub mod package;
pub mod version;
pub use self::version::Version;
