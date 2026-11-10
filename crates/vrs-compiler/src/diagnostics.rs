//! Diagnostics du compilateur (format JSON de rustc/Cargo).

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Gravité.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Error,
    Warning,
    Note,
    Help,
}

impl Level {
    fn parse(s: &str) -> Self {
        match s {
            "error" | "error: internal compiler error" => Self::Error,
            "warning" => Self::Warning,
            "help" => Self::Help,
            _ => Self::Note,
        }
    }

    /// Clé de traduction.
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::Error => "diag-level-error",
            Self::Warning => "diag-level-warning",
            Self::Note => "diag-level-note",
            Self::Help => "diag-level-help",
        }
    }
}

/// Emplacement dans un fichier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub file: PathBuf,
    pub line_start: usize,
    pub column_start: usize,
    pub line_end: usize,
    pub column_end: usize,
    pub label: Option<String>,
}

/// Correction proposée par le compilateur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    pub message: String,
    pub span: Span,
    pub replacement: String,
}

/// Un diagnostic du compilateur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: Level,
    /// Code officiel (`E0308`) ou nom du lint (`unused_variables`).
    pub code: Option<String>,
    /// Message original (en anglais, tel que produit par rustc).
    pub message: String,
    /// Emplacement principal.
    pub span: Option<Span>,
    /// Rendu complet de rustc (message original avec contexte).
    pub rendered: String,
    pub suggestions: Vec<Suggestion>,
    /// Paquet concerné.
    pub package: Option<String>,
}

impl Diagnostic {
    /// Clé de traduction de l'explication pédagogique de VRS, si le code est connu.
    pub fn explanation_key(&self) -> Option<String> {
        let code = self.code.as_deref()?;
        if code.starts_with('E') && code[1..].chars().all(|c| c.is_ascii_digit()) {
            Some(format!("rustc-{code}"))
        } else if code.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
            Some(format!("lint-{code}"))
        } else {
            // Lints Clippy (`clippy::needless_return`) : pas d'explication locale pour l'instant.
            None
        }
    }

    /// Lien vers la documentation officielle de l'erreur.
    pub fn doc_url(&self) -> Option<String> {
        let code = self.code.as_deref()?;
        if let Some(lint) = code.strip_prefix("clippy::") {
            return Some(format!(
                "https://rust-lang.github.io/rust-clippy/master/index.html#{lint}"
            ));
        }
        code.starts_with('E')
            .then(|| format!("https://doc.rust-lang.org/error_codes/{code}.html"))
    }
}

#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    #[serde(default)]
    package_id: Option<String>,
    #[serde(default)]
    message: Option<RustcMessage>,
    #[serde(default)]
    success: Option<bool>,
}

#[derive(Deserialize)]
struct RustcMessage {
    message: String,
    #[serde(default)]
    code: Option<RustcCode>,
    level: String,
    #[serde(default)]
    spans: Vec<RustcSpan>,
    #[serde(default)]
    children: Vec<RustcMessage>,
    #[serde(default)]
    rendered: Option<String>,
}

#[derive(Deserialize)]
struct RustcCode {
    code: String,
}

#[derive(Deserialize)]
struct RustcSpan {
    file_name: String,
    line_start: usize,
    line_end: usize,
    column_start: usize,
    column_end: usize,
    is_primary: bool,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    suggested_replacement: Option<String>,
}

fn to_span(s: &RustcSpan, root: &Path) -> Span {
    let p = PathBuf::from(&s.file_name);
    Span {
        file: if p.is_absolute() { p } else { root.join(p) },
        line_start: s.line_start,
        column_start: s.column_start,
        line_end: s.line_end,
        column_end: s.column_end,
        label: s.label.clone(),
    }
}

/// Résultat de l'analyse d'une ligne de sortie de Cargo.
#[derive(Debug, PartialEq)]
pub enum CargoLine {
    /// Un diagnostic du compilateur.
    Diagnostic(Box<Diagnostic>),
    /// Fin de compilation (`build-finished`).
    BuildFinished(bool),
    /// Autre message JSON de Cargo (artefact compilé…), sans intérêt pour l'utilisateur.
    Ignored,
    /// Ligne qui n'est pas du JSON Cargo (sortie du programme exécuté…).
    Text,
}

/// Analyse une ligne de `--message-format=json`. Les chemins relatifs sont
/// résolus par rapport à `root` (répertoire du workspace).
pub fn parse_cargo_line(line: &str, root: &Path) -> CargoLine {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('{') {
        return CargoLine::Text;
    }
    let Ok(msg) = serde_json::from_str::<CargoMessage>(trimmed) else {
        return CargoLine::Text;
    };
    match msg.reason.as_str() {
        "compiler-message" => {
            let Some(m) = msg.message else {
                return CargoLine::Ignored;
            };
            let level = Level::parse(&m.level);
            // Les messages récapitulatifs (« aborting due to… ») n'ont pas d'emplacement.
            let span = m
                .spans
                .iter()
                .find(|s| s.is_primary)
                .or(m.spans.first())
                .map(|s| to_span(s, root));
            let mut suggestions = Vec::new();
            for child in &m.children {
                for s in &child.spans {
                    if let Some(rep) = &s.suggested_replacement {
                        suggestions.push(Suggestion {
                            message: child.message.clone(),
                            span: to_span(s, root),
                            replacement: rep.clone(),
                        });
                    }
                }
            }
            let package = msg.package_id.as_deref().map(package_name);
            CargoLine::Diagnostic(Box::new(Diagnostic {
                level,
                code: m.code.map(|c| c.code),
                message: m.message,
                span,
                rendered: m.rendered.unwrap_or_default(),
                suggestions,
                package,
            }))
        }
        "build-finished" => CargoLine::BuildFinished(msg.success.unwrap_or(false)),
        _ => CargoLine::Ignored,
    }
}

/// `path+file:///x/mon-app#0.1.0` ou `mon-app 0.1.0 (path+file://…)` → `mon-app`.
fn package_name(id: &str) -> String {
    if let Some((_, rest)) = id.rsplit_once('#') {
        // Format récent : …/mon-app#0.1.0 ou …#mon-app@0.1.0
        if let Some((name, _)) = rest.split_once('@') {
            return name.to_string();
        }
        let before = id.rsplit_once('#').map(|(b, _)| b).unwrap_or(id);
        return before.rsplit('/').next().unwrap_or(before).to_string();
    }
    id.split_whitespace().next().unwrap_or(id).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const E0308: &str = r#"{"reason":"compiler-message","package_id":"path+file:///tmp/demo#0.1.0","manifest_path":"/tmp/demo/Cargo.toml","target":{},"message":{"rendered":"error[E0308]: mismatched types\n --> src/main.rs:2:18\n","$message_type":"diagnostic","children":[{"children":[],"code":null,"level":"help","message":"try using a conversion method","rendered":null,"spans":[{"byte_end":30,"byte_start":28,"column_end":20,"column_start":18,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":null,"line_end":2,"line_start":2,"suggested_replacement":"\"5\".to_string()","suggestion_applicability":"MaybeIncorrect","text":[]}]}],"code":{"code":"E0308","explanation":"..."},"level":"error","message":"mismatched types","spans":[{"byte_end":30,"byte_start":28,"column_end":20,"column_start":18,"expansion":null,"file_name":"src/main.rs","is_primary":true,"label":"expected `String`, found integer","line_end":2,"line_start":2,"suggested_replacement":null,"suggestion_applicability":null,"text":[]}]}}"#;

    #[test]
    fn parses_error() {
        let CargoLine::Diagnostic(d) = parse_cargo_line(E0308, Path::new("/tmp/demo")) else {
            panic!("diagnostic attendu");
        };
        assert_eq!(d.level, Level::Error);
        assert_eq!(d.code.as_deref(), Some("E0308"));
        assert_eq!(d.message, "mismatched types");
        let span = d.span.as_ref().unwrap();
        assert_eq!(span.file, PathBuf::from("/tmp/demo/src/main.rs"));
        assert_eq!((span.line_start, span.column_start), (2, 18));
        assert_eq!(d.explanation_key().as_deref(), Some("rustc-E0308"));
        assert_eq!(d.suggestions.len(), 1);
        assert_eq!(d.suggestions[0].replacement, "\"5\".to_string()");
        assert_eq!(d.package.as_deref(), Some("demo"));
        assert!(d.doc_url().unwrap().ends_with("E0308.html"));
    }

    #[test]
    fn other_lines() {
        let root = Path::new("/");
        assert_eq!(parse_cargo_line("Hello world", root), CargoLine::Text);
        assert_eq!(
            parse_cargo_line(r#"{"reason":"build-finished","success":true}"#, root),
            CargoLine::BuildFinished(true)
        );
        assert_eq!(
            parse_cargo_line(r#"{"reason":"compiler-artifact","package_id":"x"}"#, root),
            CargoLine::Ignored
        );
        assert_eq!(parse_cargo_line("{pas du json", root), CargoLine::Text);
    }

    #[test]
    fn lint_keys() {
        let mut d = Diagnostic {
            level: Level::Warning,
            code: Some("unused_variables".into()),
            message: String::new(),
            span: None,
            rendered: String::new(),
            suggestions: vec![],
            package: None,
        };
        assert_eq!(
            d.explanation_key().as_deref(),
            Some("lint-unused_variables")
        );
        d.code = Some("clippy::needless_return".into());
        assert_eq!(d.explanation_key(), None);
        assert!(d.doc_url().unwrap().contains("needless_return"));
    }

    #[test]
    fn package_names() {
        assert_eq!(package_name("path+file:///x/mon-app#0.1.0"), "mon-app");
        assert_eq!(package_name("registry+https://x#serde@1.0.0"), "serde");
        assert_eq!(package_name("mon-app 0.1.0 (path+file:///x)"), "mon-app");
    }
}
