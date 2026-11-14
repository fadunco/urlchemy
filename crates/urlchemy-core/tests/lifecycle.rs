#[path = "support/scratch.rs"]
mod scratch;
use scratch::Scratch;
use std::path::{Path, PathBuf};
use urlchemy_core::config::{configuration_path, load_file};

#[test]
fn configuration_path_is_explicit() {
    assert_eq!(
        configuration_path(Path::new("/fixture-home")),
        PathBuf::from("/fixture-home/Library/Application Support/URLchemy/config.yaml")
    );
}
#[test]
fn missing_and_unreadable_files_are_diagnostics() {
    let dir = Scratch::new();
    let e = load_file(&dir.file()).unwrap_err();
    assert_eq!(e.items()[0].code(), "E_CONFIG_MISSING");
    assert_eq!(e.items()[0].location(), None);
    let e = load_file(dir.root()).unwrap_err();
    assert_eq!(e.items()[0].code(), "E_CONFIG_READ");
    assert_eq!(e.items()[0].location(), None);
    assert_eq!(std::fs::read_dir(dir.root()).unwrap().count(), 0);
}
#[test]
fn bounded_read_and_no_mutation() {
    let dir = Scratch::new();
    let path = dir.file();
    let mut bytes = b"navigation: []\n#".to_vec();
    bytes.resize(1_048_576, b'x');
    std::fs::write(&path, &bytes).unwrap();
    assert!(load_file(&path).is_ok());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    bytes.push(b'x');
    std::fs::write(&path, &bytes).unwrap();
    assert_eq!(
        load_file(&path).unwrap_err().items()[0].code(),
        "E_CONFIG_LIMIT"
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let valid = include_bytes!("../../../docs/superpowers/specs/yaml-url-contract/fixed.yaml");
    std::fs::write(&path, valid).unwrap();
    let catalog = load_file(&path).unwrap();
    std::fs::write(&path, "invalid: [").unwrap();
    assert!(load_file(&path).is_err());
    assert!(catalog.flow_by_key("fixed").is_some());
    assert_eq!(std::fs::read_dir(dir.root()).unwrap().count(), 1);
}
