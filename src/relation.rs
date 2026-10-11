//! Relationships between packages
//!
//! This module contains a parser for the values of relationship fields
//! like `Depends`, `Build-Depends` or `Conflicts`, as specified in the
//! Debian Policy Manual, section 7.1 "Syntax of relationship fields".

use std::fmt;

use super::Version;

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
            .map(|x| format!("{x}"))
            .collect::<Vec<String>>()
            .join(" | ");
        write!(f, "{alts}")
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
            a.push(parse_single_dep(sd)?);
        }
        result.push(Dependency { alternatives: a });
    }
    Ok(result)
}
