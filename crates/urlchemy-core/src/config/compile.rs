use super::raw::*;
use crate::{
    catalog::*,
    diagnostic::{Diagnostic, Diagnostics, Origin},
    url::{compile_template, parse_base},
    value::{CaptureName, CapturedValue, TextError, check_text, valid_alias, valid_name},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_CATALOG: AtomicU64 = AtomicU64::new(1);
struct Compiler {
    owner: u64,
    index: u32,
    errors: Vec<Diagnostic>,
    keys: BTreeMap<String, Origin>,
    flows: Vec<Flow>,
    origin: Origin,
}
impl Compiler {
    fn error(&mut self, code: &'static str, origin: &Origin) {
        self.errors.push(Diagnostic::new(code, origin));
    }
    fn index(&mut self) -> Option<u32> {
        let index = self.index;
        self.index = self.index.checked_add(1).or_else(|| {
            self.error("E_CONFIG_LIMIT", &self.origin.clone());
            None
        })?;
        Some(index)
    }
    fn text(&mut self, f: &Field<String>, value: bool) {
        if let Err(e) = check_text(&f.value) {
            self.error(
                match (value, e) {
                    (true, TextError::Required) => "E_VALUE_REQUIRED",
                    (true, TextError::Control) => "E_VALUE_CONTROL",
                    (false, TextError::Required) => "E_TEXT_REQUIRED",
                    (false, TextError::Control) => "E_TEXT_CONTROL",
                },
                &f.origin,
            );
        }
    }
    fn alias(&mut self, f: &Option<Field<String>>) {
        if let Some(f) = f
            && !valid_alias(&f.value)
        {
            self.error("E_ALIAS", &f.origin);
        }
    }
    fn presentation(&mut self, p: RawPresentation) -> Presentation {
        self.text(&p.label, false);
        if let Some(f) = &p.emoji {
            self.text(f, false);
        }
        self.alias(&p.alias);
        Presentation {
            label: p.label.value,
            emoji: p.emoji.map(|f| f.value),
            alias: p.alias.map(|f| f.value),
        }
    }
    fn capture(&mut self, f: &Field<String>, available: &mut BTreeSet<CaptureName>) -> bool {
        if !valid_name(&f.value) {
            self.error("E_NAME", &f.origin);
            return false;
        }
        if !available.insert(CaptureName(f.value.clone())) {
            self.error("E_CAPTURE_DUPLICATE", &f.origin);
        }
        true
    }
    fn entry(&mut self, raw: RawEntry) -> Option<Entry> {
        let id = EntryId {
            owner: self.owner,
            index: self.index()?,
        };
        match raw {
            RawEntry::Folder(raw) => {
                let presentation = self.presentation(raw.presentation);
                let children = raw
                    .children
                    .into_iter()
                    .filter_map(|r| self.entry(r))
                    .collect();
                Some(Entry {
                    id,
                    presentation,
                    kind: EntryKind::Folder(children),
                })
            }
            RawEntry::Flow(raw) => {
                if !valid_name(&raw.id.value) {
                    self.error("E_NAME", &raw.id.origin);
                }
                if let Some(prior) = self
                    .keys
                    .insert(raw.id.value.clone(), raw.id.origin.clone())
                {
                    self.errors
                        .push(Diagnostic::new("E_FLOW_DUPLICATE", &raw.id.origin).related(&prior));
                }
                let presentation = self.presentation(raw.presentation);
                self.alias(&raw.direct_alias);
                let base = parse_base(&raw.base.value, &raw.base.origin)
                    .map_err(|e| self.errors.push(e))
                    .ok();
                let start = self.step(raw.start, BTreeSet::new(), false);
                let flow_id = FlowId {
                    owner: self.owner,
                    index: self.index()?,
                };
                self.flows.push(Flow {
                    id: flow_id,
                    key: raw.id.value,
                    direct_alias: raw.direct_alias.map(|f| f.value),
                    base: base?,
                    start: start?,
                });
                Some(Entry {
                    id,
                    presentation,
                    kind: EntryKind::Flow(flow_id),
                })
            }
        }
    }
    fn step(
        &mut self,
        raw: RawStep,
        mut available: BTreeSet<CaptureName>,
        mut bad_path: bool,
    ) -> Option<Step> {
        let id = StepId {
            owner: self.owner,
            index: self.index()?,
        };
        let kind = match raw.kind {
            RawStepKind::Destination { template } => {
                let compiled = compile_template(&template.value, &template.origin)
                    .map_err(|e| self.errors.push(e))
                    .ok()?;
                if !bad_path
                    && compiled
                        .references
                        .iter()
                        .any(|name| !available.contains(name))
                {
                    self.error("E_CAPTURE_UNAVAILABLE", &template.origin);
                }
                StepKind::Destination(compiled)
            }
            RawStepKind::Input {
                label,
                capture,
                suggestions,
                next,
            } => {
                self.text(&label, false);
                bad_path |= !self.capture(&capture, &mut available);
                let suggestions = suggestions
                    .into_iter()
                    .filter_map(|s| {
                        self.text(&s.label, false);
                        self.text(&s.value, true);
                        Some(Suggestion {
                            id: SuggestionId {
                                owner: self.owner,
                                index: self.index()?,
                            },
                            label: s.label.value,
                            value: CapturedValue(s.value.value),
                        })
                    })
                    .collect();
                StepKind::Input {
                    label: label.value,
                    capture: CaptureName(capture.value),
                    suggestions,
                    next: Box::new(self.step(*next, available, bad_path)?),
                }
            }
            RawStepKind::Select {
                label,
                capture,
                options,
                next,
            } => {
                self.text(&label, false);
                if let Some(capture) = &capture {
                    bad_path |= !self.capture(capture, &mut available);
                }
                let common = next.is_some();
                let mut choices = Vec::new();
                let mut branches = Vec::new();
                for option in options {
                    self.text(&option.label, false);
                    if let Some(value) = &option.value {
                        self.text(value, true);
                    }
                    if capture.is_some() != option.value.is_some() {
                        let origin = option
                            .value
                            .as_ref()
                            .map(|v| v.origin.clone())
                            .unwrap_or_else(|| {
                                option
                                    .origin
                                    .at(&format!("{}/value", option.origin.pointer))
                            });
                        self.error("E_CAPTURE_VALUE", &origin);
                    }
                    let choice = Choice {
                        id: ChoiceId {
                            owner: self.owner,
                            index: self.index()?,
                        },
                        label: option.label.value,
                        capture: capture.as_ref().zip(option.value).map(|(name, value)| {
                            (CaptureName(name.value.clone()), CapturedValue(value.value))
                        }),
                    };
                    if common {
                        if option.next.is_some() {
                            self.error(
                                "E_CONTINUATION",
                                option.next_key.as_ref().unwrap_or(&option.origin),
                            );
                        }
                        choices.push(choice);
                    } else if let Some(next) = option.next {
                        if let Some(next) = self.step(*next, available.clone(), bad_path) {
                            branches.push(BranchChoice {
                                choice,
                                next: Box::new(next),
                            });
                        }
                    } else {
                        self.error(
                            "E_CONTINUATION",
                            &option.origin.at(&format!("{}/next", option.origin.pointer)),
                        );
                    }
                }
                let selection = if let Some(next) = next {
                    Selection::Common {
                        choices,
                        next: Box::new(self.step(*next, available, bad_path)?),
                    }
                } else {
                    Selection::Branched(branches)
                };
                StepKind::Select {
                    label: label.value,
                    selection,
                }
            }
        };
        Some(Step {
            id,
            origin: raw.origin,
            kind,
        })
    }
}

pub(super) fn compile(raw: RawConfig) -> Result<Catalog, Diagnostics> {
    let owner = NEXT_CATALOG
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| Diagnostics::from(Diagnostic::new("E_CONFIG_LIMIT", &raw.origin)))?;
    let mut c = Compiler {
        owner,
        index: 0,
        errors: Vec::new(),
        keys: BTreeMap::new(),
        flows: Vec::new(),
        origin: raw.origin,
    };
    let roots = raw
        .navigation
        .into_iter()
        .filter_map(|r| c.entry(r))
        .collect();
    if c.errors.is_empty() {
        Ok(Catalog {
            owner,
            roots,
            flows: c.flows,
        })
    } else {
        Err(Diagnostics::collect(c.errors))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::load_bytes;
    use std::path::Path;
    fn fixture(name: &str) -> Result<crate::Catalog, crate::Diagnostics> {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/superpowers/specs/yaml-url-contract")
            .join(name);
        load_bytes(&p, &std::fs::read(&p).unwrap())
    }
    #[test]
    fn catalog_invariants() {
        for f in ["fixed.yaml", "linear.yaml", "common.yaml", "branching.yaml"] {
            assert!(fixture(f).is_ok(), "{f}");
        }
        for (f, code) in [
            ("invalid-missing-continuation.yaml", "E_CONTINUATION"),
            ("invalid-mixed-continuation.yaml", "E_CONTINUATION"),
            ("invalid-noncapturing-value.yaml", "E_CAPTURE_VALUE"),
            ("invalid-unavailable-capture.yaml", "E_CAPTURE_UNAVAILABLE"),
            ("invalid-duplicate-capture.yaml", "E_CAPTURE_DUPLICATE"),
            ("invalid-base.yaml", "E_BASE_URL"),
            ("invalid-template.yaml", "E_TEMPLATE_SYNTAX"),
        ] {
            assert_eq!(fixture(f).unwrap_err().items()[0].code(), code, "{f}");
        }
    }
    #[test]
    fn atomic_catalog() {
        let mut text = String::from("navigation:\n");
        for n in 0..51 {
            text.push_str(&format!("  - flow:\n      id: flow{n}\n      label: Example\n      baseURL: invalid\n      start:\n        destination:\n          template: /\n"));
        }
        let e = load_bytes(Path::new("f"), text.as_bytes()).unwrap_err();
        assert_eq!(e.items().len(), 50);
        assert!(e.truncated());
        assert!(e.items().iter().all(|d| d.code() == "E_BASE_URL"));
        let duplicate = "navigation: [{flow: {id: same, label: One, baseURL: 'https://example.com', start: {destination: {template: /}}}}, {flow: {id: same, label: Two, baseURL: 'https://example.com', start: {destination: {template: /}}}}]";
        assert_eq!(
            load_bytes(Path::new("f"), duplicate.as_bytes())
                .unwrap_err()
                .items()[0]
                .code(),
            "E_FLOW_DUPLICATE"
        );
        let invalid = "navigation: [{flow: {id: flow, label: One, baseURL: 'https://example.com', start: {input: {label: X, capture: 'bad name', next: {destination: {template: '/{x}'}}}}}}]";
        let e = load_bytes(Path::new("f"), invalid.as_bytes()).unwrap_err();
        assert_eq!(e.items().len(), 1);
        assert_eq!(e.items()[0].code(), "E_NAME");
    }
}
