use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Origin {
    pub source: PathBuf,
    pub pointer: String,
    pub location: Option<SourceLocation>,
    pub offset: usize,
}

impl Origin {
    pub fn file(source: &Path) -> Self {
        Self {
            source: source.into(),
            pointer: String::new(),
            location: None,
            offset: 0,
        }
    }
    pub fn at(&self, pointer: &str) -> Self {
        Self {
            pointer: pointer.into(),
            ..self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    code: &'static str,
    origin: Box<Origin>,
    related: Vec<SourceLocation>,
    detail: Option<&'static str>,
}

impl Diagnostic {
    pub(crate) fn new(code: &'static str, origin: &Origin) -> Self {
        Self {
            code,
            origin: Box::new(origin.clone()),
            related: Vec::new(),
            detail: None,
        }
    }
    pub(crate) fn related(mut self, origin: &Origin) -> Self {
        self.related.extend(origin.location);
        self
    }
    pub(crate) fn with_message(mut self, message: &'static str) -> Self {
        self.detail = Some(message);
        self
    }
    pub(crate) fn set_pointer(&mut self, pointer: &str) {
        self.origin.pointer = pointer.into();
    }
    pub fn code(&self) -> &'static str {
        self.code
    }
    pub(crate) fn origin_offset(&self) -> usize {
        self.origin.offset
    }
    pub fn pointer(&self) -> &str {
        &self.origin.pointer
    }
    pub fn location(&self) -> Option<SourceLocation> {
        self.origin.location
    }
    pub fn source(&self) -> &Path {
        &self.origin.source
    }
    pub fn related_locations(&self) -> &[SourceLocation] {
        &self.related
    }
    pub fn message(&self) -> &'static str {
        if let Some(message) = self.detail {
            return message;
        }
        match self.code {
            "E_SCHEMA_TYPE" => "Expected the required field type.",
            "E_SCHEMA_REQUIRED" => "Define the required field.",
            "E_SCHEMA_UNKNOWN_FIELD" => "Remove the unknown field.",
            "E_SHORTCUT_SYNTAX" => {
                "Define one supported key combination with the required modifiers."
            }
            "E_SHORTCUT_CONFLICT" => {
                "Choose a shortcut that does not conflict with another binding or a protected command."
            }
            "E_CONTINUATION" => {
                "Define either one common continuation or a continuation for every option."
            }
            "E_CAPTURE_UNAVAILABLE" => {
                "This destination references a capture unavailable on its path."
            }
            "E_INPUT_REQUIRED" | "E_VALUE_REQUIRED" | "E_TEXT_REQUIRED" => {
                "Enter a nonblank value."
            }
            "E_INPUT_CONTROL" | "E_VALUE_CONTROL" | "E_TEXT_CONTROL" => {
                "Remove control characters."
            }
            "E_RESOLVE_DOT_SEGMENT" => "A rendered path segment must not be a dot segment.",
            "E_CONFIG_MISSING" => "Create the configuration file at the expected location.",
            "E_CONFIG_READ" => "The configuration file could not be read.",
            "E_CONFIG_LIMIT" => "The configuration exceeds a resource limit.",
            "E_YAML_SYNTAX" => "Correct the YAML syntax at this location.",
            "E_YAML_FEATURE" => "Remove the unsupported YAML feature.",
            "E_YAML_DUPLICATE_KEY" => "Remove the duplicate mapping key.",
            "E_YAML_ENCODING" => "Save the configuration as UTF-8.",
            "E_YAML_DOCUMENT" => "Define exactly one YAML document.",
            "E_BASE_URL" => "Define a valid fixed HTTP or HTTPS base URL.",
            "E_TEMPLATE_SYNTAX" => "Correct the destination template syntax.",
            "E_ACTION_CATALOG" => "Invoke a new session with this configuration.",
            "E_ACTION_TARGET" => "Select a target available at the current step.",
            "E_ACTION_CONTEXT" => "This action is unavailable in the current state.",
            _ => "The configuration or action violates the required contract.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
    truncated: bool,
}
impl Diagnostics {
    pub(crate) fn collect(mut items: Vec<Diagnostic>) -> Self {
        items.sort_by_key(|d| d.origin.offset);
        let truncated = items.len() > 50;
        items.truncate(50);
        Self { items, truncated }
    }
    pub fn items(&self) -> &[Diagnostic] {
        &self.items
    }
    pub fn truncated(&self) -> bool {
        self.truncated
    }
}
impl From<Diagnostic> for Diagnostics {
    fn from(item: Diagnostic) -> Self {
        Self::collect(vec![item])
    }
}
