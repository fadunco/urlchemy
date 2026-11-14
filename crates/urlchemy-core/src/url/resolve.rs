use super::{CompiledTemplate, FixedBase, ResolvedUrl, Token, dot_segment};
use crate::{
    diagnostic::Diagnostic,
    value::{CaptureName, CapturedValue},
};
use std::collections::BTreeMap;

pub(super) fn encode_capture(text: &str) -> String {
    const HEX: &[u8] = b"0123456789ABCDEF";
    let mut result = String::new();
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            result.push(char::from(b));
        } else {
            result.push('%');
            result.push(char::from(HEX[usize::from(b >> 4)]));
            result.push(char::from(HEX[usize::from(b & 15)]));
        }
    }
    result
}
pub(crate) fn resolve(
    base: &FixedBase,
    template: &CompiledTemplate,
    captures: &BTreeMap<CaptureName, CapturedValue>,
) -> Result<ResolvedUrl, Diagnostic> {
    let error = |code| Diagnostic::new(code, &template.origin);
    let render = |ts: &[Token]| -> Result<String, Diagnostic> {
        let mut result = String::new();
        for token in ts {
            match token {
                Token::Literal(s) => result.push_str(s),
                Token::Capture(name) => result.push_str(&encode_capture(
                    &captures
                        .get(name)
                        .ok_or_else(|| error("E_CAPTURE_UNAVAILABLE"))?
                        .0,
                )),
            }
        }
        Ok(result)
    };
    let mut path = base.url.path().to_string();
    if let Some(segments) = &template.path {
        let mut rendered = Vec::new();
        for segment in segments {
            let s = render(segment)?;
            if dot_segment(&s) {
                return Err(error("E_RESOLVE_DOT_SEGMENT"));
            }
            rendered.push(s);
        }
        if path.ends_with('/') {
            path.pop();
        }
        path.push('/');
        path.push_str(&rendered.join("/"));
    }
    let query = template
        .query
        .as_ref()
        .map(|pairs| {
            pairs
                .iter()
                .map(|(k, ts)| Ok(format!("{k}={}", render(ts)?)))
                .collect::<Result<Vec<_>, Diagnostic>>()
                .map(|p| p.join("&"))
        })
        .transpose()?;
    let prefix = &base.url[..::url::Position::BeforePath];
    let mut text = format!("{prefix}{path}");
    if let Some(q) = &query {
        text.push('?');
        text.push_str(q);
    }
    let parsed = ::url::Url::parse(&text).map_err(|_| error("E_RESOLVE_STRUCTURE"))?;
    if &parsed[..::url::Position::BeforePath] != prefix
        || parsed.path() != path
        || parsed.query() != query.as_deref()
        || parsed.fragment().is_some()
        || parsed.as_str() != text
    {
        return Err(error("E_RESOLVE_STRUCTURE"));
    }
    Ok(ResolvedUrl(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        diagnostic::Origin,
        url::{base::parse_base, template::compile_template},
        value::{CaptureName, CapturedValue},
    };
    use std::{collections::BTreeMap, path::Path};
    #[test]
    fn encoding_and_prefix() {
        assert_eq!(
            encode_capture("A+B & café/ops"),
            "A%2BB%20%26%20caf%C3%A9%2Fops"
        );
        assert_eq!(encode_capture("%2F"), "%252F");
        assert_eq!(encode_capture("~._-"), "~._-");
        let o = Origin::file(Path::new("f"));
        for (base, path, expected) in [
            (
                "https://example.com/company",
                "/dashboard",
                "https://example.com/company/dashboard",
            ),
            (
                "https://example.com/company/",
                "/dashboard",
                "https://example.com/company/dashboard",
            ),
            (
                "https://example.com/company",
                "/",
                "https://example.com/company/",
            ),
            (
                "https://example.com/company/",
                "",
                "https://example.com/company/",
            ),
            (
                "https://example.com/company",
                "?view=summary",
                "https://example.com/company?view=summary",
            ),
            (
                "https://example.com/company//",
                "/dashboard",
                "https://example.com/company//dashboard",
            ),
        ] {
            assert_eq!(
                resolve(
                    &parse_base(base, &o).unwrap(),
                    &compile_template(path, &o).unwrap(),
                    &BTreeMap::new()
                )
                .unwrap()
                .as_str(),
                expected
            );
        }
        let base = parse_base("https://example.com/company", &o).unwrap();
        let mut captures = BTreeMap::new();
        captures.insert(CaptureName("x".into()), CapturedValue("..".into()));
        assert_eq!(
            resolve(&base, &compile_template("/{x}", &o).unwrap(), &captures)
                .unwrap_err()
                .code(),
            "E_RESOLVE_DOT_SEGMENT"
        );
        assert_eq!(
            resolve(&base, &compile_template("?q={x}", &o).unwrap(), &captures)
                .unwrap()
                .as_str(),
            "https://example.com/company?q=.."
        );
        captures.insert(CaptureName("x".into()), CapturedValue(".".into()));
        assert_eq!(
            resolve(&base, &compile_template("/.{x}", &o).unwrap(), &captures)
                .unwrap_err()
                .code(),
            "E_RESOLVE_DOT_SEGMENT"
        );
        captures.insert(CaptureName("x".into()), CapturedValue("{x}".into()));
        assert_eq!(
            resolve(
                &base,
                &compile_template("/%7B/{x}?q=&q=%2f", &o).unwrap(),
                &captures
            )
            .unwrap()
            .as_str(),
            "https://example.com/company/%7B/%7Bx%7D?q=&q=%2f"
        );
    }
}
