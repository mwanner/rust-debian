use std::env;
use std::path::PathBuf;

use log::*;
use tempfile::TempDir;

use debian::control::{parse_dep_list, ControlFile, VRel};
use debian::{Version, VersionElement, VersionPart};

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
    let dl = parse_dep_list(bd).unwrap();

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
    assert_eq!(cf.get_paragraphs().len(), 10, "number of paragraphs");

    let gp = cf.get_paragraphs().get(0).unwrap();
    assert_eq!(gp.get_entry("Source").unwrap(), "postgis");
}

#[test]
fn control_file_fbautostart() {
    setup();

    let path = data_path().join("control-fbautostart");
    let cf = ControlFile::from_file(&path).unwrap();
    assert_eq!(cf.get_paragraphs().len(), 2, "number of paragraphs");

    let source = cf.get_paragraphs().get(0).unwrap();
    assert_eq!(
        source.get_entry("Maintainer").unwrap(),
        "Paul Tagliamonte <paultag@ubuntu.com>"
    );
    assert_eq!(source.get_entry("Source").unwrap(), "fbautostart");

    let bd_str = source.get_entry("Build-Depends").unwrap();
    let bd = parse_dep_list(bd_str).unwrap();
    assert_eq!(bd.len(), 1);
    assert_eq!(bd[0].alternatives.len(), 1);

    let first_dep = &bd[0].alternatives[0];
    assert_eq!(first_dep.package, "debhelper");
    assert_eq!(
        first_dep.version,
        Some((VRel::GreaterOrEqual, Version::parse("9").unwrap()))
    );

    let package = cf.get_paragraphs().get(1).unwrap();
    assert_eq!(package.get_entry("Package").unwrap(), "fbautostart");
    assert_eq!(package.get_entry("Architecture").unwrap(), "any");
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
