mod compile;
mod file;
pub use file::{configuration_path, load_file};
mod raw;
mod scalar;
mod schema;
mod source;
mod yaml;
use crate::{Catalog, Diagnostics};
use std::path::Path;

pub fn load_bytes(source: &Path, bytes: &[u8]) -> Result<Catalog, Diagnostics> {
    compile::compile(schema::deserialize(yaml::parse_yaml(source, bytes)?)?)
}
