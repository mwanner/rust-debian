extern crate debian;
#[macro_use]
extern crate log;
extern crate tempfile;

use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use tempfile::TempDir;

use debian::package::{parse_dep_list, ControlFile, VRel};
use debian::version::{Version, VersionElement, VersionPart};

fn data_path() -> PathBuf {
    // Not sure what the best way is - this works when invoked from cargo.
    let path = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(path_str) => PathBuf::from(path_str),
        None => panic!(),
    };
    println!("Path: {}", path.display());
    // ..while this obviously didn't work:
    // let mut path = env::current_exe().unwrap().parent().unwrap();
    path.join("tests").join("data")
}

fn setup() {
    let root = TempDir::new();
    let root = root.expect("Should have created a temp directory.");
    assert!(env::set_current_dir(root.path()).is_ok());
    debug!(
        "path setup; root={}, data={}",
        root.path().display(),
        data_path().display()
    );
}

#[test]
fn control_file_foo() {
    setup();

    let path = data_path().join("control-foo");

    let cf = ControlFile::from_file(&path).unwrap();
    assert!(cf.get_paragraphs().len() == 2);

    let gp = cf.get_paragraphs().get(0).unwrap();
    assert_eq!(gp.get_entry("Source").unwrap(), "foo");

    let bd = gp.get_entry("Build-Depends").unwrap();
    let dl = debian::package::parse_dep_list(bd).unwrap();

    let libbluetooth = dl.get(1).unwrap().alternatives.get(0).unwrap();
    assert_eq!(libbluetooth.arch.as_ref().unwrap(), "linux-any");
    assert_eq!(libbluetooth.condition.as_ref().unwrap(), "!stage1");

    let xvfb = dl.get(2).unwrap().alternatives.get(0).unwrap();
    assert_eq!(xvfb.condition.as_ref().unwrap(), "!nocheck");
    assert_eq!(xvfb.arch, None);
}

#[test]
fn control_file_postgis() {
    setup();

    let path = data_path().join("control-postgis");
    let cf = ControlFile::from_file(&path).unwrap();
    assert!(cf.get_paragraphs().len() == 10);

    let gp = cf.get_paragraphs().get(0).unwrap();
    assert_eq!(gp.get_entry("Source").unwrap(), "postgis");
}

#[test]
fn version_basics() {
    let v = Version::parse("7:2.1.4-0~bpo2").unwrap();
    assert_eq!(v.epoch, 7);
    assert_eq!(&v.upstream_version.to_string(), "2.1.4");
    assert_eq!(&v.upstream_version.to_string(), "2.1.4");
    assert_eq!(&v.debian_revision.to_string(), "0~bpo2");
    assert_eq!(Version::from_str("7:2.1.4-0~bpo2").unwrap(), v);

    let v = Version::parse("2.1.4-0~bpo2").unwrap();
    assert_eq!(v.epoch, 0);
    assert_eq!(&v.upstream_version.to_string(), "2.1.4");
    assert_eq!(&v.debian_revision.to_string(), "0~bpo2");

    let v = Version::parse("7:2.1.4").unwrap();
    assert_eq!(v.epoch, 7);
    assert_eq!(v.upstream_version.to_string(), "2.1.4");
    assert_eq!(v.debian_revision.to_string(), "");

    let v = Version::parse("2.1.4").unwrap();
    assert_eq!(v.epoch, 0);
    assert_eq!(v.upstream_version.to_string(), "2.1.4");
    assert_eq!(v.debian_revision.to_string(), "");

    let v = Version::parse("1:1:1-8-8").unwrap();
    assert_eq!(v.epoch, 1);
    assert_eq!(v.upstream_version.to_string(), "1:1-8");
    assert_eq!(v.debian_revision.to_string(), "8");
}

#[test]
fn version_comparisons() {
    let v = Version::parse("7:2.1.4-0~bpo2").unwrap();
    assert!(v < Version::parse("8:1.8-0~bpo2").unwrap());

    assert_eq!(
        Version::parse("0:1.0").unwrap(),
        Version::parse("1.0").unwrap()
    );
    assert!(Version::parse("7:1.8-0~bpo2").unwrap() < Version::parse("8:1.8-0~bpo2").unwrap());
    assert!(Version::parse("8:1.8-0~bpo1").unwrap() < Version::parse("8:1.8-0~bpo2").unwrap());
    assert!(Version::parse("8:1.8-0~bpo2").unwrap() < Version::parse("8:1.8-1~bpo2").unwrap());
    assert!(Version::parse("8:1.7-0~bpo2").unwrap() < Version::parse("8:1.8-0~bpo2").unwrap());
    assert!(Version::parse("8:0.8-0~bpo2").unwrap() < Version::parse("8:1.8-0~bpo2").unwrap());

    assert!(Version::parse("9:1.8-0~bpo2").unwrap() > Version::parse("8:1.8-0~bpo2").unwrap());
    assert!(Version::parse("8:1.8-0~bpo3").unwrap() > Version::parse("8:1.8-0~bpo2").unwrap());
    assert!(Version::parse("8:1.8-2~bpo2").unwrap() > Version::parse("8:1.8-1~bpo2").unwrap());
    assert!(Version::parse("8:1.9-0~bpo2").unwrap() > Version::parse("8:1.8-0~bpo2").unwrap());
    assert!(Version::parse("8:2.8-0~bpo2").unwrap() > Version::parse("8:1.8-0~bpo2").unwrap());

}

#[test]
fn dependency_basics() {
    let deps = parse_dep_list("foo (>= 3.2) | bar, baz (>= 1)").unwrap();
    assert_eq!(deps.len(), 2);
    assert_eq!(deps[0].alternatives.len(), 2);
    assert_eq!(deps[1].alternatives.len(), 1);
    let sd1 = &deps[0].alternatives[0];
    assert_eq!(&sd1.package[..], "foo");
    assert_eq!(
        sd1.version,
        Some((
            VRel::GreaterOrEqual,
            Version {
                epoch: 0,
                upstream_version: VersionPart {
                    elements: vec![
                        VersionElement {
                            alpha: "".to_string(),
                            numeric: 3
                        },
                        VersionElement {
                            alpha: ".".to_string(),
                            numeric: 2
                        }
                    ]
                },
                debian_revision: VersionPart { elements: vec![] }
            }
        ))
    );
    let sd2 = &deps[0].alternatives[1];
    assert_eq!(&sd2.package[..], "bar");
    assert_eq!(sd2.version, None);
    let sd3 = &deps[1].alternatives[0];
    assert_eq!(&sd3.package[..], "baz");
    assert_eq!(
        sd3.version,
        Some((
            VRel::GreaterOrEqual,
            Version {
                epoch: 0,
                upstream_version: VersionPart {
                    elements: vec![VersionElement {
                        alpha: "".to_string(),
                        numeric: 1
                    }]
                },
                debian_revision: VersionPart { elements: vec![] }
            }
        ))
    );
}

#[cfg(feature = "serde")]
#[test]
fn serde_tests() {
    let data = r#"[
        "8:1.8-0~bpo2",
        "1.8-0",
        "1:1:1-8-8"
    ]"#;
    let versions: Vec<Version> = serde_json::from_str(data).unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0], Version::parse("8:1.8-0~bpo2").unwrap());
    assert_eq!(versions[1], Version::parse("1.8-0").unwrap());
    assert_eq!(versions[2], Version::parse("1:1:1-8-8").unwrap());

    // test serialization
    let ser = serde_json::to_string(&versions).unwrap();
    assert_eq!(ser, r#"["8:1.8-0~bpo2","1.8-0","1:1:1-8-8"]"#);
}
