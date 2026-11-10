//! Conversions : chemins ↔ URI, colonnes en caractères ↔ encodage LSP, extraits.

use std::path::{Path, PathBuf};

/// Encodage des colonnes négocié avec le serveur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf16,
    Utf32,
}

/// `/home/x/a b.rs` → `file:///home/x/a%20b.rs`.
pub fn path_to_uri(path: &Path) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut s = abs.to_string_lossy().replace('\\', "/");
    if !s.starts_with('/') {
        s.insert(0, '/');
    }
    let mut out = String::from("file://");
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' | b':' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `file:///home/x/a%20b.rs` → `/home/x/a b.rs`.
pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    let s = String::from_utf8(out).ok()?;
    // Windows : `/C:/x` → `C:/x`.
    let s = if s.len() > 2 && s.as_bytes()[2] == b':' {
        s[1..].to_owned()
    } else {
        s
    };
    Some(PathBuf::from(s))
}

fn line_of(text: &str, line: usize) -> &str {
    text.split('\n').nth(line).unwrap_or_default()
}

/// Colonne en caractères → colonne dans l'encodage du serveur.
pub fn char_col_to_encoded(text: &str, line: usize, col: usize, enc: Encoding) -> usize {
    let l = line_of(text, line);
    l.chars()
        .take(col)
        .map(|c| match enc {
            Encoding::Utf8 => c.len_utf8(),
            Encoding::Utf16 => c.len_utf16(),
            Encoding::Utf32 => 1,
        })
        .sum::<usize>()
        + col.saturating_sub(l.chars().count())
}

/// Colonne dans l'encodage du serveur → colonne en caractères.
pub fn encoded_col_to_char(text: &str, line: usize, col: usize, enc: Encoding) -> usize {
    let mut units = 0;
    let mut chars = 0;
    for c in line_of(text, line).chars() {
        if units >= col {
            break;
        }
        units += match enc {
            Encoding::Utf8 => c.len_utf8(),
            Encoding::Utf16 => c.len_utf16(),
            Encoding::Utf32 => 1,
        };
        chars += 1;
    }
    chars
}

/// Retire les marqueurs d'extrait LSP : `foo(${1:x})$0` → `foo(x)$0` ;
/// le premier marqueur devient la position du curseur (`$0`).
pub fn strip_snippet(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut cursor_placed = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() {
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if c == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '{' {
                // ${n:texte} ou ${n}
                let mut j = i + 2;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                let mut depth = 1;
                let start = if j < chars.len() && chars[j] == ':' {
                    j + 1
                } else {
                    j
                };
                let mut k = start;
                while k < chars.len() {
                    if chars[k] == '{' {
                        depth += 1;
                    } else if chars[k] == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    k += 1;
                }
                let inner: String = chars[start.min(k)..k].iter().collect();
                if !cursor_placed {
                    out.push_str("$0");
                    cursor_placed = true;
                }
                out.push_str(&strip_snippet(&inner).replace("$0", ""));
                i = k + 1;
                continue;
            }
            if chars[i + 1].is_ascii_digit() {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
                if !cursor_placed {
                    out.push_str("$0");
                    cursor_placed = true;
                }
                i = j;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uris() {
        let p = PathBuf::from("/tmp/dossier été/a b.rs");
        let uri = path_to_uri(&p);
        assert_eq!(uri, "file:///tmp/dossier%20%C3%A9t%C3%A9/a%20b.rs");
        assert_eq!(uri_to_path(&uri).unwrap(), p);
    }

    #[test]
    fn columns() {
        let t = "let été = \"😀x\";";
        // « x » est le 13e caractère (index 12).
        assert_eq!(char_col_to_encoded(t, 0, 12, Encoding::Utf32), 12);
        assert_eq!(char_col_to_encoded(t, 0, 12, Encoding::Utf16), 13);
        assert_eq!(encoded_col_to_char(t, 0, 13, Encoding::Utf16), 12);
        assert_eq!(char_col_to_encoded(t, 0, 12, Encoding::Utf8), 17);
        assert_eq!(encoded_col_to_char(t, 0, 17, Encoding::Utf8), 12);
    }

    #[test]
    fn snippets() {
        assert_eq!(
            strip_snippet("push_str(${1:string})$0"),
            "push_str($0string)"
        );
        assert_eq!(strip_snippet("len()"), "len()");
        assert_eq!(strip_snippet("println!(\"$1\")$0"), "println!(\"$0\")");
    }
}
