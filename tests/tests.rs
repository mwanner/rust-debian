extern crate debian;
#[macro_use]
extern crate log;
extern crate tempdir;

use std::env;
use std::path::PathBuf;

use tempdir::TempDir;

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
    let root = TempDir::new("control-file-cycle");
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
