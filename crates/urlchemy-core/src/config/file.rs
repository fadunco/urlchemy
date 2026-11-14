use super::{load_bytes, yaml::BYTE_LIMIT};
use crate::{Catalog, Diagnostic, Diagnostics, diagnostic::Origin};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

pub fn configuration_path(home: &Path) -> PathBuf {
    home.join("Library/Application Support/URLchemy/config.yaml")
}
pub fn load_file(path: &Path) -> Result<Catalog, Diagnostics> {
    let file = File::open(path).map_err(|e| {
        Diagnostics::from(Diagnostic::new(
            if e.kind() == io::ErrorKind::NotFound {
                "E_CONFIG_MISSING"
            } else {
                "E_CONFIG_READ"
            },
            &Origin::file(path),
        ))
    })?;
    load_reader(path, file)
}
fn load_reader(source: &Path, reader: impl Read) -> Result<Catalog, Diagnostics> {
    let mut bytes = Vec::new();
    reader
        .take((BYTE_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| Diagnostics::from(Diagnostic::new("E_CONFIG_READ", &Origin::file(source))))?;
    load_bytes(source, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Read};
    struct Failing;
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }
    #[test]
    fn read_failures_are_atomic() {
        let reader = io::Cursor::new(b"navigation: []\n").chain(Failing);
        let error = load_reader(std::path::Path::new("fixture"), reader).unwrap_err();
        assert_eq!(error.items()[0].code(), "E_CONFIG_READ");
        assert_eq!(error.items()[0].location(), None);
    }
}
