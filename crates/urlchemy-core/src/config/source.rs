use crate::diagnostic::{Origin, SourceLocation};
use std::path::Path;

pub(super) struct SourceMap {
    locations: Vec<(usize, SourceLocation)>,
    origin: Origin,
}
impl SourceMap {
    pub fn new(source: &Path, text: &str, bom: usize) -> Self {
        let mut line = 1;
        let mut column = 1;
        let mut cr = false;
        let mut locations = Vec::new();
        for (byte, c) in text.char_indices() {
            locations.push((byte + bom, SourceLocation { line, column }));
            if c == '\r' {
                line += 1;
                column = 1;
            } else if c == '\n' {
                if !cr {
                    line += 1;
                }
                column = 1;
            } else {
                column += 1;
            }
            cr = c == '\r';
        }
        locations.push((text.len() + bom, SourceLocation { line, column }));
        Self {
            locations,
            origin: Origin::file(source),
        }
    }
    pub fn origin(&self, index: usize) -> Origin {
        let (offset, location) = self.locations[index.min(self.locations.len() - 1)];
        Origin {
            offset,
            location: Some(location),
            ..self.origin.clone()
        }
    }
}
