use super::{CompiledTemplate, Token, dot_segment};
use crate::{
    diagnostic::{Diagnostic, Origin},
    value::{CaptureName, valid_name},
};

fn tokens(text: &str, origin: &Origin) -> Result<Vec<Token>, Diagnostic> {
    let error = || Diagnostic::new("E_TEMPLATE_SYNTAX", origin);
    let mut result = Vec::new();
    let mut literal = String::new();
    let mut i = 0;
    let bytes = text.as_bytes();
    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                if !literal.is_empty() {
                    result.push(Token::Literal(std::mem::take(&mut literal)));
                }
                let end = text[i + 1..]
                    .find('}')
                    .map(|n| n + i + 1)
                    .ok_or_else(error)?;
                let name = &text[i + 1..end];
                if !valid_name(name) {
                    return Err(error());
                }
                result.push(Token::Capture(CaptureName(name.into())));
                i = end + 1;
            }
            b'%' if i + 2 < bytes.len()
                && bytes[i + 1].is_ascii_hexdigit()
                && bytes[i + 2].is_ascii_hexdigit() =>
            {
                literal.push_str(&text[i..i + 3]);
                i += 3;
            }
            b if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') => {
                literal.push(char::from(b));
                i += 1;
            }
            _ => return Err(error()),
        }
    }
    if !literal.is_empty() {
        result.push(Token::Literal(literal));
    }
    Ok(result)
}

pub(crate) fn compile_template(
    text: &str,
    origin: &Origin,
) -> Result<CompiledTemplate, Diagnostic> {
    let error = || Diagnostic::new("E_TEMPLATE_SYNTAX", origin);
    if !text.is_ascii() {
        return Err(error());
    }
    let (path_text, query_text) = text
        .split_once('?')
        .map_or((text, None), |(p, q)| (p, Some(q)));
    let path = if path_text.is_empty() {
        None
    } else {
        if !path_text.starts_with('/') || path_text.starts_with("//") {
            return Err(error());
        }
        let segments = path_text[1..]
            .split('/')
            .map(|s| {
                let ts = tokens(s, origin)?;
                if ts.iter().all(|t| matches!(t, Token::Literal(_))) && dot_segment(s) {
                    return Err(error());
                }
                Ok(ts)
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        Some(segments)
    };
    let query = query_text
        .map(|q| {
            q.split('&')
                .map(|pair| {
                    let (key, value) = pair.split_once('=').ok_or_else(error)?;
                    let key_tokens = tokens(key, origin)?;
                    if key.is_empty() || key_tokens.iter().any(|t| matches!(t, Token::Capture(_))) {
                        return Err(error());
                    }
                    Ok((key.into(), tokens(value, origin)?))
                })
                .collect::<Result<Vec<_>, Diagnostic>>()
        })
        .transpose()?;
    let mut references = Vec::new();
    for ts in path
        .iter()
        .flatten()
        .chain(query.iter().flatten().map(|(_, ts)| ts))
    {
        for t in ts {
            if let Token::Capture(name) = t {
                references.push(name.clone());
            }
        }
    }
    Ok(CompiledTemplate {
        path,
        query,
        references,
        origin: origin.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Origin;
    use std::path::Path;
    #[test]
    fn template_grammar() {
        let origin = Origin::file(Path::new("f"));
        for s in [
            "",
            "?view=summary",
            "/ticket/APP-{ticket_number}",
            "/%7B?x=&x={value}",
            "/a//b/",
        ] {
            assert!(compile_template(s, &origin).is_ok(), "{s}");
        }
        for s in [
            "//host",
            "relative",
            "/#fragment",
            "/café",
            "/{bad",
            "/{{x}}",
            "/%Q1",
            "/?{key}=x",
            "/?bare",
            "/?x=a=b",
            "/?x=x&",
            "/?",
            "/.%2E",
            "/{123}",
        ] {
            assert_eq!(
                compile_template(s, &origin).unwrap_err().code(),
                "E_TEMPLATE_SYNTAX",
                "{s}"
            );
        }
    }
}
