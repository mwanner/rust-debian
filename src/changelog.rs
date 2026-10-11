//! Tools related to Debian packaging
//!
//! This module contains a `Changelog` and a `ControlFile` parser for the
//! Debian changelog and control files usually used for packaging.

use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::Path;

use chrono::prelude::*;

/// Represents a single entry in a debian/changelog file.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
pub struct ChangelogEntry {
    /// source package name
    pkg: String,
    /// debian revision
    version: String,
    /// distribution(s) where this version should be installed when it
    /// is uploaded
    distributions: Vec<String>,
    // urgency of the upload
    urgency: String,
    // changelog description
    detail: String,
    // name of the uploader of the package
    maintainer_name: String,
    // email of the uploader of the package
    maintainer_email: String,
    // date of the upload
    ts: DateTime<Local>,
}

/// Represents a complete debian/changelog file
///
/// Implemented simply as a collection of `ChangelogEntry`, completely
/// stored in memory.
///
/// # Examples
///
/// ```
/// use debian::Changelog;
/// use std::path::Path;
///
/// let changelog = Changelog::from_file(Path::new("debian/changelog"));
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
pub struct Changelog {
    entries: Vec<ChangelogEntry>,
}

impl ChangelogEntry {
    /// Create a new ChangelogEntry
    pub fn new(pkg: String, version: String, detail: String) -> ChangelogEntry {
        ChangelogEntry {
            pkg,
            version,
            distributions: vec!["UNRELEASED".to_string()],
            urgency: "medium".to_string(),
            detail,
            maintainer_name: get_default_maintainer_name(),
            maintainer_email: get_default_maintainer_email(),
            ts: Local::now(),
        }
    }

    fn serialize(&self) -> String {
        format!(
            "{} ({}) {}; urgency={}\n\n{}\n -- {} <{}>  {}\n\n",
            self.pkg,
            self.version,
            self.distributions.join(" "),
            self.urgency,
            self.detail,
            self.maintainer_name,
            self.maintainer_email,
            self.ts.to_rfc2822()
        )
    }
}

impl Changelog {
    #[doc(hidden)]
    #[deprecated(
        since = "0.2.0",
        note = "use `from_file` or `default` instead"
    )]
    /// Creates a new Changelog starting from a single entry.
    pub fn new(single_entry: ChangelogEntry) -> Changelog {
        Changelog {
            entries: vec![single_entry],
        }
    }

    /// Serializes this `Changelog` to a file on disk.
    ///
    /// Creates the file, if it doesn't already exist, overrides it otherwise.
    ///
    /// # Errors
    ///
    /// This function uses `File::create` and forwards any possible error.
    pub fn to_file(&self, out_file_path: &Path) -> io::Result<()> {
        let mut file = File::create(out_file_path)?;
        for entry in &self.entries {
            match file.write_all(entry.serialize().as_bytes()) {
                Ok(()) => {}
                Err(f) => return Err(f),
            }
        }
        Ok(())
    }

    /// Deserialize a debian/changelog file from disk.
    ///
    /// Reads a Debian changelog file into memory.
    pub fn from_file(in_file: &Path) -> io::Result<Changelog> {
        let file = File::open(in_file)?;
        let mut buf = io::BufReader::new(file);
        let entries = vec![];
        loop {
            let mut line = String::new();
            buf.read_line(&mut line)?;
            let is_eof = line.is_empty();

            // Loop termination condition
            if is_eof {
                break;
            }
        }

        Ok(Changelog { entries })
    }
}

/// A helper routine to determine the default Debian maintainer name
/// from the environment.
pub fn get_default_maintainer_name() -> String {
    match env::var("DEBFULLNAME") {
        Ok(name) => name,
        Err(_) => match env::var("NAME") {
            Ok(name) => name,
            Err(_) => "Mickey Mouse".to_string(),
        },
    }
}

/// A helper routine to determine the default Debian email address
/// from the environment.
pub fn get_default_maintainer_email() -> String {
    match env::var("DEBEMAIL") {
        Ok(email) => email,
        Err(_) => match env::var("EMAIL") {
            Ok(email) => email,
            Err(_) => "mmouse@disney.com".to_string(),
        },
    }
}

// Formerly duplicated here, kept for compatibility.
pub use super::relation::{
    Dependency, SingleDependency, VRel, parse_dep_list, parse_single_dep,
};
