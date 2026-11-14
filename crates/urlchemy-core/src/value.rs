use std::fmt;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CaptureName(pub String);
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CapturedValue(pub String);
impl fmt::Debug for CapturedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CapturedValue([redacted])")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextError {
    Required,
    Control,
}
pub(crate) fn check_text(text: &str) -> Result<(), TextError> {
    if text.is_empty() {
        return Err(TextError::Required);
    }
    if text
        .chars()
        .any(|c| matches!(c, '\u{0}'..='\u{1f}' | '\u{7f}'..='\u{9f}' | '\u{2028}' | '\u{2029}'))
    {
        return Err(TextError::Control);
    }
    if text.chars().all(|c| matches!(c, '\u{9}'..='\u{d}' | '\u{20}' | '\u{85}' | '\u{a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}')) { return Err(TextError::Required); }
    Ok(())
}
pub(crate) fn valid_name(s: &str) -> bool {
    let mut bytes = s.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}
pub(crate) fn valid_alias(s: &str) -> bool {
    let mut bytes = s.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn required_text_contract() {
        assert_eq!(check_text(""), Err(TextError::Required));
        assert_eq!(check_text(" \u{00a0} "), Err(TextError::Required));
        assert_eq!(check_text("jobs\narchive"), Err(TextError::Control));
        assert_eq!(check_text("\t"), Err(TextError::Control));
        assert_eq!(check_text(" accounts "), Ok(()));
        assert_eq!(check_text("café/ops & SRE"), Ok(()));
        for c in (0..=0x9f).filter_map(char::from_u32) {
            if c <= '\u{1f}' || c >= '\u{7f}' {
                assert_eq!(check_text(&format!("x{c}")), Err(TextError::Control));
            }
        }
        for c in [
            '\u{a0}', '\u{1680}', '\u{2000}', '\u{200a}', '\u{202f}', '\u{205f}', '\u{3000}',
        ] {
            assert_eq!(check_text(&c.to_string()), Err(TextError::Required));
        }
        assert!(valid_name("app-section"));
        assert!(!valid_name("123"));
        assert!(valid_alias("open-dd"));
        assert!(!valid_alias("Open-dd"));
    }
}
