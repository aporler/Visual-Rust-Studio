//! Conversions de positions dans le texte (lignes, colonnes, caractères).
//!
//! egui manipule des index de **caractères** ; rustc donne des lignes et des
//! colonnes 1-based (en caractères).

/// Index de caractère du début de la ligne `line` (1-based).
pub fn line_start_char(text: &str, line: usize) -> usize {
    if line <= 1 {
        return 0;
    }
    let mut count = 0;
    let mut current = 1;
    for c in text.chars() {
        count += 1;
        if c == '\n' {
            current += 1;
            if current == line {
                return count;
            }
        }
    }
    count
}

/// (ligne, colonne) 1-based → index de caractère, borné à la longueur de la ligne.
pub fn line_col_to_char(text: &str, line: usize, col: usize) -> usize {
    let start = line_start_char(text, line);
    let line_len = text.chars().skip(start).take_while(|c| *c != '\n').count();
    start + col.saturating_sub(1).min(line_len)
}

/// Index de caractère → (ligne, colonne) 1-based.
pub fn char_to_line_col(text: &str, index: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for c in text.chars().take(index) {
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Toutes les occurrences (index de caractères début, fin).
pub fn find_all(text: &str, needle: &str) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return Vec::new();
    }
    let n = needle.chars().count();
    let mut out = Vec::new();
    let mut char_pos = 0;
    let mut last_byte = 0;
    for (byte, _) in text.match_indices(needle) {
        char_pos += text[last_byte..byte].chars().count();
        out.push((char_pos, char_pos + n));
        char_pos += n;
        last_byte = byte + needle.len();
    }
    out
}

fn char_to_byte(text: &str, index: usize) -> usize {
    text.char_indices()
        .nth(index)
        .map_or(text.len(), |(b, _)| b)
}

/// Remplace les caractères `[a, b)` par `replacement`.
pub fn replace_chars(text: &mut String, a: usize, b: usize, replacement: &str) {
    let ba = char_to_byte(text, a);
    let bb = char_to_byte(text, b);
    text.replace_range(ba..bb, replacement);
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: &str = "fn main() {\n    let é = 1;\n}\n";

    #[test]
    fn positions() {
        assert_eq!(line_start_char(T, 1), 0);
        assert_eq!(line_start_char(T, 2), 12);
        assert_eq!(line_start_char(T, 3), 27);
        assert_eq!(line_col_to_char(T, 2, 9), 20);
        assert_eq!(T.chars().nth(20), Some('é'));
        assert_eq!(char_to_line_col(T, 20), (2, 9));
        // Colonne trop grande : bornée à la fin de la ligne.
        assert_eq!(line_col_to_char(T, 1, 999), 11);
    }

    #[test]
    fn find_and_replace() {
        let m = find_all("aé aé a", "aé");
        assert_eq!(m, vec![(0, 2), (3, 5)]);
        let mut s = "aé aé".to_string();
        replace_chars(&mut s, 3, 5, "x");
        assert_eq!(s, "aé x");
        assert!(find_all("abc", "").is_empty());
    }
}
