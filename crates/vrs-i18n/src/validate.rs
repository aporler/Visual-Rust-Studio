//! Validation des traductions : syntaxe, doublons, clés absentes, paramètres.
//!
//! Utilisé par l'outil `vrs-i18n-check` et par les tests : les traducteurs
//! peuvent vérifier leurs fichiers sans connaître Rust.

use std::collections::{BTreeMap, BTreeSet};

use fluent_syntax::ast;
use unic_langid::LanguageIdentifier;

use crate::Catalog;

/// Gravité d'un problème.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Traduction incomplète : le repli s'applique.
    Info,
    /// Probablement une erreur (clé inconnue, paramètre différent).
    Warning,
    /// Le fichier est invalide ou ambigu.
    Error,
}

/// Problème détecté.
#[derive(Clone, Debug)]
pub struct Issue {
    pub severity: Severity,
    pub locale: String,
    pub file: String,
    pub key: Option<String>,
    pub message: String,
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sev = match self.severity {
            Severity::Info => "info",
            Severity::Warning => "avertissement",
            Severity::Error => "erreur",
        };
        write!(f, "[{sev}] {}/{}", self.locale, self.file)?;
        if let Some(k) = &self.key {
            write!(f, " ({k})")?;
        }
        write!(f, " : {}", self.message)
    }
}

/// Informations extraites d'une langue : message → (fichier, variables).
type MessageInfo = BTreeMap<String, (String, BTreeSet<String>)>;

fn collect(catalog: &Catalog, lang: &LanguageIdentifier, issues: &mut Vec<Issue>) -> MessageInfo {
    let mut out: MessageInfo = BTreeMap::new();
    let locale = lang.to_string();
    for file in catalog.files(lang) {
        let resource = match fluent_syntax::parser::parse(file.source.as_str()) {
            Ok(r) => r,
            Err((r, errors)) => {
                for e in errors {
                    issues.push(Issue {
                        severity: Severity::Error,
                        locale: locale.clone(),
                        file: file.file_name.clone(),
                        key: None,
                        message: format!("erreur de syntaxe : {:?} (octets {:?})", e.kind, e.pos),
                    });
                }
                r
            }
        };
        let mut seen_in_file = BTreeSet::new();
        for entry in &resource.body {
            let ast::Entry::Message(msg) = entry else {
                continue;
            };
            let id = msg.id.name.to_string();
            let mut vars = BTreeSet::new();
            if let Some(p) = &msg.value {
                pattern_vars(p, &mut vars);
            }
            for attr in &msg.attributes {
                pattern_vars(&attr.value, &mut vars);
            }
            if !seen_in_file.insert(id.clone()) {
                issues.push(Issue {
                    severity: Severity::Error,
                    locale: locale.clone(),
                    file: file.file_name.clone(),
                    key: Some(id.clone()),
                    message: "clé définie plusieurs fois dans le même fichier".into(),
                });
            } else if let Some((other, _)) = out.get(&id)
                && *other != file.file_name
                && file.origin == crate::ResourceOrigin::Embedded
            {
                issues.push(Issue {
                    severity: Severity::Warning,
                    locale: locale.clone(),
                    file: file.file_name.clone(),
                    key: Some(id.clone()),
                    message: format!("clé déjà définie dans {other}"),
                });
            }
            if msg.value.is_none() && msg.attributes.is_empty() {
                issues.push(Issue {
                    severity: Severity::Error,
                    locale: locale.clone(),
                    file: file.file_name.clone(),
                    key: Some(id.clone()),
                    message: "message sans valeur".into(),
                });
            }
            out.insert(id, (file.file_name.clone(), vars));
        }
    }
    out
}

fn pattern_vars(p: &ast::Pattern<&str>, vars: &mut BTreeSet<String>) {
    for el in &p.elements {
        if let ast::PatternElement::Placeable { expression } = el {
            expr_vars(expression, vars);
        }
    }
}

fn expr_vars(e: &ast::Expression<&str>, vars: &mut BTreeSet<String>) {
    match e {
        ast::Expression::Inline(i) => inline_vars(i, vars),
        ast::Expression::Select { selector, variants } => {
            inline_vars(selector, vars);
            for v in variants {
                pattern_vars(&v.value, vars);
            }
        }
    }
}

fn inline_vars(i: &ast::InlineExpression<&str>, vars: &mut BTreeSet<String>) {
    match i {
        ast::InlineExpression::VariableReference { id } => {
            vars.insert(id.name.to_string());
        }
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline_vars(a, vars);
            }
            for n in &arguments.named {
                inline_vars(&n.value, vars);
            }
        }
        ast::InlineExpression::Placeable { expression } => expr_vars(expression, vars),
        _ => {}
    }
}

/// Valide toutes les langues du catalogue par rapport à la langue de référence.
pub fn validate(catalog: &Catalog, reference: &LanguageIdentifier) -> Vec<Issue> {
    let mut issues = Vec::new();
    let reference_info = collect(catalog, reference, &mut issues);
    if reference_info.is_empty() {
        issues.push(Issue {
            severity: Severity::Error,
            locale: reference.to_string(),
            file: "*".into(),
            key: None,
            message: "la langue de référence est vide".into(),
        });
        return issues;
    }
    for lang in catalog.available() {
        if &lang == reference {
            continue;
        }
        let info = collect(catalog, &lang, &mut issues);
        let locale = lang.to_string();
        if !info.contains_key("language-name") {
            issues.push(Issue {
                severity: Severity::Warning,
                locale: locale.clone(),
                file: "*".into(),
                key: Some("language-name".into()),
                message: "le nom de la langue n'est pas défini".into(),
            });
        }
        let mut missing = 0usize;
        for (key, (_, ref_vars)) in &reference_info {
            match info.get(key) {
                None => missing += 1,
                Some((file, vars)) => {
                    let unknown: Vec<_> = vars.difference(ref_vars).cloned().collect();
                    if !unknown.is_empty() {
                        issues.push(Issue {
                            severity: Severity::Warning,
                            locale: locale.clone(),
                            file: file.clone(),
                            key: Some(key.clone()),
                            message: format!(
                                "paramètres inconnus de la référence : {}",
                                unknown.join(", ")
                            ),
                        });
                    }
                    let absent: Vec<_> = ref_vars.difference(vars).cloned().collect();
                    if !absent.is_empty() {
                        issues.push(Issue {
                            severity: Severity::Info,
                            locale: locale.clone(),
                            file: file.clone(),
                            key: Some(key.clone()),
                            message: format!("paramètres non utilisés : {}", absent.join(", ")),
                        });
                    }
                }
            }
        }
        for (key, (file, _)) in &info {
            if !reference_info.contains_key(key) {
                issues.push(Issue {
                    severity: Severity::Warning,
                    locale: locale.clone(),
                    file: file.clone(),
                    key: Some(key.clone()),
                    message: format!("clé absente de {reference} (inutilisée ?)"),
                });
            }
        }
        if missing > 0 {
            issues.push(Issue {
                severity: Severity::Info,
                locale: locale.clone(),
                file: "*".into(),
                key: None,
                message: format!(
                    "{missing} clé(s) non traduite(s) sur {} — repli appliqué",
                    reference_info.len()
                ),
            });
        }
    }
    issues
}

/// Statistiques de couverture : (langue, clés traduites, total de référence).
pub fn coverage(catalog: &Catalog, reference: &LanguageIdentifier) -> Vec<(String, usize, usize)> {
    let mut sink = Vec::new();
    let reference_info = collect(catalog, reference, &mut sink);
    catalog
        .available()
        .iter()
        .map(|l| {
            let info = collect(catalog, l, &mut sink);
            let n = reference_info
                .keys()
                .filter(|k| info.contains_key(*k))
                .count();
            (l.to_string(), n, reference_info.len())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResourceOrigin;

    #[test]
    fn embedded_catalog_has_no_errors_or_warnings() {
        let c = Catalog::embedded();
        let issues = validate(&c, &crate::default_locale());
        let bad: Vec<_> = issues
            .iter()
            .filter(|i| i.severity >= Severity::Warning)
            .map(|i| i.to_string())
            .collect();
        assert!(bad.is_empty(), "{bad:#?}");
    }

    #[test]
    fn fr_ca_is_complete() {
        let c = Catalog::embedded();
        let cov = coverage(&c, &crate::default_locale());
        let fr = cov.iter().find(|(l, ..)| l == "fr-CA").unwrap();
        assert_eq!(fr.1, fr.2, "fr-CA doit être complet");
    }

    #[test]
    fn detects_problems() {
        let mut c = Catalog::new();
        let en: LanguageIdentifier = "en-US".parse().unwrap();
        let xx: LanguageIdentifier = "xx-YY".parse().unwrap();
        c.add(
            en.clone(),
            "main.ftl",
            ResourceOrigin::Embedded,
            "a = A { $n }\nb = B\n".into(),
        );
        c.add(
            xx,
            "main.ftl",
            ResourceOrigin::Embedded,
            "a = A { $m }\na = again\nc = C\nbroken = {\n".into(),
        );
        let issues = validate(&c, &en);
        let text: Vec<String> = issues.iter().map(|i| i.to_string()).collect();
        let joined = text.join("\n");
        assert!(joined.contains("syntaxe"), "{joined}");
        assert!(joined.contains("plusieurs fois"), "{joined}");
        assert!(joined.contains("absente de en-US"), "{joined}");
        assert!(joined.contains("non traduite"), "{joined}");
    }
}
