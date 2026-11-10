//! Code écrit par l'utilisateur : création initiale et ajout de gestionnaires.
//!
//! Règle absolue : le code existant n'est **jamais** réécrit. VRS insère seulement
//! une nouvelle méthode avant l'accolade fermante du dernier bloc `impl` de la
//! fenêtre. Si le fichier contient une erreur de syntaxe, rien n'est modifié.

use vrs_components::FormDocument;

use crate::Comments;

/// Contenu initial de `src/forms/<fenêtre>.rs`.
pub fn user_file_skeleton(form: &FormDocument, comments: &dyn Comments) -> String {
    let doc = comments
        .user_file_doc(&form.name)
        .lines()
        .map(|l| format!("//! {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{doc}\n\
         #![allow(unused_imports, unused_variables)]\n\
         \n\
         use super::{name};\n\
         use crate::vrs::{{AppContext, egui}};\n\
         \n\
         impl {name} {{\n\
         }}\n",
        name = form.name
    )
}

/// Est-ce un bloc `impl Type { … }` (sans trait) ?
fn is_inherent_impl_of(item: &syn::ItemImpl, type_name: &str) -> bool {
    if item.trait_.is_some() {
        return false;
    }
    match &*item.self_ty {
        syn::Type::Path(p) => p.path.segments.last().is_some_and(|s| s.ident == type_name),
        _ => false,
    }
}

/// Convertit (ligne 1-based, colonne en caractères 0-based) en position d'octet.
fn byte_offset(src: &str, line: usize, column: usize) -> Option<usize> {
    let mut offset = 0;
    for (i, l) in src.split_inclusive('\n').enumerate() {
        if i + 1 == line {
            let col_bytes: usize = l.chars().take(column).map(char::len_utf8).sum();
            return Some(offset + col_bytes);
        }
        offset += l.len();
    }
    None
}

/// Ajoute la méthode `handler` à `impl <type_name>` si elle n'existe pas encore.
///
/// - `Ok(None)` : la méthode existe déjà, rien à faire.
/// - `Ok(Some(nouveau_source))` : la méthode a été ajoutée.
/// - `Err(message)` : le fichier ne peut pas être analysé ; il ne faut pas y toucher.
pub fn ensure_handler(
    src: &str,
    type_name: &str,
    handler: &str,
    doc: &str,
    todo: &str,
) -> Result<Option<String>, String> {
    let file = syn::parse_file(src).map_err(|e| {
        let start = e.span().start();
        format!("ligne {}, colonne {} : {e}", start.line, start.column + 1)
    })?;
    let impls: Vec<&syn::ItemImpl> = file
        .items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Impl(imp) if is_inherent_impl_of(imp, type_name) => Some(imp),
            _ => None,
        })
        .collect();
    let exists = impls.iter().any(|imp| {
        imp.items
            .iter()
            .any(|it| matches!(it, syn::ImplItem::Fn(f) if f.sig.ident == handler))
    });
    if exists {
        return Ok(None);
    }

    let method = format!(
        "    /// {doc}\n    pub fn {handler}(&mut self, app: &mut AppContext) {{\n        // {todo}\n    }}\n"
    );

    let new_src = match impls.last() {
        Some(imp) => {
            let close = imp.brace_token.span.close().start();
            let pos = byte_offset(src, close.line, close.column)
                .ok_or_else(|| "position de l'accolade introuvable".to_string())?;
            let line_start = src[..pos].rfind('\n').map_or(0, |i| i + 1);
            let before_on_line = &src[line_start..pos];
            let mut out = String::with_capacity(src.len() + method.len() + 2);
            if before_on_line.trim().is_empty() {
                // Cas habituel : `}` seul sur sa ligne.
                out.push_str(&src[..line_start]);
                let prev = src[..line_start].trim_end();
                if !prev.ends_with('{') {
                    out.push('\n');
                }
                out.push_str(&method);
                out.push_str(&src[line_start..]);
            } else {
                // `impl X { … }` sur une seule ligne.
                out.push_str(&src[..pos]);
                out.push('\n');
                out.push_str(&method);
                out.push_str(&src[pos..]);
            }
            out
        }
        None => {
            let mut out = src.trim_end().to_string();
            out.push_str(&format!("\n\nimpl {type_name} {{\n{method}}}\n"));
            out
        }
    };
    // Vérification : le résultat doit rester du Rust valide.
    syn::parse_file(&new_src).map_err(|e| format!("insertion invalide : {e}"))?;
    Ok(Some(new_src))
}

/// Ligne (1-based) de la déclaration `fn name`, pour ouvrir l'éditeur au bon endroit.
pub fn find_fn_line(src: &str, name: &str) -> Option<usize> {
    if let Ok(file) = syn::parse_file(src) {
        for item in &file.items {
            if let syn::Item::Impl(imp) = item {
                for it in &imp.items {
                    if let syn::ImplItem::Fn(f) = it
                        && f.sig.ident == name
                    {
                        return Some(f.sig.ident.span().start().line);
                    }
                }
            }
            if let syn::Item::Fn(f) = item
                && f.sig.ident == name
            {
                return Some(f.sig.ident.span().start().line);
            }
        }
    }
    // Repli textuel si le fichier contient une erreur de syntaxe.
    let needle = format!("fn {name}");
    src.lines()
        .position(|l| {
            l.find(&needle).is_some_and(|i| {
                !l[i + needle.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_')
            })
        })
        .map(|i| i + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EnglishComments;

    fn skeleton() -> String {
        user_file_skeleton(&FormDocument::new("Form1"), &EnglishComments)
    }

    #[test]
    fn skeleton_is_valid_rust() {
        syn::parse_file(&skeleton()).unwrap();
    }

    #[test]
    fn adds_missing_handler_once() {
        let src = skeleton();
        let a = ensure_handler(&src, "Form1", "button1_click", "Doc.", "TODO")
            .unwrap()
            .unwrap();
        assert!(a.contains("pub fn button1_click(&mut self, app: &mut AppContext)"));
        assert!(a.contains("    /// Doc.\n"));
        assert!(
            ensure_handler(&a, "Form1", "button1_click", "Doc.", "TODO")
                .unwrap()
                .is_none()
        );
        let b = ensure_handler(&a, "Form1", "text_box1_text_changed", "Doc.", "TODO")
            .unwrap()
            .unwrap();
        assert!(b.find("button1_click").unwrap() < b.find("text_box1_text_changed").unwrap());
        // Tout le texte d'origine est conservé tel quel, au caractère près.
        let a_lines: Vec<&str> = a.lines().collect();
        let b_lines: Vec<&str> = b.lines().collect();
        assert!(a_lines.iter().all(|l| b_lines.contains(l)));
    }

    #[test]
    fn existing_user_code_is_preserved() {
        let src = "//! Mon code — été\nuse super::Form1;\nuse crate::vrs::AppContext;\n\n\
                   impl Form1 {\n    pub fn button1_click(&mut self, app: &mut AppContext) {\n        \
                   self.text_box1.text = \"Bonjour\".into(); // é\n    }\n\n    fn helper(&self) -> u32 { 42 }\n}\n\n\
                   fn libre() {}\n";
        let out = ensure_handler(src, "Form1", "button2_click", "Doc.", "TODO")
            .unwrap()
            .unwrap();
        assert!(out.starts_with(&src[..src.rfind("}\n\nfn libre").unwrap()]));
        assert!(out.ends_with("}\n\nfn libre() {}\n"));
        assert!(
            out.contains(
                "fn helper(&self) -> u32 { 42 }\n\n    /// Doc.\n    pub fn button2_click"
            )
        );
    }

    #[test]
    fn single_line_impl_and_missing_impl() {
        let out = ensure_handler(
            "struct Form1;\nimpl Form1 {}\n",
            "Form1",
            "x_click",
            "D",
            "T",
        )
        .unwrap()
        .unwrap();
        syn::parse_file(&out).unwrap();
        assert!(out.contains("pub fn x_click"));
        let out = ensure_handler("struct Form1;\n", "Form1", "x_click", "D", "T")
            .unwrap()
            .unwrap();
        assert!(out.contains("impl Form1 {\n    /// D\n"));
        // Les impl de trait ne comptent pas.
        let out = ensure_handler(
            "struct Form1;\nimpl Default for Form1 { fn default() -> Self { Form1 } }\n",
            "Form1",
            "x_click",
            "D",
            "T",
        )
        .unwrap()
        .unwrap();
        assert!(out.contains("\nimpl Form1 {\n"));
    }

    #[test]
    fn broken_user_code_is_not_touched() {
        let err = ensure_handler("impl Form1 {\n    fn oops( {\n}\n", "Form1", "a", "D", "T")
            .unwrap_err();
        assert!(err.contains("ligne"));
    }

    #[test]
    fn find_lines() {
        let src = ensure_handler(&skeleton(), "Form1", "button1_click", "D", "T")
            .unwrap()
            .unwrap();
        let line = find_fn_line(&src, "button1_click").unwrap();
        assert!(
            src.lines()
                .nth(line - 1)
                .unwrap()
                .contains("fn button1_click")
        );
        assert_eq!(find_fn_line("fn ab() {}\nfn a() {", "a"), Some(2));
    }
}
