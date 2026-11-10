//! Identifiants Rust : validation et conversion.
//!
//! Les noms de contrôles deviennent des champs Rust (`snake_case`) et les noms
//! de fenêtres deviennent des types (`UpperCamelCase`). Ces règles garantissent
//! que le code généré compile sans avertissement de style.

/// Mots-clés réservés de Rust (édition 2024), y compris les mots réservés pour le futur.
pub const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "gen", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
    "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "union",
];

/// Est-ce un mot-clé réservé ?
pub fn is_keyword(s: &str) -> bool {
    RUST_KEYWORDS.contains(&s)
}

/// Identifiant Rust valide (ASCII) et non réservé.
pub fn is_valid_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    s != "_" && chars.all(|c| c == '_' || c.is_ascii_alphanumeric()) && !is_keyword(s)
}

/// Identifiant `snake_case` valide (utilisé pour les noms de contrôles et de modules).
pub fn is_snake_case_ident(s: &str) -> bool {
    is_valid_ident(s)
        && s.chars()
            .all(|c| c == '_' || c.is_ascii_lowercase() || c.is_ascii_digit())
        && !s.contains("__")
}

/// Identifiant `UpperCamelCase` valide (utilisé pour les noms de fenêtres).
pub fn is_upper_camel_ident(s: &str) -> bool {
    is_valid_ident(s)
        && s.starts_with(|c: char| c.is_ascii_uppercase())
        && s.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Convertit `Form1`, `MainWindow`, `textBox1` en `form1`, `main_window`, `text_box1`.
pub fn to_snake_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let prev = i.checked_sub(1).map(|j| chars[j]);
            let next = chars.get(i + 1).copied();
            let boundary = match prev {
                Some(p) if p.is_ascii_lowercase() || p.is_ascii_digit() => true,
                Some(p) if p.is_ascii_uppercase() => next.is_some_and(|n| n.is_ascii_lowercase()),
                _ => false,
            };
            if boundary && !out.ends_with('_') {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

/// Nom de crate Cargo valide : lettres ASCII, chiffres, `-` et `_`, commençant par une lettre.
pub fn is_valid_crate_name(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && s.len() <= 64
        && !is_keyword(&s.replace('-', "_"))
        && !matches!(
            s,
            "test" | "core" | "std" | "alloc" | "proc_macro" | "proc-macro"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idents() {
        assert!(is_valid_ident("button1"));
        assert!(is_valid_ident("_x"));
        assert!(!is_valid_ident("1button"));
        assert!(!is_valid_ident("fn"));
        assert!(!is_valid_ident("_"));
        assert!(!is_valid_ident("été"));
        assert!(is_snake_case_ident("text_box1"));
        assert!(!is_snake_case_ident("TextBox1"));
        assert!(is_upper_camel_ident("Form1"));
        assert!(!is_upper_camel_ident("form1"));
        assert!(!is_upper_camel_ident("Form_1"));
    }

    #[test]
    fn snake_case() {
        assert_eq!(to_snake_case("Form1"), "form1");
        assert_eq!(to_snake_case("MainWindow"), "main_window");
        assert_eq!(to_snake_case("textBox1"), "text_box1");
        assert_eq!(to_snake_case("HTTPServer"), "http_server");
        assert_eq!(to_snake_case("Mon App"), "mon_app");
    }

    #[test]
    fn crate_names() {
        assert!(is_valid_crate_name("mon-app"));
        assert!(is_valid_crate_name("MonApp2"));
        assert!(!is_valid_crate_name("2app"));
        assert!(!is_valid_crate_name("mon app"));
        assert!(!is_valid_crate_name("std"));
        assert!(!is_valid_crate_name("fn"));
    }
}
