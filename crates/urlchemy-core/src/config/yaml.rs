use super::{
    scalar::{ScalarKind, classify_scalar},
    source::SourceMap,
};
use crate::diagnostic::{Diagnostic, Diagnostics, Origin};
use std::collections::BTreeMap;
use std::path::Path;
use yaml_rust2::{
    parser::{Event, Parser},
    scanner::{Scanner, TokenType},
};

pub(crate) const BYTE_LIMIT: usize = 1_048_576;
#[derive(Clone, Debug)]
pub(super) struct Node {
    pub kind: Kind,
    pub origin: Origin,
}
#[derive(Clone, Debug)]
pub(super) enum Kind {
    Scalar(String, ScalarKind),
    Sequence(Vec<Node>),
    Mapping(Vec<(Node, Node)>),
}
#[derive(Debug)]
pub(super) struct YamlDocument {
    pub root: Option<Node>,
    pub origin: Origin,
}
struct Frame {
    origin: Origin,
    mapping: bool,
    children: Vec<Node>,
    keys: BTreeMap<String, Origin>,
}

fn child_pointer(stack: &[Frame]) -> String {
    let Some(frame) = stack.last() else {
        return String::new();
    };
    let component = if frame.mapping {
        if frame.children.len().is_multiple_of(2) {
            return frame.origin.pointer.clone();
        }
        match frame.children.last().map(|n| &n.kind) {
            Some(Kind::Scalar(s, _)) => s.replace('~', "~0").replace('/', "~1"),
            _ => return frame.origin.pointer.clone(),
        }
    } else {
        frame.children.len().to_string()
    };
    format!("{}/{component}", frame.origin.pointer)
}

pub(super) fn parse_yaml(source: &Path, bytes: &[u8]) -> Result<YamlDocument, Diagnostics> {
    let file = Origin::file(source);
    if bytes.len() > BYTE_LIMIT {
        return Err(Diagnostic::new("E_CONFIG_LIMIT", &file).into());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| Diagnostics::from(Diagnostic::new("E_YAML_ENCODING", &file)))?;
    let bom = usize::from(text.starts_with('\u{feff}')) * 3;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let map = SourceMap::new(source, text, bom);
    if text.starts_with('\u{feff}') {
        return Err(Diagnostic::new("E_YAML_SYNTAX", &map.origin(0)).into());
    }
    let mut scanner = Scanner::new(text.chars());
    let mut depth = 0usize;
    // Reject source controls before the scanner can treat NUL as end-of-stream.
    let mut candidate = text
        .chars()
        .enumerate()
        .find(|(_, c)| {
            !matches!(c,
                '\u{9}' | '\u{a}' | '\u{d}' | '\u{20}'..='\u{7e}' | '\u{85}' |
                '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}'
            )
        })
        .map(|(index, _)| Diagnostic::new("E_YAML_SYNTAX", &map.origin(index)));
    for token in scanner.by_ref() {
        let origin = map.origin(token.0.index());
        let code = match &token.1 {
            TokenType::Anchor(_)
            | TokenType::Alias(_)
            | TokenType::Tag(_, _)
            | TokenType::TagDirective(_, _)
            | TokenType::VersionDirective(_, _) => Some("E_YAML_FEATURE"),
            TokenType::BlockMappingStart
            | TokenType::BlockSequenceStart
            | TokenType::FlowMappingStart
            | TokenType::FlowSequenceStart => {
                depth += 1;
                (depth > 128).then_some("E_CONFIG_LIMIT")
            }
            TokenType::BlockEnd | TokenType::FlowMappingEnd | TokenType::FlowSequenceEnd => {
                depth = depth.saturating_sub(1);
                None
            }
            _ => None,
        };
        if let Some(code) = code {
            if candidate
                .as_ref()
                .is_none_or(|c| origin.offset < c.origin_offset())
            {
                candidate = Some(Diagnostic::new(code, &origin));
            }
            break;
        }
    }
    if let Some(e) = scanner.get_error() {
        let origin = map.origin(e.marker().index());
        if candidate
            .as_ref()
            .is_none_or(|c| origin.offset < c.origin_offset())
        {
            candidate = Some(Diagnostic::new("E_YAML_SYNTAX", &origin));
        }
    }
    let mut parser = Parser::new_from_str(text);
    let mut stack: Vec<Frame> = Vec::new();
    let mut root = None;
    let mut documents = 0;
    let mut count = 0;
    loop {
        let (event, marker) = match parser.next_token() {
            Ok(pair) => pair,
            Err(e) => {
                let origin = map.origin(e.marker().index());
                if let Some(mut c) = candidate
                    .take()
                    .filter(|c| c.origin_offset() <= origin.offset)
                {
                    if c.code() != "E_YAML_SYNTAX" {
                        c.set_pointer(&child_pointer(&stack));
                    }
                    return Err(c.into());
                }
                return Err(Diagnostic::new("E_YAML_SYNTAX", &origin).into());
            }
        };
        let origin = map.origin(marker.index()).at(&child_pointer(&stack));
        if let Some(c) = &candidate
            && c.origin_offset() <= origin.offset
        {
            let mut c = c.clone();
            if c.code() != "E_YAML_SYNTAX" {
                c.set_pointer(&child_pointer(&stack));
            }
            return Err(c.into());
        }
        let node = match event {
            Event::StreamEnd => break,
            Event::DocumentStart => {
                documents += 1;
                if documents > 1 {
                    return Err(Diagnostic::new("E_YAML_DOCUMENT", &origin).into());
                }
                None
            }
            Event::MappingStart(_, _) | Event::SequenceStart(_, _) => {
                count += 1;
                stack.push(Frame {
                    origin: origin.clone(),
                    mapping: matches!(event, Event::MappingStart(_, _)),
                    children: Vec::new(),
                    keys: BTreeMap::new(),
                });
                None
            }
            Event::MappingEnd | Event::SequenceEnd => {
                let mut frame = stack
                    .pop()
                    .ok_or_else(|| Diagnostics::from(Diagnostic::new("E_YAML_SYNTAX", &origin)))?;
                // Implicit block mappings can start at the first key's colon in parser events.
                if frame.mapping
                    && let Some(first) = frame.children.first()
                    && first.origin.offset < frame.origin.offset
                {
                    frame.origin = first.origin.at(&frame.origin.pointer);
                }
                let kind = if frame.mapping {
                    if frame.children.len() % 2 != 0 {
                        return Err(Diagnostic::new("E_YAML_SYNTAX", &origin).into());
                    }
                    let mut iter = frame.children.into_iter();
                    let mut pairs = Vec::new();
                    while let (Some(k), Some(v)) = (iter.next(), iter.next()) {
                        pairs.push((k, v));
                    }
                    Kind::Mapping(pairs)
                } else {
                    Kind::Sequence(frame.children)
                };
                Some(Node {
                    kind,
                    origin: frame.origin,
                })
            }
            Event::Scalar(value, style, _, _) => {
                count += 1;
                Some(Node {
                    kind: Kind::Scalar(value.clone(), classify_scalar(&value, style)),
                    origin,
                })
            }
            Event::Alias(_) => return Err(Diagnostic::new("E_YAML_FEATURE", &origin).into()),
            _ => None,
        };
        if count > 50_000 || stack.len() > 128 {
            return Err(Diagnostic::new("E_CONFIG_LIMIT", &map.origin(marker.index())).into());
        }
        if let Some(mut node) = node {
            if let Some(frame) = stack.last_mut() {
                if frame.mapping
                    && frame.children.len().is_multiple_of(2)
                    && let Kind::Scalar(key, _) = &node.kind
                {
                    node.origin.pointer = format!(
                        "{}/{}",
                        frame.origin.pointer,
                        key.replace('~', "~0").replace('/', "~1")
                    );
                    if key == "<<" {
                        return Err(Diagnostic::new("E_YAML_FEATURE", &node.origin).into());
                    }
                    if let Some(previous) = frame.keys.insert(key.clone(), node.origin.clone()) {
                        return Err(Diagnostic::new("E_YAML_DUPLICATE_KEY", &node.origin)
                            .related(&previous)
                            .into());
                    }
                }
                frame.children.push(node);
            } else {
                root = Some(node);
            }
        }
    }
    if let Some(c) = candidate {
        return Err(c.into());
    }
    if root
        .as_ref()
        .is_some_and(|n| matches!(&n.kind, Kind::Scalar(s, ScalarKind::Null) if s.is_empty()))
    {
        root = None;
    }
    Ok(YamlDocument {
        root,
        origin: map.origin(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::SourceLocation;
    use std::path::Path;

    #[test]
    fn located_yaml_gate() {
        let e = parse_yaml(Path::new("fixture.yaml"), b"navigation: &entries []\n").unwrap_err();
        assert_eq!(e.items()[0].code(), "E_YAML_FEATURE");
        assert_eq!(
            e.items()[0].location(),
            Some(SourceLocation {
                line: 1,
                column: 13
            })
        );
        let e = parse_yaml(Path::new("fixture.yaml"), b"a: x\na: y\n").unwrap_err();
        assert_eq!(e.items()[0].code(), "E_YAML_DUPLICATE_KEY");
        assert_eq!(
            e.items()[0].location(),
            Some(SourceLocation { line: 2, column: 1 })
        );
        for text in [
            "a: !!str x",
            "a: *missing",
            "a: {<<: x}",
            "%YAML 1.2\n---\na: x",
            "%FOO bar\n---\na: x",
        ] {
            assert_eq!(
                parse_yaml(Path::new("f"), text.as_bytes())
                    .unwrap_err()
                    .items()[0]
                    .code(),
                "E_YAML_FEATURE",
                "{text}"
            );
        }
        assert_eq!(
            parse_yaml(Path::new("f"), b"---\na: x\n---\n")
                .unwrap_err()
                .items()[0]
                .code(),
            "E_YAML_DOCUMENT"
        );
        assert_eq!(
            parse_yaml(Path::new("f"), &[255]).unwrap_err().items()[0].code(),
            "E_YAML_ENCODING"
        );
        assert_eq!(
            parse_yaml(Path::new("f"), b"navigation: [\n")
                .unwrap_err()
                .items()[0]
                .code(),
            "E_YAML_SYNTAX"
        );
        assert!(parse_yaml(Path::new("f"), b"a: '& % !'\n").is_ok());
        assert!(
            parse_yaml(Path::new("f"), b"# empty\n")
                .unwrap()
                .root
                .is_none()
        );
    }

    #[test]
    fn yaml_limits() {
        for (size, valid) in [(1_048_576, true), (1_048_577, false)] {
            let text = format!("#{}", "x".repeat(size - 1));
            assert_eq!(parse_yaml(Path::new("f"), text.as_bytes()).is_ok(), valid);
        }
        for (depth, valid) in [(128, true), (129, false), (10_000, false)] {
            let text = format!("{}x{}", "[".repeat(depth), "]".repeat(depth));
            assert_eq!(parse_yaml(Path::new("f"), text.as_bytes()).is_ok(), valid);
        }
        for (count, valid) in [(49_999, true), (50_000, false)] {
            let text = format!("[{}]", vec!["x"; count].join(","));
            assert_eq!(parse_yaml(Path::new("f"), text.as_bytes()).is_ok(), valid);
        }
    }
}
