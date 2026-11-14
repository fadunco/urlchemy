use super::{FixedBase, dot_segment, valid_escapes};
use crate::diagnostic::{Diagnostic, Origin};

pub(crate) fn parse_base(text: &str, origin: &Origin) -> Result<FixedBase, Diagnostic> {
    let error = || Diagnostic::new("E_BASE_URL", origin);
    if !text.is_ascii()
        || text.bytes().any(|b| b <= b' ' || b == 127)
        || text.contains(['\\', '?', '#', '{', '}'])
        || !valid_escapes(text)
    {
        return Err(error());
    }
    let (scheme, rest) = text.split_once("://").ok_or_else(error)?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return Err(error());
    }
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    if authority.is_empty() || authority.contains(['@', '%']) || path.split('/').any(dot_segment) {
        return Err(error());
    }
    let url = ::url::Url::parse(text).map_err(|_| error())?;
    if url.host_str().is_none()
        || url.port() == Some(0)
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(error());
    }
    Ok(FixedBase { url })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Origin;
    use std::path::Path;
    #[test]
    fn strict_bases() {
        let origin = Origin::file(Path::new("f"));
        assert_eq!(
            parse_base("HTTPS://EXAMPLE.COM:443", &origin)
                .unwrap()
                .url
                .as_str(),
            "https://example.com/"
        );
        assert_eq!(
            parse_base("https://[::1]:8443/company/", &origin)
                .unwrap()
                .url
                .as_str(),
            "https://[::1]:8443/company/"
        );
        for s in [
            "https://user:secret@example.com",
            "https:///example.com",
            "https://example.com?",
            "https://example.com#",
            "https://example.com:0",
            "https://example.com:65536",
            "https://example.com/%Q0",
            "https://example.com/café",
            "https://example.com\\x",
            "https://example.com/a/..",
            "https://example.com/.%2e",
            "javascript:alert(1)",
        ] {
            assert_eq!(
                parse_base(s, &origin).unwrap_err().code(),
                "E_BASE_URL",
                "{s}"
            );
        }
    }
}
