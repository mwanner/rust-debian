use std::str::FromStr;

use debian::Version;

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
    assert!(
        Version::parse("7:1.8-0~bpo2").unwrap()
            < Version::parse("8:1.8-0~bpo2").unwrap()
    );
    assert!(
        Version::parse("8:1.8-0~bpo1").unwrap()
            < Version::parse("8:1.8-0~bpo2").unwrap()
    );
    assert!(
        Version::parse("8:1.8-0~bpo2").unwrap()
            < Version::parse("8:1.8-1~bpo2").unwrap()
    );
    assert!(
        Version::parse("8:1.7-0~bpo2").unwrap()
            < Version::parse("8:1.8-0~bpo2").unwrap()
    );
    assert!(
        Version::parse("8:0.8-0~bpo2").unwrap()
            < Version::parse("8:1.8-0~bpo2").unwrap()
    );

    assert!(
        Version::parse("9:1.8-0~bpo2").unwrap()
            > Version::parse("8:1.8-0~bpo2").unwrap()
    );
    assert!(
        Version::parse("8:1.8-0~bpo3").unwrap()
            > Version::parse("8:1.8-0~bpo2").unwrap()
    );
    assert!(
        Version::parse("8:1.8-2~bpo2").unwrap()
            > Version::parse("8:1.8-1~bpo2").unwrap()
    );
    assert!(
        Version::parse("8:1.9-0~bpo2").unwrap()
            > Version::parse("8:1.8-0~bpo2").unwrap()
    );
    assert!(
        Version::parse("8:2.8-0~bpo2").unwrap()
            > Version::parse("8:1.8-0~bpo2").unwrap()
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
