use std::fmt;
use std::env;

use std::fs::File;
use std::io;
use std::io::{Write, BufRead};
use std::path::Path;

use chrono::{FixedOffset, Local};
use chrono::datetime::DateTime;
// FIXME: this might be useful:
// use email::rfc822::Rfc822DateParser;

use super::Version;

pub struct ChangelogEntry {
    pkg: String,
    version: String,
    dist: String,
    urgency: String,
    detail: String,
    maintainer_name: String,
	maintainer_email: String,
    ts: DateTime<FixedOffset>
}

pub struct Changelog {
    entries: Vec<ChangelogEntry>
}

impl ChangelogEntry {
    pub fn new(pkg: String, version: String, detail: String)
               -> ChangelogEntry {
        ChangelogEntry {
            pkg: pkg,
            version: version,
            dist: "UNRELEASED".to_string(),
            urgency: "medium".to_string(),
            detail: detail,
            maintainer_name: get_default_maintainer_name(),
			maintainer_email: get_default_maintainer_email(),
            ts: get_default_ts()
        }
    }

    fn serialize(&self) -> String {
        let ts_str = self.ts.format("%a, %d %b %Y %H:%M:%S %z");
        format!("{} ({}) {}; urgency={}\n\n{}\n -- {} <{}>  {}\n\n",
                self.pkg, self.version, self.dist, self.urgency,
                self.detail,
                self.maintainer_name,
				self.maintainer_email,
				ts_str
                ).to_string()
    }
}

impl Changelog {
    pub fn new(single_entry: ChangelogEntry) -> Changelog {
        Changelog {
            entries: vec![single_entry]
        }
    }

    pub fn to_file(&self, out_file_path: &Path) -> io::Result<()> {
        let mut file = match File::create(out_file_path, ) {
            Ok(f) => f,
            Err(f) => return Err(f)
        };
        for entry in self.entries.iter() {
            match file.write(entry.serialize().as_bytes()) {
                Ok(_) => {},
                Err(f) => return Err(f)
            }
        }
        Ok(())
    }
}

pub fn get_default_maintainer_name() -> String {
    match env::var("DEBFULLNAME") {
        Ok(name) => name,
        Err(_) => match env::var("NAME") {
            Ok(name) => name,
            Err(_) => "Mickey Mouse".to_string()
        }
    }
}

pub fn get_default_maintainer_email() -> String {
    match env::var("DEBEMAIL") {
        Ok(email) => email,
        Err(_) => match env::var("EMAIL") {
            Ok(email) => email,
            Err(_) => "mmouse@disney.com".to_string()
        }
    }
}

fn get_default_ts() -> DateTime<FixedOffset> {
    let now = Local::now();
	return now.with_timezone(&now.offset());
}

/*
fn parse_changelog() -> Changelog {
    let mut result = Changelog { entries: vec![] };

    let now = Local::now();
    let offset = FixedOffset::east(now.offset().local_minus_utc().
                                   num_seconds() as i32);

    let e = ChangelogEntry {
        pkg: "aoeu".to_string(),
        version: "1.0-1".to_string(),
        dist: "UNRELEASED".to_string(),
        urgency: "medium".to_string(),
        detail: "".to_string(),
        maintainer_name: get_default_maintainer_name(),
        maintainer_email: get_default_maintainer_email(),
        ts: now.with_offset::<FixedOffset>(offset)
    };

    result.entries.push(e);
    
    return result;
}
*/

#[derive(Debug, Clone)]
pub enum ControlValue {
    Simple(String),
    Folded(String),
    MultiLine(String)
}

#[derive(Debug, Clone)]
pub struct ControlEntry {
    key: String,
    value: ControlValue
}

#[derive(Debug, Clone)]
pub struct ControlParagraph {
    entries: Vec<ControlEntry>
}

#[derive(Debug)]
pub struct ControlFile {
    paragraphs: Vec<ControlParagraph>
}

impl ControlEntry {
    pub fn new(key: &str, val: String) -> ControlEntry {
        let cval = match key {
            // Fields appearing in both types of source paragraphs
            "Maintainer" => ControlValue::Simple(val),
            "Section" => ControlValue::Simple(val),
            "Priority" => ControlValue::Simple(val),

            "Pre-Depends" => ControlValue::Folded(val),
            "Depends" => ControlValue::Folded(val),
            "Build-Depends" => ControlValue::Folded(val),
            "Build-Depends-Indep" => ControlValue::Folded(val),

            "Homepage" => ControlValue::Simple(val),

            // Fields appearing in the general paragraph, only
            "Source" => ControlValue::Simple(val),
            "Uploaders" => ControlValue::Folded(val),
            "Standards-Version" => ControlValue::Simple(val),
            "Vcs-Browser" => ControlValue::Simple(val),
            "Vcs-Git" => ControlValue::Simple(val),
            "Recommends" => ControlValue::Folded(val),
            "Suggests" => ControlValue::Folded(val),
            "Breaks" => ControlValue::Folded(val),
            "Replaces" => ControlValue::Folded(val),

            // Fields appearing in binary paragraph, only
            "Package" => ControlValue::Simple(val),
            "Changed-By" => ControlValue::Simple(val),
            "Architecture" => ControlValue::Simple(val),
            "Essential" => ControlValue::Simple(val),
            "Description" => ControlValue::MultiLine(val),
            "Built-Using" => ControlValue::Simple(val),
            "Binaries" => ControlValue::Folded(val),
            "Package-Type" => ControlValue::Simple(val),
            "Dgit" => ControlValue::Folded(val),

            // Fields appearing in binary packages' control files
            "Version" => ControlValue::Simple(val),
            "Installed-Size" => ControlValue::Simple(val),

            _ => {
                debug!("Unknown key: {}", key);
                ControlValue::Simple(val)
            }
        };
        ControlEntry { key: key.to_string(), value: cval }
    }
}

impl ControlParagraph {
    pub fn new() -> ControlParagraph {
        ControlParagraph {
            entries: vec![]
        }
    }

    pub fn add_entry(&mut self, key: &str, val: String) {
        let e = ControlEntry::new(key, val);
        self.entries.push(e);
    }

    pub fn update_entry(&mut self, key: &str, val: String) -> bool {
        for entry in self.entries.iter_mut() {
            if entry.key == key {
                // FIXME: ControlValue adaption shouldn't need an
                // instantiation of an entire ControlEntry. Maybe the
                // difference between folded, simple and multi-line is
                // unneeded, anyways?
                let e = ControlEntry::new(key, val);
                entry.value = e.value;
                return true;
            }
        }

        // Append entry
        self.add_entry(key, val);
        return false;
    }

    pub fn has_entry(&self, key: &str) -> bool {
        for entry in self.entries.iter() {
            if entry.key == key {
                return true;
            }
        }
        return false;
    }

    pub fn get_entry(&self, key: &str) -> Option<&str> {
        for entry in self.entries.iter() {
            if entry.key == key {
                return Some(match entry.value {
                    ControlValue::Simple(ref v) => &v,
                    ControlValue::Folded(ref v) => &v,
                    ControlValue::MultiLine(ref v) => &v
                });
            }
        }
        return None;
    }
}

impl ControlFile {
    pub fn new() -> ControlFile {
        ControlFile { paragraphs: vec![] }
    }

    pub fn add_paragraph(&mut self, p: ControlParagraph) {
        self.paragraphs.push(p);
    }
    
    pub fn from_file(in_file: &Path) -> io::Result<ControlFile> {
		let file = try!(File::open(in_file));
        let mut buf = io::BufReader::new(file);
        let mut paragraphs = Vec::new();
        let mut cur_entry: Option<String> = None;
        let mut cur_para = ControlParagraph::new();
        loop {
			let mut line = "".to_string();

			try!(buf.read_line(&mut line));
			let is_eof = line.len() == 0;

			let (is_end_of_para, is_indented) = {
				let trimmed_line = line.trim();
				(trimmed_line.len() == 0,
				 line.starts_with(" ") && line.len() > 1)
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
                        },
                        None => {
                            // FIXME: handle this parser error!
                            debug!("Parser error in line before: '{}', with value '{}'", line, v);
                        }
                    };

                    // begin new entry
                    if is_end_of_para { None } else { Some(line) }
                },
                (Some(v), true, false) => Some(v + &line),
                (None, _, false) => Some(line),
                (_, _, true) => None,
            };

            // Possibly terminate the current paragraph and append it
            // to the main structure.
            if is_end_of_para && cur_para.entries.len() > 0 {
                paragraphs.push(cur_para);
                cur_para = ControlParagraph::new();
            }

            // Loop termination condition
            if is_eof { break; }
        }
        
        Ok(ControlFile { paragraphs: paragraphs })
    }

    pub fn serialize(&self, out_file: &Path) -> io::Result<()> {
        let mut file = match File::create(out_file) {
            Ok(f) => f,
            Err(e) => return Err(e)
        };

        for para in self.paragraphs.iter() {
            for entry in para.entries.iter() {
                let v = match entry.value.clone() {
                    ControlValue::Simple(v) => v,
                    ControlValue::Folded(v) => v,
                    ControlValue::MultiLine(v) => v,
                };
                let s = entry.key.clone() + ": " + &v + "\n";
                try!(file.write(s.as_bytes()));
            }
            try!(file.write("\n".as_bytes()));
        }

        Ok(())
    }

    pub fn get_paragraphs(&self) -> &Vec<ControlParagraph> {
        &self.paragraphs
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum VRel {
    GreaterOrEqual,
    Greater,
    LesserOrEqual,
    Lesser,
    Equal,
}

impl fmt::Display for VRel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &VRel::GreaterOrEqual => write!(f, ">="),
            &VRel::Greater => write!(f, ">>"),
            &VRel::LesserOrEqual => write!(f, "<="),
            &VRel::Lesser => write!(f, "<<"),
            &VRel::Equal => write!(f, "=")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SingleDependency {
    pub package: String,
    pub version: Option<(VRel, Version)>,
    pub arch: Option<String>
}

impl fmt::Display for SingleDependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (&self.version, &self.arch) {
            (&None, &None) => write!(f, "{}", self.package),
            (&Some((ref vrel, ref ver)), &None) =>
                write!(f, "{} ({} {})", self.package, vrel, ver),
            (&None, &Some(ref a)) => write!(f, "{} [{}]", self.package, a),
            (&Some((ref vrel, ref ver)), &Some(ref a)) =>
                write!(f, "{} ({} {}) [{}]", self.package,
                                        vrel, ver, a)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Dependency {
    pub alternatives: Vec<SingleDependency>
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let alts = self.alternatives.iter().map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .connect(" | ");
        write!(f, "{}", alts)
    }
}

fn parse_single_dep(s: &str) -> Result<SingleDependency, &'static str> {
    enum ST {
        PackageName,
        PreVersion,
        InVersionRel,
        InVersionDef,
        PreArch,
        InArch,
        Done
    }
    let mut st = ST::PackageName;
    let mut result = SingleDependency {
        package: "".to_string(),
        version: None,
        arch: None
    };
    let mut vrel = "".to_string();
    let mut vdef = "".to_string();
    let mut arch = "".to_string();
    for ch in s.chars() {
        match st {
            ST::PackageName => {
                if ch.is_whitespace() { st = ST::PreVersion; }
                else if ch == '(' { st = ST::InVersionRel; }
                else { result.package.push(ch); }
            },
            ST::PreVersion => {
                if ch.is_whitespace() { }
                else if ch == '(' { st = ST::InVersionRel; }
                else { return Err("garbage after package name"); }
            },
            ST::InVersionRel => {
                if ch == '>' || ch == '<' || ch == '=' { vrel.push(ch); }
                else if ch == ')' { return Err("no version given"); }
                else {
                    st = ST::InVersionDef;
                    vdef.push(ch);
                }
            },
            ST::InVersionDef => {
                if ch == ')' {
                    let version = match Version::parse(vdef.trim()) {
                        Ok(v) => v,
                        Err(_) => return Err("error parsing version")
                    };
                    result.version = match &vrel[..] {
                        ">=" | ">" => Some((VRel::GreaterOrEqual, version)),
                        ">>" => Some((VRel::Greater, version)),
                        "<=" | "<" => Some((VRel::LesserOrEqual, version)),
                        "<<" => Some((VRel::Lesser, version)),
                        "=" => Some((VRel::Equal, version)),
                        _ => return Err("invalid relation")
                    };
                    st = ST::PreArch;
                } else { vdef.push(ch); }
            },
            ST::PreArch => {
                if ch.is_whitespace() { }
                else if ch == '[' { st = ST::InArch; }
                else { return Err("garbage after version"); }
            },
            ST::InArch => {
                if ch == ']' {
                    let arch = arch.trim().to_string();
                    if arch.len() > 0 { result.arch = Some(arch); }
                    else { return Err("empty arch given"); }
                    st = ST::Done;
                }
                else { arch.push(ch); }
            },
            ST::Done => {
                if ch.is_whitespace() { }
                else { return Err("garbage after arch"); }
            }
        }
    }
    return Ok(result);
}

pub fn parse_dep_list(s: &str) -> Result<Vec<Dependency>, &'static str> {
    let mut result = vec![];
    for s in s.split(',').map(|x| x.trim()) {
        let mut a = vec![];
        for sd in s.split('|').map(|x| x.trim()) {
            a.push(match parse_single_dep(sd) {
                Ok(v) => v,
                Err(e) => return Err(e)
            });
        }
        result.push(Dependency { alternatives: a });
    }
    return Ok(result);
}

