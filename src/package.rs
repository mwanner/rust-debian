//! Tools related to Debian packaging
//!
//! This module contains a `Changelog` and a `ControlFile` parser for the
//! Debian changelog and control files usually used for packaging.

use std::env;
use std::fmt;

use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::path::Path;

use chrono::prelude::*;

use super::Version;

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
/// use debian::package::Changelog;
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
        let mut file = match File::create(out_file_path) {
            Ok(f) => f,
            Err(f) => return Err(f),
        };
        for entry in &self.entries {
            match file.write(entry.serialize().as_bytes()) {
                Ok(_) => {}
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

    pub fn from_file(in_file: &Path) -> io::Result<ControlFile> {
        let file = File::open(in_file)?;
        let mut buf = io::BufReader::new(file);
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
                    if is_end_of_para {
                        None
                    } else {
                        Some(line)
                    }
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
        let mut file = match File::create(out_file) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

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


/// Version relations
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VRel {
    GreaterOrEqual,
    Greater,
    LesserOrEqual,
    Lesser,
    Equal,
}

impl fmt::Display for VRel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VRel::GreaterOrEqual => write!(f, ">="),
            VRel::Greater => write!(f, ">>"),
            VRel::LesserOrEqual => write!(f, "<="),
            VRel::Lesser => write!(f, "<<"),
            VRel::Equal => write!(f, "="),
        }
    }
}

/// A dependency on another package
#[derive(Debug, PartialEq, Clone)]
pub struct SingleDependency {
    pub package: String,
    pub version: Option<(VRel, Version)>,
    pub arch: Option<String>,
    pub condition: Option<String>,
}

impl fmt::Display for SingleDependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (&self.version, &self.arch) {
            (&None, &None) => write!(f, "{}", self.package),
            (&Some((ref vrel, ref ver)), &None) => {
                write!(f, "{} ({} {})", self.package, vrel, ver)
            }
            (&None, Some(a)) => write!(f, "{} [{}]", self.package, a),
            (&Some((ref vrel, ref ver)), Some(a)) => {
                write!(f, "{} ({} {}) [{}]", self.package, vrel, ver, a)
            }
        }
    }
}

/// Multiple variants that may statisfy a dependency
#[derive(Debug, PartialEq, Clone)]
pub struct Dependency {
    pub alternatives: Vec<SingleDependency>,
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let alts = self
            .alternatives
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join(" | ");
        write!(f, "{}", alts)
    }
}

/// Parse a single dependency
pub fn parse_single_dep(s: &str) -> Result<SingleDependency, &'static str> {
    enum ST {
        PackageName,
        PreVersion,
        InVersionRel,
        InVersionDef,
        PreArch,
        InArch,
        InDependencyCondition,
        PreDependencyCondition,
        Done,
    }
    let mut st = ST::PackageName;
    let mut result = SingleDependency {
        package: "".to_string(),
        version: None,
        arch: None,
        condition: None,
    };
    let mut vrel = "".to_string();
    let mut vdef = "".to_string();
    let mut arch = "".to_string();
    for ch in s.chars() {
        match st {
            ST::PackageName => {
                if ch.is_whitespace() {
                    st = ST::PreVersion;
                } else if ch == '(' {
                    st = ST::InVersionRel;
                } else {
                    result.package.push(ch);
                }
            }
            ST::PreVersion => {
                if ch.is_whitespace() {
                } else if ch == '(' {
                    st = ST::InVersionRel;
                } else if ch == '<' {
                    st = ST::InDependencyCondition;
                    result.condition = Some("".to_string());
                } else if ch == '[' {
                    st = ST::InArch;
                } else {
                    return Err("garbage after package name");
                }
            }
            ST::InVersionRel => {
                if ch == '>' || ch == '<' || ch == '=' {
                    vrel.push(ch);
                } else if ch == ')' {
                    return Err("no version given");
                } else {
                    st = ST::InVersionDef;
                    vdef.push(ch);
                }
            }
            ST::InVersionDef => {
                if ch == ')' {
                    if let "${binary:Version}" | "${source:Version}" =
                        vdef.trim()
                    {
                        continue;
                    }

                    let version = match Version::parse(vdef.trim()) {
                        Ok(v) => v,
                        Err(_) => return Err("error parsing version"),
                    };
                    result.version = match &vrel[..] {
                        ">=" | ">" => Some((VRel::GreaterOrEqual, version)),
                        ">>" => Some((VRel::Greater, version)),
                        "<=" | "<" => Some((VRel::LesserOrEqual, version)),
                        "<<" => Some((VRel::Lesser, version)),
                        "=" => Some((VRel::Equal, version)),
                        _ => return Err("invalid relation"),
                    };
                    st = ST::PreArch;
                } else {
                    vdef.push(ch);
                }
            }
            ST::PreArch => {
                if ch.is_whitespace() {
                } else if ch == '[' {
                    st = ST::InArch;
                } else {
                    return Err("garbage after version");
                }
            }
            ST::InArch => {
                if ch == ']' {
                    let arch = arch.trim().to_string();
                    if !arch.is_empty() {
                        result.arch = Some(arch);
                    } else {
                        return Err("empty arch given");
                    }
                    st = ST::PreDependencyCondition;
                } else {
                    arch.push(ch);
                }
            }
            ST::InDependencyCondition => {
                if ch == '>' {
                    st = ST::Done
                } else {
                    match result.condition {
                        Some(ref mut c) => c.push(ch),
                        _ => unreachable!(),
                    };
                }
            }
            ST::PreDependencyCondition => {
                if ch.is_whitespace() {
                    continue;
                }
                if ch == '<' {
                    st = ST::InDependencyCondition;
                    result.condition = Some("".to_string());
                } else {
                    st = ST::Done;
                }
            }
            ST::Done => {
                if ch.is_whitespace() {
                } else {
                    return Err("garbage after arch");
                }
            }
        }
    }
    Ok(result)
}

/// Parse a dependency list, comma separated, with pipes separating
/// variants
pub fn parse_dep_list(s: &str) -> Result<Vec<Dependency>, &'static str> {
    let mut result = vec![];
    for s in s.split(',').map(|x| x.trim()) {
        let mut a = vec![];
        for sd in s.split('|').map(|x| x.trim()) {
            a.push(match parse_single_dep(sd) {
                Ok(v) => v,
                Err(e) => return Err(e),
            });
        }
        result.push(Dependency { alternatives: a });
    }
    Ok(result)
}
