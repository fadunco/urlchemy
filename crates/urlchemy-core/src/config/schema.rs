use super::{
    raw::*,
    scalar::ScalarKind,
    yaml::{Kind, Node, YamlDocument},
};
use crate::diagnostic::{Diagnostic, Diagnostics, Origin};

struct Converter {
    errors: Vec<Diagnostic>,
}
type Pairs<'a> = &'a [(Node, Node)];
impl Converter {
    fn type_error(&mut self, origin: &Origin, message: &'static str) {
        self.errors
            .push(Diagnostic::new("E_SCHEMA_TYPE", origin).with_message(message));
    }
    fn error(&mut self, code: &'static str, origin: &Origin) {
        self.errors.push(Diagnostic::new(code, origin));
    }
    fn mapping<'a>(&mut self, node: &'a Node, allowed: &[&str]) -> Option<Pairs<'a>> {
        let Kind::Mapping(pairs) = &node.kind else {
            self.type_error(&node.origin, "Expected a mapping.");
            return None;
        };
        for (key, _) in pairs {
            match &key.kind {
                Kind::Scalar(s, ScalarKind::String) if allowed.contains(&s.as_str()) => (),
                Kind::Scalar(_, ScalarKind::String) => {
                    self.error("E_SCHEMA_UNKNOWN_FIELD", &key.origin)
                }
                _ => self.type_error(&key.origin, "Expected a string."),
            }
        }
        Some(pairs)
    }
    fn get<'a>(
        &mut self,
        pairs: Pairs<'a>,
        key: &str,
        parent: &Origin,
        required: bool,
    ) -> Option<&'a Node> {
        let node = pairs
            .iter()
            .find(|(k, _)| matches!(&k.kind, Kind::Scalar(s, _) if s == key))
            .map(|(_, v)| v);
        if node.is_none() && required {
            self.error(
                "E_SCHEMA_REQUIRED",
                &parent.at(&format!("{}/{key}", parent.pointer)),
            );
        }
        node
    }
    fn string(&mut self, node: &Node) -> Option<Field<String>> {
        match &node.kind {
            Kind::Scalar(s, ScalarKind::String) => Some(Field {
                value: s.clone(),
                origin: node.origin.clone(),
            }),
            _ => {
                self.type_error(&node.origin, "Expected a string.");
                None
            }
        }
    }
    fn text(
        &mut self,
        pairs: Pairs<'_>,
        key: &str,
        origin: &Origin,
        required: bool,
    ) -> Option<Field<String>> {
        self.get(pairs, key, origin, required)
            .and_then(|n| self.string(n))
    }
    fn sequence<'a>(&mut self, node: &'a Node) -> Option<&'a [Node]> {
        if let Kind::Sequence(nodes) = &node.kind {
            Some(nodes)
        } else {
            self.type_error(&node.origin, "Expected a sequence.");
            None
        }
    }
    fn presentation(&mut self, pairs: Pairs<'_>, origin: &Origin) -> Option<RawPresentation> {
        let label = self.text(pairs, "label", origin, true);
        let emoji = self.text(pairs, "emoji", origin, false);
        let alias = self.text(pairs, "alias", origin, false);
        Some(RawPresentation {
            label: label?,
            emoji,
            alias,
        })
    }
    fn variant<'a>(&mut self, node: &'a Node, names: &[&str]) -> Option<(&'a str, &'a Node)> {
        let Kind::Mapping(pairs) = &node.kind else {
            self.error("E_SCHEMA_VARIANT", &node.origin);
            return None;
        };
        if pairs.len() == 1
            && let Kind::Scalar(name, ScalarKind::String) = &pairs[0].0.kind
            && names.contains(&name.as_str())
        {
            return Some((name, &pairs[0].1));
        }
        self.error("E_SCHEMA_VARIANT", &node.origin);
        None
    }
    fn entry(&mut self, node: &Node) -> Option<RawEntry> {
        let (name, inner) = self.variant(node, &["folder", "flow"])?;
        let origin = &inner.origin;
        if name == "folder" {
            let pairs = self.mapping(inner, &["label", "emoji", "alias", "children"])?;
            let presentation = self.presentation(pairs, origin);
            let children = self
                .get(pairs, "children", origin, true)
                .and_then(|n| self.sequence(n))
                .map(|ns| ns.iter().filter_map(|n| self.entry(n)).collect());
            Some(RawEntry::Folder(Box::new(RawFolder {
                presentation: presentation?,
                children: children?,
            })))
        } else {
            let pairs = self.mapping(
                inner,
                &[
                    "id",
                    "label",
                    "emoji",
                    "alias",
                    "direct_alias",
                    "baseURL",
                    "start",
                ],
            )?;
            let id = self.text(pairs, "id", origin, true);
            let presentation = self.presentation(pairs, origin);
            let direct_alias = self.text(pairs, "direct_alias", origin, false);
            let base = self.text(pairs, "baseURL", origin, true);
            let start = self
                .get(pairs, "start", origin, true)
                .and_then(|n| self.step(n));
            Some(RawEntry::Flow(Box::new(RawFlow {
                id: id?,
                presentation: presentation?,
                direct_alias,
                base: base?,
                start: start?,
            })))
        }
    }
    fn step(&mut self, node: &Node) -> Option<RawStep> {
        let (name, inner) = self.variant(node, &["input", "select", "destination"])?;
        let origin = &inner.origin;
        let kind = match name {
            "destination" => {
                let pairs = self.mapping(inner, &["template"])?;
                RawStepKind::Destination {
                    template: self.text(pairs, "template", origin, true)?,
                }
            }
            "input" => {
                let pairs = self.mapping(inner, &["label", "capture", "suggestions", "next"])?;
                let label = self.text(pairs, "label", origin, true);
                let capture = self.text(pairs, "capture", origin, true);
                let suggestions = self
                    .get(pairs, "suggestions", origin, false)
                    .and_then(|n| self.sequence(n))
                    .map(|ns| ns.iter().filter_map(|n| self.suggestion(n)).collect())
                    .unwrap_or_default();
                let next = self
                    .get(pairs, "next", origin, true)
                    .and_then(|n| self.step(n));
                RawStepKind::Input {
                    label: label?,
                    capture: capture?,
                    suggestions,
                    next: Box::new(next?),
                }
            }
            _ => {
                let pairs = self.mapping(inner, &["label", "capture", "options", "next"])?;
                let label = self.text(pairs, "label", origin, true);
                let capture = self.text(pairs, "capture", origin, false);
                let options = self.get(pairs, "options", origin, true).and_then(|n| {
                    self.sequence(n).map(|ns| {
                        if ns.is_empty() {
                            self.error("E_SCHEMA_EMPTY_OPTIONS", &n.origin);
                        }
                        ns.iter().filter_map(|n| self.choice(n)).collect()
                    })
                });
                let next = self
                    .get(pairs, "next", origin, false)
                    .and_then(|n| self.step(n))
                    .map(Box::new);
                RawStepKind::Select {
                    label: label?,
                    capture,
                    options: options?,
                    next,
                }
            }
        };
        Some(RawStep {
            origin: origin.clone(),
            kind,
        })
    }
    fn suggestion(&mut self, node: &Node) -> Option<RawSuggestion> {
        let pairs = self.mapping(node, &["label", "value"])?;
        let label = self.text(pairs, "label", &node.origin, true);
        let value = self.text(pairs, "value", &node.origin, true);
        Some(RawSuggestion {
            label: label?,
            value: value?,
        })
    }
    fn choice(&mut self, node: &Node) -> Option<RawChoice> {
        let pairs = self.mapping(node, &["label", "value", "next"])?;
        let label = self.text(pairs, "label", &node.origin, true);
        let value = self.text(pairs, "value", &node.origin, false);
        let next = self
            .get(pairs, "next", &node.origin, false)
            .and_then(|n| self.step(n))
            .map(Box::new);
        let next_key = pairs
            .iter()
            .find(|(k, _)| matches!(&k.kind, Kind::Scalar(s, _) if s == "next"))
            .map(|(k, _)| k.origin.clone());
        Some(RawChoice {
            origin: node.origin.clone(),
            label: label?,
            value,
            next,
            next_key,
        })
    }
}

pub(super) fn deserialize(document: YamlDocument) -> Result<RawConfig, Diagnostics> {
    let mut c = Converter { errors: Vec::new() };
    let mut shortcuts = None;
    let navigation = document.root.as_ref().and_then(|root| {
        let pairs = c.mapping(root, &["navigation", "shortcuts"])?;
        shortcuts = c
            .get(pairs, "shortcuts", &root.origin, false)
            .and_then(|node| {
                let pairs = c.mapping(node, &["invoke", "launch"])?;
                Some(RawShortcuts {
                    invoke: c.text(pairs, "invoke", &node.origin, false),
                    launch: c.text(pairs, "launch", &node.origin, false),
                })
            });
        c.get(pairs, "navigation", &root.origin, true)
            .and_then(|n| c.sequence(n))
            .map(|ns| ns.iter().filter_map(|n| c.entry(n)).collect())
    });
    if document.root.is_none() {
        c.type_error(&document.origin, "Expected a mapping.");
    }
    if c.errors.is_empty() {
        Ok(RawConfig {
            origin: document.origin,
            navigation: navigation.unwrap_or_default(),
            shortcuts,
        })
    } else {
        Err(Diagnostics::collect(c.errors))
    }
}

#[cfg(test)]
mod tests {
    use super::super::yaml::parse_yaml;
    use super::*;
    use std::path::Path;

    fn fixture(name: &str) -> Result<super::super::raw::RawConfig, crate::diagnostic::Diagnostics> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/superpowers/specs/yaml-url-contract")
            .join(name);
        deserialize(parse_yaml(&path, &std::fs::read(&path).unwrap())?)
    }
    #[test]
    fn strict_schema() {
        for file in ["fixed.yaml", "linear.yaml", "common.yaml", "branching.yaml"] {
            assert!(fixture(file).is_ok(), "{file}");
        }
        for (file, code, pointer) in [
            (
                "invalid-unknown-field.yaml",
                "E_SCHEMA_UNKNOWN_FIELD",
                "/navigation/0/flow/baseUrl",
            ),
            (
                "invalid-string-type.yaml",
                "E_SCHEMA_TYPE",
                "/navigation/0/flow/start/select/options/0/value",
            ),
            (
                "invalid-null.yaml",
                "E_SCHEMA_TYPE",
                "/navigation/0/flow/emoji",
            ),
            (
                "invalid-empty-options.yaml",
                "E_SCHEMA_EMPTY_OPTIONS",
                "/navigation/0/flow/start/select/options",
            ),
        ] {
            let errors = fixture(file).unwrap_err();
            assert_eq!(errors.items()[0].code(), code, "{file}");
            assert_eq!(errors.items()[0].pointer(), pointer, "{file}");
        }
        assert!(deserialize(parse_yaml(Path::new("f"), b"navigation: []").unwrap()).is_ok());
        assert_eq!(
            deserialize(parse_yaml(Path::new("f"), b"").unwrap())
                .unwrap_err()
                .items()[0]
                .code(),
            "E_SCHEMA_TYPE"
        );
        assert_eq!(
            deserialize(parse_yaml(Path::new("f"), b"navigation: [{unknown: {}}]").unwrap())
                .unwrap_err()
                .items()[0]
                .code(),
            "E_SCHEMA_VARIANT"
        );
    }
}
