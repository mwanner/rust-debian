//! Tools related to Debian control files
//!
//! This module contains a `ControlFile` parser for the control files
//! usually used for packaging. Parsing the values of relationship fields
//! is covered by the `relation` module.

use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::Path;

use log::*;

/// A value in a field of a control file
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum ControlValue {
    /// A simple string value
    Simple(String),
    /// A folder string value
    Folded(String),
    /// A multiline string value
    MultiLine(String),
}

/// A single field or entry in a control file
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct ControlEntry {
    key: String,
    value: ControlValue,
}

/// A paragraph consisting of multiple entries of type `ControlEntry`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct ControlParagraph {
    entries: Vec<ControlEntry>,
}

/// A control file consisting of multiple paragraphs.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
pub struct ControlFile {
    paragraphs: Vec<ControlParagraph>,
}

impl ControlValue {
    /// Creates a `ControlValue` from a `String` choosing its type
    /// from the key.
    pub fn new(key: &str, val: String) -> ControlValue {
        match key {
            "Architecture" | "Built-Using" | "Changed-By" | "Essential"
            | "Homepage" | "Installed-Size" | "Maintainer" | "Package"
            | "Package-Type" | "Priority" | "Section" | "Source"
            | "Standards-Version" | "Vcs-Browser" | "Vcs-Git" | "Version" => {
                ControlValue::Simple(val)
            }
            "Binaries"
            | "Breaks"
            | "Build-Depends"
            | "Build-Depends-Indep"
            | "Depends"
            | "Dgit"
            | "Pre-Depends"
            | "Recommends"
            | "Replaces"
            | "Suggests"
            | "Uploaders" => ControlValue::Folded(val),
            "Description" => ControlValue::MultiLine(val),
            _ => {
                debug!("Unknown key: {}", key);
                ControlValue::Simple(val)
            }
        }
    }
}

impl ControlEntry {
    /// Creates a new `ControlEntry` given a key-value pair.
    pub fn new(key: &str, val: String) -> ControlEntry {
        ControlEntry {
            key: key.to_string(),
            value: ControlValue::new(key, val),
        }
    }
}

impl ControlParagraph {
    #[doc(hidden)]
    #[deprecated(since = "0.2.0", note = "use `default` instead")]
    /// Creates a new `ControlParagraph`
    pub fn new() -> ControlParagraph {
        ControlParagraph { entries: vec![] }
    }

    /// Append an entry at the end of the paragraph.
    pub fn add_entry(&mut self, key: &str, val: String) {
        let e = ControlEntry::new(key, val);
        self.entries.push(e);
    }

    /// Update or append an entry in the paragraph, returning true if
    /// the entry was found and replaced, false if appended.
    pub fn update_entry(&mut self, key: &str, val: String) -> bool {
        for entry in &mut self.entries {
            if entry.key == key {
                entry.value = ControlValue::new(key, val);
                return true;
            }
        }

        // append entry
        self.add_entry(key, val);
        false
    }

    /// Check if an entry exists in the paragraph
    pub fn has_entry(&self, key: &str) -> bool {
        for entry in &self.entries {
            if entry.key == key {
                return true;
            }
        }
        false
    }

    /// Get the value of an entry in the paragraph
    pub fn get_entry(&self, key: &str) -> Option<&str> {
        for entry in &self.entries {
            if entry.key == key {
                return Some(match entry.value {
                    ControlValue::Simple(ref v)
                    | ControlValue::Folded(ref v)
                    | ControlValue::MultiLine(ref v) => v,
                });
            }
        }
        None
    }
}

impl ControlFile {
    #[doc(hidden)]
    #[deprecated(
        since = "0.2.0",
        note = "use `from_file` or `default` instead"
    )]
    /// Creates a new `ControlFile`.
    pub fn new() -> ControlFile {
        ControlFile { paragraphs: vec![] }
    }

    pub fn add_paragraph(&mut self, p: ControlParagraph) {
        self.paragraphs.push(p);
    }

    /// Parses a control file from the given path.
    pub fn from_file(in_file: &Path) -> io::Result<ControlFile> {
        let file = File::open(in_file)?;
        ControlFile::from_reader(io::BufReader::new(file))
    }

    /// Parses a control file from any buffered reader, e.g. an in-memory
    /// buffer or a network stream.
    pub fn from_reader<R: BufRead>(mut buf: R) -> io::Result<ControlFile> {
        let mut paragraphs = Vec::new();
        let mut cur_entry: Option<String> = None;
        let mut cur_para = ControlParagraph::default();
        loop {
            let mut line = "".to_string();

            buf.read_line(&mut line)?;
            let is_eof = line.is_empty();

            let (is_end_of_para, is_indented) = {
                let trimmed_line = line.trim();
                (
                    trimmed_line.is_empty(),
                    line.starts_with(' ') && line.len() > 1,
                )
            };

            // Possibly terminate the current entry and append to the
            // current paragraph.
            cur_entry = match (cur_entry, is_indented, is_end_of_para) {
                (Some(v), false, _) => {
                    // terminate the last entry
                    let mut v2 = v.splitn(2, ':');
                    let key = v2.next().unwrap();
                    match v2.next() {
                        Some(value) => {
                            let value = value.trim().to_string();
                            cur_para.add_entry(key, value);
                        }
                        None => {
                            // FIXME: handle this parser error!
                            debug!(
                                "Parser error in line before: '{}', with value '{}'",
                                line, v
                            );
                        }
                    };

                    // begin new entry
                    if is_end_of_para { None } else { Some(line) }
                }
                (Some(v), true, false) => Some(v + &line),
                (None, _, false) => Some(line),
                (_, _, true) => None,
            };

            // Possibly terminate the current paragraph and append it
            // to the main structure.
            if is_end_of_para && !cur_para.entries.is_empty() {
                paragraphs.push(cur_para);
                cur_para = ControlParagraph::default();
            }

            // Loop termination condition
            if is_eof {
                break;
            }
        }

        Ok(ControlFile { paragraphs })
    }

    pub fn serialize(&self, out_file: &Path) -> io::Result<()> {
        let mut file = File::create(out_file)?;
        for para in &self.paragraphs {
            for entry in &para.entries {
                let v = match entry.value.clone() {
                    ControlValue::Simple(v)
                    | ControlValue::Folded(v)
                    | ControlValue::MultiLine(v) => v,
                };
                let s = entry.key.clone() + ": " + &v + "\n";
                file.write_all(s.as_bytes())?;
            }
            file.write_all(b"\n")?;
        }

        Ok(())
    }

    pub fn get_paragraphs(&self) -> &Vec<ControlParagraph> {
        &self.paragraphs
    }
}

// Formerly defined here, kept for compatibility.
pub use super::relation::{
    Dependency, SingleDependency, VRel, parse_dep_list, parse_single_dep,
};
