use std::cmp::Ordering;
use std::fmt;
use std::num::from_str_radix;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct VersionPart {
    pub alpha: String,
    pub numeric: u64
}

impl Ord for VersionPart {
    fn cmp(&self, other: &VersionPart) -> Ordering {
        assert!(self.alpha.len() == 0);
        assert!(other.alpha.len() == 0);
        // FIXME: compare alpha, first!
        self.numeric.cmp(&other.numeric)
    }
}

impl fmt::Display for VersionPart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.alpha, self.numeric)
    }
}

impl fmt::Display for Vec<VersionPart> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let parts = self.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        write!(f, "{}", parts.concat())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Version {
    pub epoch: i32,
    pub upstream_version: Vec<VersionPart>,
    pub debian_revision: Vec<VersionPart>
}

#[derive(Debug)]
pub struct ParseError {
    pub pos: i32,
    pub msg: String
}

impl Version {
    pub fn parse_parts(s: &str) -> Result<Vec<VersionPart>, ParseError> {
        if s.len() == 0 {
            return Ok(vec![]);
        }
        let mut result : Vec<VersionPart> = vec![];
        let mut in_numeric_part = false;
        let mut cur = VersionPart { alpha: "".to_string(), numeric: 0 };
        for c in s.chars() {
            match (in_numeric_part, c.is_digit(10)) {
                (false, false) => cur.alpha.push(c),
                (_, true) => {
                    in_numeric_part = true;
                    cur.numeric *= 10;
                    cur.numeric += (c as u64) - ('0' as u64);
                }
                (true, false) => {
                    result.push(cur);
                    in_numeric_part = false;
                    cur = VersionPart { alpha: "".to_string(), numeric: 0 };
                    cur.alpha.push(c);
                }
            }
        }
        result.push(cur);
        Ok(result)
    }

    pub fn parse(s: &str) -> Result<Version, ParseError> {
        let first_colon = s.find(':');
        let last_dash = s.rfind('-');

        let epoch = match first_colon {
            Some(l) => {
                let epoch_str = &s[..l];
                match from_str_radix(epoch_str, 10) {
                    Ok(v) => Some((l, v)),
                    Err(_) => return Err(ParseError { pos: 0,
                        msg: "Expected a numeric epoch.".to_string() })
                }
            },
            None => None
        };

        Ok(match (epoch, last_dash) {
            (Some((l, epoch)), Some(r)) => Version {
                epoch: epoch,
                upstream_version: try!(Version::parse_parts(&s[l+1..r])),
                debian_revision: try!(Version::parse_parts(&s[r+1..]))
            },
            (Some((l, epoch)), None) => Version {
                epoch: epoch,
                upstream_version: try!(Version::parse_parts(&s[l+1..])),
                debian_revision: vec![],
            },
            (None, Some(r)) => Version {
                epoch: 0,
                upstream_version: try!(Version::parse_parts(&s[..r])),
                debian_revision: try!(Version::parse_parts(&s[r+1..]))
            },
            (None, None) => Version {
                epoch: 0,
                upstream_version: try!(Version::parse_parts(&s[..])),
                debian_revision: vec![]
            }
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        let epoch_cmp = self.epoch.cmp(&other.epoch);
        match epoch_cmp {
            Ordering::Equal => Ordering::Equal,
            ord => ord
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.epoch, self.debian_revision.len()) {
            (0, 0) => write!(f, "{}", self.upstream_version),
            (0, _) => write!(f, "{}-{}", self.upstream_version, self.debian_revision),
            (_, 0) => write!(f, "{}:{}", self.epoch, self.upstream_version),
            (_, _) => write!(f, "{}:{}-{}", self.epoch, self.upstream_version, self.debian_revision)
        }
    }
}
