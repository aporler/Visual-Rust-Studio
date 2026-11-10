//! Aide à la saisie : autocomplétion, équivalences VB.NET et corrections rapides.
//!
//! Tout ce qui est proposé correspond à du Rust réel et à des API qui existent
//! dans le runtime `vrs` : rien n'est inventé.

use std::collections::BTreeMap;

/// Catégorie d'une proposition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemKind {
    /// Équivalence pour une habitude VB.NET (affichée en premier).
    Hint,
    Field,
    Method,
    Snippet,
    Keyword,
}

impl ItemKind {
    /// Petit symbole affiché dans la liste.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Hint => "★",
            Self::Field => "◦",
            Self::Method => "ƒ",
            Self::Snippet => "⚙",
            Self::Keyword => "#",
        }
    }
}

/// Une proposition d'autocomplétion.
#[derive(Clone, Debug, PartialEq)]
pub struct CompletionItem {
    /// Texte affiché.
    pub label: String,
    /// Texte inséré à la place du mot en cours. `$0` marque la position du curseur.
    pub insert: String,
    pub kind: ItemKind,
    /// Clé de traduction de la description, et ses paramètres.
    pub doc_key: String,
    pub doc_args: Vec<(String, String)>,
    /// Description brute (fournie par rust-analyzer), affichée telle quelle.
    pub doc_text: Option<String>,
}

impl CompletionItem {
    fn new(label: &str, insert: &str, kind: ItemKind, doc_key: &str) -> Self {
        Self {
            label: label.to_owned(),
            insert: insert.to_owned(),
            kind,
            doc_key: doc_key.to_owned(),
            doc_args: Vec::new(),
            doc_text: None,
        }
    }

    /// Proposition venant d'un serveur de langage (rust-analyzer).
    pub fn external(label: &str, insert: &str, kind: ItemKind, doc: Option<String>) -> Self {
        Self {
            doc_text: doc,
            ..Self::new(label, insert, kind, "")
        }
    }

    fn arg(mut self, k: &str, v: &str) -> Self {
        self.doc_args.push((k.to_owned(), v.to_owned()));
        self
    }
}

/// Correction rapide applicable à une portion de texte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fix {
    /// Texte de remplacement.
    pub replacement: String,
    /// Clé de traduction du libellé (`fix-replace`, `fix-apply-suggestion`).
    pub label_key: String,
    /// Portion remplacée (ligne début, col début, ligne fin, col fin ; 1-based),
    /// si elle diffère de celle du diagnostic.
    pub range: Option<(usize, usize, usize, usize)>,
}

/// Contrôle d'une fenêtre, tel que connu de l'éditeur : (nom du champ, type VRS).
pub type ControlInfo = (String, String);

/// Contexte d'autocomplétion d'un fichier.
#[derive(Clone, Debug, Default)]
pub struct CompletionContext {
    /// Contrôles de la fenêtre si le fichier est le code d'une fenêtre VRS.
    pub controls: Vec<ControlInfo>,
    /// Le fichier est-il le code d'une fenêtre (accès à `self` et `app`) ?
    pub is_form: bool,
    /// Fenêtres du projet (`Form1`, `Form2`…), pour `app.show_form(…)`.
    pub forms: Vec<String>,
}

/// `text_box1` → `TextBox1` (nom tel qu'on l'écrirait en VB.NET).
pub fn vb_style_name(field: &str) -> String {
    field
        .split('_')
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}

/// Équivalences des habitudes VB.NET : (mots VB en minuscules, proposition Rust).
fn vb_hints(ctx: &CompletionContext) -> Vec<(Vec<String>, CompletionItem)> {
    let mut v: Vec<(Vec<String>, CompletionItem)> = Vec::new();
    let words = |w: &[&str]| w.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    if ctx.is_form {
        v.push((
            words(&["close", "end", "exit", "quit"]),
            CompletionItem::new(
                "close() → app.exit()",
                "app.exit();",
                ItemKind::Hint,
                "hint-close",
            ),
        ));
        v.push((
            words(&["me"]),
            CompletionItem::new("Me → self", "self", ItemKind::Hint, "hint-me"),
        ));
        v.push((
            words(&["text", "caption", "title"]),
            CompletionItem::new(
                "Me.Text = … → app.set_title(…)",
                "app.set_title(\"$0\");",
                ItemKind::Hint,
                "hint-title",
            ),
        ));
        for f in &ctx.forms {
            for (vb, insert, key) in [
                (
                    "Show",
                    format!("app.show_form(super::{f}::default());"),
                    "hint-show-form",
                ),
                (
                    "ShowDialog",
                    format!("app.show_dialog(super::{f}::default());"),
                    "hint-show-dialog",
                ),
            ] {
                let label = format!("{f}.{vb}() → {}", insert.trim_end_matches(';'));
                v.push((
                    vec![f.to_lowercase()],
                    CompletionItem::new(&label, &insert, ItemKind::Hint, key).arg("form", f),
                ));
            }
        }
        for (name, kind) in &ctx.controls {
            let vb = vb_style_name(name);
            v.push((
                vec![vb.to_lowercase()],
                CompletionItem::new(
                    &format!("{vb} → self.{name}"),
                    &format!("self.{name}"),
                    ItemKind::Hint,
                    "hint-control-name",
                )
                .arg("vb", &vb)
                .arg("name", name)
                .arg("kind", kind),
            ));
        }
    }
    if ctx.is_form {
        v.push((
            words(&["msgbox", "messagebox"]),
            CompletionItem::new(
                "MsgBox(…) → app.message_box(…)",
                "app.message_box(\"$0\");",
                ItemKind::Hint,
                "hint-msgbox",
            ),
        ));
    }
    v.push((
        words(&["dim"]),
        CompletionItem::new("Dim → let", "let $0 = ", ItemKind::Hint, "hint-dim"),
    ));
    v.push((
        words(&["true"]),
        CompletionItem::new("True → true", "true", ItemKind::Hint, "hint-bool"),
    ));
    v.push((
        words(&["false"]),
        CompletionItem::new("False → false", "false", ItemKind::Hint, "hint-bool"),
    ));
    v.push((
        words(&["nothing", "null"]),
        CompletionItem::new("Nothing → None", "None", ItemKind::Hint, "hint-nothing"),
    ));
    v.push((
        words(&["console", "writeline", "print", "debug"]),
        CompletionItem::new(
            "Console.WriteLine → println!",
            "println!(\"$0\");",
            ItemKind::Hint,
            "hint-print",
        ),
    ));
    v
}

fn keywords() -> Vec<CompletionItem> {
    let mut v: Vec<CompletionItem> = [
        "let", "mut", "if", "else", "for", "in", "while", "loop", "match", "fn", "return", "break",
        "continue", "true", "false", "struct", "enum", "impl", "use", "pub", "const",
    ]
    .iter()
    .map(|k| CompletionItem::new(k, k, ItemKind::Keyword, "completion-keyword"))
    .collect();
    for (label, insert, key) in [
        ("println!", "println!(\"$0\");", "snippet-println"),
        ("format!", "format!(\"$0\")", "snippet-format"),
        (
            "String::from",
            "String::from(\"$0\")",
            "snippet-string-from",
        ),
        ("if … { }", "if $0 {\n}", "snippet-if"),
        ("for … in … { }", "for $0 in  {\n}", "snippet-for"),
        ("Some", "Some($0)", "snippet-some"),
        ("None", "None", "snippet-none"),
        ("Ok", "Ok($0)", "snippet-ok"),
        ("Err", "Err($0)", "snippet-err"),
        ("Vec::new", "Vec::new()", "snippet-vec-new"),
    ] {
        v.push(CompletionItem::new(label, insert, ItemKind::Snippet, key));
    }
    v
}

fn string_methods() -> Vec<CompletionItem> {
    [
        ("push_str", "push_str(\"$0\")", "api-string-push-str"),
        ("clear", "clear()", "api-string-clear"),
        ("len", "len()", "api-string-len"),
        ("is_empty", "is_empty()", "api-string-is-empty"),
        ("trim", "trim()", "api-string-trim"),
        ("to_uppercase", "to_uppercase()", "api-string-to-uppercase"),
        ("to_lowercase", "to_lowercase()", "api-string-to-lowercase"),
        ("contains", "contains(\"$0\")", "api-string-contains"),
        ("parse::<i32>", "parse::<i32>()", "api-string-parse"),
    ]
    .iter()
    .map(|(l, i, k)| CompletionItem::new(l, i, ItemKind::Method, k))
    .collect()
}

/// Membres disponibles après `receveur.` (chemin comme `self.text_box1`).
fn members(path: &str, ctx: &CompletionContext) -> Vec<CompletionItem> {
    let parts: Vec<&str> = path.split('.').collect();
    match parts.as_slice() {
        ["app"] if ctx.is_form => {
            let mut v = vec![
                CompletionItem::new("exit()", "exit()", ItemKind::Method, "api-app-exit"),
                CompletionItem::new("close()", "close()", ItemKind::Method, "api-app-close"),
                CompletionItem::new("hide()", "hide()", ItemKind::Method, "api-app-hide"),
                CompletionItem::new(
                    "message_box(…)",
                    "message_box(\"$0\");",
                    ItemKind::Method,
                    "api-app-message-box",
                ),
                CompletionItem::new(
                    "set_title(…)",
                    "set_title(\"$0\")",
                    ItemKind::Method,
                    "api-app-set-title",
                ),
                CompletionItem::new(
                    "set_size(…)",
                    "set_size($0)",
                    ItemKind::Method,
                    "api-app-set-size",
                ),
            ];
            for f in &ctx.forms {
                v.push(
                    CompletionItem::new(
                        &format!("show_form({f})"),
                        &format!("show_form(super::{f}::default())"),
                        ItemKind::Method,
                        "api-app-show-form",
                    )
                    .arg("form", f),
                );
                v.push(
                    CompletionItem::new(
                        &format!("show_dialog({f})"),
                        &format!("show_dialog(super::{f}::default())"),
                        ItemKind::Method,
                        "api-app-show-dialog",
                    )
                    .arg("form", f),
                );
            }
            v.push(CompletionItem::new(
                "show_hidden(…)",
                "show_hidden($0)",
                ItemKind::Method,
                "api-app-show-hidden",
            ));
            v.push(CompletionItem::new(
                "with_form(…)",
                "with_form::<$0, _>(id, |f| {})",
                ItemKind::Method,
                "api-app-with-form",
            ));
            v.push(CompletionItem::new(
                "egui_ctx()",
                "egui_ctx()",
                ItemKind::Method,
                "api-app-egui-ctx",
            ));
            v
        }
        ["self"] if ctx.is_form => ctx
            .controls
            .iter()
            .map(|(name, kind)| {
                CompletionItem::new(name, name, ItemKind::Field, "completion-control")
                    .arg("kind", kind)
            })
            .collect(),
        ["self", control] if ctx.controls.iter().any(|(n, _)| n == control) => {
            let kind = ctx
                .controls
                .iter()
                .find(|(n, _)| n == control)
                .map(|(_, k)| k.as_str())
                .unwrap_or_default();
            let mut v = vec![
                CompletionItem::new("text", "text", ItemKind::Field, "api-text"),
                CompletionItem::new("visible", "visible", ItemKind::Field, "api-visible"),
                CompletionItem::new("enabled", "enabled", ItemKind::Field, "api-enabled"),
                CompletionItem::new("tooltip", "tooltip", ItemKind::Field, "api-tooltip"),
                CompletionItem::new("bounds", "bounds", ItemKind::Field, "api-bounds"),
                CompletionItem::new(
                    "back_color",
                    "back_color = Some(vrs::Color::from_rgb($0));",
                    ItemKind::Field,
                    "api-back-color",
                ),
                CompletionItem::new(
                    "fore_color",
                    "fore_color = Some(vrs::Color::from_rgb($0));",
                    ItemKind::Field,
                    "api-fore-color",
                ),
                CompletionItem::new(
                    "font_size",
                    "font_size = Some($0);",
                    ItemKind::Field,
                    "api-font-size",
                ),
                CompletionItem::new("anchor", "anchor", ItemKind::Field, "api-anchor"),
            ];
            let extra: &[(&str, &str, ItemKind, &str)] = match kind {
                "CheckBox" => &[("checked", "checked", ItemKind::Field, "api-checked")],
                "RadioButton" => &[
                    ("checked", "checked", ItemKind::Field, "api-checked"),
                    ("group", "group", ItemKind::Field, "api-group"),
                ],
                "ComboBox" | "ListBox" => &[
                    ("items", "items", ItemKind::Field, "api-items"),
                    ("selected", "selected", ItemKind::Field, "api-selected"),
                    (
                        "selected_text()",
                        "selected_text()",
                        ItemKind::Method,
                        "api-selected-text",
                    ),
                ],
                "TabControl" => &[
                    ("pages", "pages", ItemKind::Field, "api-pages"),
                    ("selected", "selected", ItemKind::Field, "api-selected-page"),
                ],
                "PictureBox" => &[(
                    "image",
                    "image = Some(egui::include_image!(\"../../resources/$0\"));",
                    ItemKind::Field,
                    "api-image",
                )],
                "ProgressBar" | "Slider" => &[
                    ("value", "value", ItemKind::Field, "api-value"),
                    ("minimum", "minimum", ItemKind::Field, "api-minimum"),
                    ("maximum", "maximum", ItemKind::Field, "api-maximum"),
                ],
                _ => &[],
            };
            for (l, i, k, d) in extra.iter().rev() {
                v.insert(1, CompletionItem::new(l, i, *k, d));
            }
            v
        }
        ["self", control, "text" | "tooltip"] if ctx.controls.iter().any(|(n, _)| n == control) => {
            string_methods()
        }
        ["self", control, "bounds"] if ctx.controls.iter().any(|(n, _)| n == control) => {
            ["x", "y", "width", "height"]
                .iter()
                .map(|f| {
                    CompletionItem::new(f, f, ItemKind::Field, "api-bounds-field").arg("field", f)
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Analyse le texte avant le curseur : (début du mot en cours, mot partiel, receveur éventuel).
pub fn word_context(text: &str, cursor: usize) -> (usize, String, Option<String>) {
    let before: Vec<char> = text.chars().take(cursor).collect();
    let mut start = before.len();
    while start > 0 && is_ident_char(before[start - 1]) {
        start -= 1;
    }
    let partial: String = before[start..].iter().collect();
    let mut receiver = None;
    if start > 0 && before[start - 1] == '.' {
        let mut r = start - 1;
        while r > 0 && (is_ident_char(before[r - 1]) || before[r - 1] == '.') {
            r -= 1;
        }
        let path: String = before[r..start - 1].iter().collect();
        if !path.is_empty() && !path.starts_with('.') {
            receiver = Some(path);
        }
    }
    (start, partial, receiver)
}

/// Propositions pour la position du curseur.
pub fn complete(
    text: &str,
    cursor: usize,
    ctx: &CompletionContext,
) -> (usize, Vec<CompletionItem>) {
    let (start, partial, receiver) = word_context(text, cursor);
    let lower = partial.to_lowercase();
    let matches = |label: &str| lower.is_empty() || label.to_lowercase().starts_with(&lower);
    let mut items = Vec::new();
    if let Some(path) = receiver {
        // Après `Me.` : proposer les équivalents.
        if path.eq_ignore_ascii_case("me") && ctx.is_form {
            items.push(CompletionItem::new(
                "Me.Close() → app.exit()",
                "app.exit();",
                ItemKind::Hint,
                "hint-close",
            ));
            return (start - 3, items);
        }
        items.extend(
            members(&path, ctx)
                .into_iter()
                .filter(|i| matches(&i.label)),
        );
        return (start, items);
    }
    if partial.is_empty() {
        return (start, items);
    }
    for (words, item) in vb_hints(ctx) {
        if words
            .iter()
            .any(|w| w.starts_with(&lower) && lower.len() >= 2.min(w.len()))
        {
            items.push(item);
        }
    }
    if ctx.is_form {
        for (label, key) in [("self", "api-self"), ("app", "api-app")] {
            if matches(label) {
                items.push(CompletionItem::new(label, label, ItemKind::Keyword, key));
            }
        }
    }
    items.extend(keywords().into_iter().filter(|i| matches(&i.label)));
    // Pas de proposition identique au mot déjà tapé (sauf équivalences).
    items.retain(|i| i.kind == ItemKind::Hint || i.insert != partial);
    let mut seen = BTreeMap::new();
    items.retain(|i| seen.insert(i.label.clone(), ()).is_none());
    (start, items)
}

/// Applique une proposition : remplace `[start, cursor)` et retourne la nouvelle position du curseur.
pub fn apply_completion(text: &mut String, start: usize, cursor: usize, insert: &str) -> usize {
    let (insert, caret) = match insert.find("$0") {
        Some(i) => (insert.replace("$0", ""), insert[..i].chars().count()),
        None => (insert.to_owned(), insert.chars().count()),
    };
    crate::text::replace_chars(text, start, cursor, &insert);
    start + caret
}

/// Suggestion trouvée dans le code (habitude VB.NET) : (ligne, col début, col fin, clé du message, correction).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lint {
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub message_key: String,
    pub fix: Fix,
}

/// Repère les habitudes VB.NET qui ne compilent pas en Rust, avec leur correction.
/// Les chaînes et les commentaires sont ignorés.
pub fn lint_vb_habits(text: &str, ctx: &CompletionContext) -> Vec<Lint> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let code = strip_strings_and_comments(raw);
        let chars: Vec<char> = code.chars().collect();
        let mut j = 0;
        while j < chars.len() {
            if !is_ident_char(chars[j]) || (j > 0 && is_ident_char(chars[j - 1])) {
                j += 1;
                continue;
            }
            let mut k = j;
            while k < chars.len() && is_ident_char(chars[k]) {
                k += 1;
            }
            let word: String = chars[j..k].iter().collect();
            let prev = chars[..j]
                .iter()
                .rev()
                .find(|c| !c.is_whitespace())
                .copied();
            let after_dot = prev == Some('.');
            let after_fn = code[..code.char_indices().nth(j).map_or(code.len(), |(b, _)| b)]
                .trim_end()
                .ends_with("fn");
            let rest: String = chars[k..].iter().collect();
            let call = rest.trim_start().starts_with('(');
            let mut push = |end: usize, key: &str, replacement: &str| {
                out.push(Lint {
                    line: i + 1,
                    col_start: j + 1,
                    col_end: end + 1,
                    message_key: key.to_owned(),
                    fix: Fix {
                        replacement: replacement.to_owned(),
                        label_key: "fix-replace".into(),
                        range: None,
                    },
                });
            };
            if !after_dot && !after_fn {
                match word.as_str() {
                    "close" | "Close" | "End" if ctx.is_form && (call || word == "End") => {
                        // Remplace `close()` (avec les parenthèses).
                        let mut end = k;
                        if call {
                            let open = k + rest.find('(').unwrap_or(0);
                            if let Some(close) = chars[open..].iter().position(|c| *c == ')') {
                                end = open + close + 1;
                            }
                        }
                        push(end, "hint-close", "app.exit()");
                    }
                    "Me" if ctx.is_form => {
                        // `Me.Close()` → `app.exit()` ; `Me` → `self`.
                        let tail: String = chars[k..].iter().collect();
                        if tail.starts_with(".Close()") {
                            push(k + ".Close()".len(), "hint-close", "app.exit()");
                        } else if tail.starts_with(".Hide()") {
                            push(k + ".Hide()".len(), "hint-hide", "app.hide()");
                        } else {
                            push(k, "hint-me", "self");
                        }
                    }
                    "MsgBox" if ctx.is_form && call => push(k, "hint-msgbox", "app.message_box"),
                    "True" => push(k, "hint-bool", "true"),
                    "False" => push(k, "hint-bool", "false"),
                    "Nothing" => push(k, "hint-nothing", "None"),
                    "Dim" => push(k, "hint-dim", "let"),
                    _ if ctx.is_form && ctx.forms.contains(&word) => {
                        let tail: String = chars[k..].iter().collect();
                        if tail.starts_with(".ShowDialog()") {
                            push(
                                k + ".ShowDialog()".len(),
                                "hint-show-dialog",
                                &format!("app.show_dialog(super::{word}::default())"),
                            );
                        } else if tail.starts_with(".Show()") {
                            push(
                                k + ".Show()".len(),
                                "hint-show-form",
                                &format!("app.show_form(super::{word}::default())"),
                            );
                        }
                    }
                    _ => {
                        if ctx.is_form
                            && let Some((name, _)) = ctx
                                .controls
                                .iter()
                                .find(|(n, _)| vb_style_name(n) == word && *n != word)
                        {
                            push(k, "hint-control-name", &format!("self.{name}"));
                        }
                    }
                }
            }
            j = k;
        }
    }
    out
}

/// Remplace le contenu des chaînes et des commentaires `//` par des espaces
/// (en conservant les positions des caractères).
fn strip_strings_and_comments(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_str = false;
    let mut prev = '\0';
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if !in_str && c == '/' && chars.get(i + 1) == Some(&'/') {
            out.extend(std::iter::repeat_n(' ', chars.len() - i));
            break;
        }
        if c == '"' && prev != '\\' {
            in_str = !in_str;
            out.push('"');
        } else if in_str {
            out.push(' ');
        } else {
            out.push(c);
        }
        prev = c;
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form_ctx() -> CompletionContext {
        CompletionContext {
            controls: vec![
                ("button1".into(), "Button".into()),
                ("text_box1".into(), "TextBox".into()),
            ],
            is_form: true,
            forms: vec!["Form1".into(), "Form2".into()],
        }
    }

    #[test]
    fn multi_window_hints() {
        let ctx = form_ctx();
        let t = "app.show";
        let (_, items) = complete(t, t.len(), &ctx);
        assert!(labels(&items).contains(&"show_form(Form2)"));
        assert!(labels(&items).contains(&"show_dialog(Form2)"));
        let t = "Form2";
        let (_, items) = complete(t, t.len(), &ctx);
        assert_eq!(items[0].insert, "app.show_form(super::Form2::default());");
        let lints = lint_vb_habits("    Form2.ShowDialog();\n    Me.Hide();\n", &ctx);
        assert_eq!(
            lints[0].fix.replacement,
            "app.show_dialog(super::Form2::default())"
        );
        assert_eq!(lints[1].fix.replacement, "app.hide()");
    }

    fn labels(items: &[CompletionItem]) -> Vec<&str> {
        items.iter().map(|i| i.label.as_str()).collect()
    }

    #[test]
    fn close_suggests_app_exit() {
        let text = "        clo";
        let (start, items) = complete(text, text.chars().count(), &form_ctx());
        assert_eq!(start, 8);
        assert_eq!(items[0].insert, "app.exit();");
        assert_eq!(items[0].kind, ItemKind::Hint);
    }

    #[test]
    fn members_of_self_and_controls() {
        let ctx = form_ctx();
        let t = "self.";
        let (_, items) = complete(t, 5, &ctx);
        assert_eq!(labels(&items), ["button1", "text_box1"]);
        let t = "self.text_box1.te";
        let (start, items) = complete(t, t.len(), &ctx);
        assert_eq!(start, 15);
        assert_eq!(labels(&items), ["text"]);
        let t = "self.text_box1.text.";
        let (_, items) = complete(t, t.len(), &ctx);
        assert!(labels(&items).contains(&"push_str"));
        let t = "app.";
        let (_, items) = complete(t, t.len(), &ctx);
        assert!(labels(&items).starts_with(&[
            "exit()",
            "close()",
            "hide()",
            "message_box(…)",
            "set_title(…)"
        ]));
        // Fichier ordinaire : ni self ni app de VRS.
        let (_, items) = complete(t, t.len(), &CompletionContext::default());
        assert!(items.is_empty());
    }

    #[test]
    fn vb_control_names_and_keywords() {
        let ctx = form_ctx();
        let t = "TextB";
        let (_, items) = complete(t, t.len(), &ctx);
        assert_eq!(items[0].insert, "self.text_box1");
        let t = "pri";
        let (_, items) = complete(t, t.len(), &ctx);
        assert!(labels(&items).contains(&"println!"));
        let t = "Me.";
        let (start, items) = complete(t, t.len(), &ctx);
        assert_eq!(start, 0);
        assert_eq!(items[0].insert, "app.exit();");
    }

    #[test]
    fn apply_with_cursor_marker() {
        let mut s = "    pri".to_string();
        let c = apply_completion(&mut s, 4, 7, "println!(\"$0\");");
        assert_eq!(s, "    println!(\"\");");
        assert_eq!(c, 14);
    }

    #[test]
    fn lints_vb_habits() {
        let ctx = form_ctx();
        let text = "fn a() {\n    close();\n    let x = True; // close()\n    TextBox1.text = \"close()\".into();\n    Me.Close();\n    self.close();\n}\nfn close() {}\n";
        let lints = lint_vb_habits(text, &ctx);
        let summary: Vec<(usize, usize, usize, &str)> = lints
            .iter()
            .map(|l| (l.line, l.col_start, l.col_end, l.fix.replacement.as_str()))
            .collect();
        assert_eq!(
            summary,
            vec![
                (2, 5, 12, "app.exit()"),
                (3, 13, 17, "true"),
                (4, 5, 13, "self.text_box1"),
                (5, 5, 15, "app.exit()"),
            ]
        );
    }

    #[test]
    fn vb_names() {
        assert_eq!(vb_style_name("text_box1"), "TextBox1");
        assert_eq!(vb_style_name("button1"), "Button1");
    }
}
