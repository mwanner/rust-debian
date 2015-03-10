use std::ascii::AsciiExt;
use std::fmt;
use std::num::from_str_radix;

#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub epoch: i32,
    pub upstream_version: String,
    pub debian_revision: String,
}

pub struct ParseError {
    pos: i32,
    msg: String
}

impl Version {
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
                upstream_version: s[l+1..r].to_string(),
                debian_revision: s[r+1..].to_string()
            },
            (Some((l, epoch)), None) => Version {
                epoch: epoch,
                upstream_version: s[l+1..].to_string(),
                debian_revision: "".to_string()
            },
            (None, Some(r)) => Version {
                epoch: 0,
                upstream_version: s[..r].to_string(),
                debian_revision: s[r+1..].to_string()
            },
            (None, None) => Version {
                epoch: 0,
                upstream_version: s.to_string(),
                debian_revision: "".to_string()
            }
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.epoch, self.debian_revision.len()) {
            (0, 0) => write!(f, "{}", self.upstream_version),
            (0, _) => write!(f, "{}-{}", self.upstream_version,
                             self.debian_revision),
            (_, 0) => write!(f, "{}:{}", self.epoch, self.upstream_version),
            (_, _) => write!(f, "{}:{}-{}", self.epoch, self.upstream_version,
                             self.debian_revision)
        }
    }
}
