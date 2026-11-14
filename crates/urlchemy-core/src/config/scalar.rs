use yaml_rust2::scanner::TScalarStyle;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScalarKind {
    String,
    Null,
    Boolean,
    Integer,
    Float,
}

pub(super) fn classify_scalar(text: &str, style: TScalarStyle) -> ScalarKind {
    if style != TScalarStyle::Plain {
        return ScalarKind::String;
    }
    if matches!(text, "" | "~" | "null" | "Null" | "NULL") {
        return ScalarKind::Null;
    }
    if matches!(text, "true" | "True" | "TRUE" | "false" | "False" | "FALSE") {
        return ScalarKind::Boolean;
    }
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if digits(unsigned)
        || text
            .strip_prefix("0o")
            .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| matches!(b, b'0'..=b'7')))
        || text
            .strip_prefix("0x")
            .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return ScalarKind::Integer;
    }
    if matches!(unsigned, ".inf" | ".Inf" | ".INF") || matches!(text, ".nan" | ".NaN" | ".NAN") {
        return ScalarKind::Float;
    }
    let (mantissa, exponent) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, None), |(a, b)| (a, Some(b)));
    let exponent_ok = exponent.is_none_or(|e| digits(e.strip_prefix(['+', '-']).unwrap_or(e)));
    let decimal = mantissa.split_once('.').is_some_and(|(a, b)| {
        (digits(a) && (b.is_empty() || digits(b))) || (a.is_empty() && digits(b))
    });
    if exponent_ok && (decimal || (exponent.is_some() && digits(mantissa))) {
        ScalarKind::Float
    } else {
        ScalarKind::String
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaml_rust2::scanner::TScalarStyle;

    #[test]
    fn core_scalar_types() {
        for text in ["123", "0o17", "0xFF", &"9".repeat(500)] {
            assert_eq!(
                classify_scalar(text, TScalarStyle::Plain),
                ScalarKind::Integer
            );
        }
        for text in ["1e3", ".5", ".NaN"] {
            assert_eq!(
                classify_scalar(text, TScalarStyle::Plain),
                ScalarKind::Float
            );
        }
        for text in ["true", "FALSE"] {
            assert_eq!(
                classify_scalar(text, TScalarStyle::Plain),
                ScalarKind::Boolean
            );
        }
        for text in ["null", "~", ""] {
            assert_eq!(classify_scalar(text, TScalarStyle::Plain), ScalarKind::Null);
        }
        for text in ["yes", "on", "0b101", "1_000", "2026-11-14"] {
            assert_eq!(
                classify_scalar(text, TScalarStyle::Plain),
                ScalarKind::String
            );
        }
        for text in ["123", "true", ""] {
            assert_eq!(
                classify_scalar(text, TScalarStyle::DoubleQuoted),
                ScalarKind::String
            );
        }
    }
}
