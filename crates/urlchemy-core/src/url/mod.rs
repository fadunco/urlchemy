mod base;
mod resolve;
mod template;
use crate::diagnostic::Origin;
pub(crate) use base::parse_base;
pub(crate) use resolve::resolve;
use std::fmt;
pub(crate) use template::compile_template;

#[derive(Clone, Debug)]
pub(crate) struct FixedBase {
    pub url: ::url::Url,
}
#[derive(Clone, Debug)]
pub(crate) enum Token {
    Literal(String),
    Capture(crate::value::CaptureName),
}
#[derive(Clone, Debug)]
pub(crate) struct CompiledTemplate {
    pub path: Option<Vec<Vec<Token>>>,
    pub query: Option<Vec<(String, Vec<Token>)>>,
    pub references: Vec<crate::value::CaptureName>,
    pub origin: Origin,
}
#[derive(Clone, PartialEq, Eq)]
pub struct ResolvedUrl(pub(crate) String);
impl ResolvedUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Debug for ResolvedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ResolvedUrl([redacted])")
    }
}

pub(super) fn dot_segment(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut decoded = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(v) = u8::from_str_radix(&text[i + 1..i + 3], 16)
        {
            decoded.push(v);
            i += 3;
            continue;
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    decoded == b"." || decoded == b".."
}
pub(super) fn valid_escapes(text: &str) -> bool {
    let mut iter = text.bytes();
    while let Some(b) = iter.next() {
        if b == b'%'
            && !(iter.next().is_some_and(|v| v.is_ascii_hexdigit())
                && iter.next().is_some_and(|v| v.is_ascii_hexdigit()))
        {
            return false;
        }
    }
    true
}
