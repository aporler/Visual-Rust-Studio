//! Éditeur de code Rust de Visual Rust Studio.
//!
//! Fonctions actuelles : coloration syntaxique (syntect), numéros de ligne,
//! indentation automatique, recherche et remplacement, aller à une ligne,
//! soulignement des diagnostics de `cargo check` avec infobulle, mise en forme
//! par rustfmt.
//!
//! L'autocomplétion, la navigation vers les définitions et le renommage
//! viendront avec l'intégration de rust-analyzer (LSP), en phase 6.

mod text;

use std::path::{Path, PathBuf};

use egui::text::{CCursor, CCursorRange};
use egui::{Color32, FontId, Key, RichText, Stroke};
use vrs_i18n::I18n;

pub use text::{char_to_line_col, find_all, line_col_to_char, line_start_char};

/// Gravité d'un diagnostic affiché dans l'éditeur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Diagnostic à souligner (lignes et colonnes 1-based, colonnes en caractères).
#[derive(Clone, Debug)]
pub struct EditorDiagnostic {
    pub severity: Severity,
    pub line_start: usize,
    pub column_start: usize,
    pub line_end: usize,
    pub column_end: usize,
    pub message: String,
}

/// Options d'affichage.
#[derive(Clone, Debug)]
pub struct EditorOptions {
    pub font_size: f32,
    pub tab_width: usize,
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            tab_width: 4,
        }
    }
}

/// Résultat d'une image.
#[derive(Debug, Default)]
pub struct EditorOutput {
    /// Le texte a été modifié pendant cette image.
    pub changed: bool,
    /// Position du curseur (ligne, colonne), 1-based.
    pub cursor: Option<(usize, usize)>,
}

/// Un fichier ouvert dans l'éditeur.
pub struct CodeEditor {
    pub path: PathBuf,
    pub text: String,
    saved_text: String,
    /// Si le fichier est généré : nom du fichier où l'utilisateur doit écrire.
    pub generated_for: Option<String>,
    pub diagnostics: Vec<EditorDiagnostic>,
    find_open: bool,
    find_focus: bool,
    find_query: String,
    replace_text: String,
    goto_line: Option<usize>,
    pending_selection: Option<(usize, usize)>,
}

impl CodeEditor {
    /// Ouvre un fichier.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_text(path, text))
    }

    /// Crée un éditeur à partir d'un texte (fichier non lu sur le disque).
    pub fn from_text(path: &Path, text: String) -> Self {
        let generated_for = detect_generated(path, &text);
        Self {
            path: path.to_path_buf(),
            saved_text: text.clone(),
            text,
            generated_for,
            diagnostics: Vec::new(),
            find_open: false,
            find_focus: false,
            find_query: String::new(),
            replace_text: String::new(),
            goto_line: None,
            pending_selection: None,
        }
    }

    /// Nom du fichier.
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    /// Modifié depuis le dernier enregistrement ?
    pub fn is_dirty(&self) -> bool {
        self.text != self.saved_text
    }

    /// Enregistre (écriture atomique).
    pub fn save(&mut self) -> std::io::Result<()> {
        vrs_core::fs::atomic_write(&self.path, &self.text)?;
        self.saved_text = self.text.clone();
        Ok(())
    }

    /// Recharge depuis le disque si le fichier n'a pas été modifié dans l'éditeur.
    /// Retourne `true` si le texte a changé.
    pub fn reload_if_clean(&mut self) -> bool {
        if self.is_dirty() {
            return false;
        }
        match std::fs::read_to_string(&self.path) {
            Ok(t) if t != self.text => {
                self.text = t.clone();
                self.saved_text = t;
                true
            }
            _ => false,
        }
    }

    /// Place le curseur au début d'une ligne (1-based) et fait défiler jusqu'à elle.
    pub fn go_to_line(&mut self, line: usize) {
        self.goto_line = Some(line.max(1));
    }

    /// Ouvre la barre de recherche.
    pub fn open_find(&mut self) {
        self.find_open = true;
        self.find_focus = true;
    }

    /// Remplace tout le texte (ex. après rustfmt), en conservant l'historique d'enregistrement.
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    fn id(&self) -> egui::Id {
        egui::Id::unique(("vrs-code-editor", &self.path))
    }

    /// Affiche l'éditeur.
    pub fn show(&mut self, ui: &mut egui::Ui, i18n: &I18n, opts: &EditorOptions) -> EditorOutput {
        let mut out = EditorOutput::default();

        if let Some(user_file) = &self.generated_for {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(
                    ui.visuals().warn_fg_color,
                    format!(
                        "⚠ {}",
                        i18n.trf(
                            "editor-generated-warning",
                            &[("file", user_file.clone().into())]
                        )
                    ),
                );
            });
            ui.separator();
        }
        if self.find_open {
            self.find_bar(ui, i18n);
            ui.separator();
        }

        let font_id = FontId::monospace(opts.font_size);
        let theme = if ui.visuals().dark_mode {
            egui_extras::syntax_highlighting::CodeTheme::dark(opts.font_size)
        } else {
            egui_extras::syntax_highlighting::CodeTheme::light(opts.font_size)
        };
        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, _wrap_width: f32| {
            let mut job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                buf.as_str(),
                "rs",
            );
            job.wrap.max_width = f32::INFINITY;
            ui.fonts_mut(|f| f.layout_job(job))
        };

        let id = self.id();
        let line_count = self.text.split('\n').count().max(1);
        let digits = line_count.to_string().len().max(3);
        let diag_lines: Vec<(usize, Severity)> = self
            .diagnostics
            .iter()
            .map(|d| (d.line_start, d.severity))
            .collect();

        egui::ScrollArea::both()
            .id_salt(("editor-scroll", &self.path))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    // Gouttière : numéros de ligne.
                    let gutter: String = (1..=line_count)
                        .map(|i| format!("{i:>digits$}"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let gutter_resp = egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(4, 2))
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(gutter)
                                        .font(font_id.clone())
                                        .color(ui.visuals().weak_text_color()),
                                )
                                .selectable(false),
                            )
                        });
                    let row_h = ui.fonts_mut(|f| f.row_height(&font_id));
                    let gutter_rect = gutter_resp.response.rect;
                    for (line, sev) in &diag_lines {
                        let y = gutter_rect.top() + 2.0 + (*line as f32 - 0.5) * row_h;
                        ui.painter().circle_filled(
                            egui::pos2(gutter_rect.left() + 3.0, y),
                            2.5,
                            severity_color(*sev, ui.visuals()),
                        );
                    }

                    let mut output = egui::TextEdit::multiline(&mut self.text)
                        .id(id)
                        .font(font_id.clone())
                        .code_editor()
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .desired_rows(40)
                        .margin(egui::Margin::symmetric(4, 2))
                        .layouter(&mut layouter)
                        .show(ui);

                    if output.response.changed() {
                        out.changed = true;
                        post_edit(ui, &mut self.text, &mut output, opts.tab_width);
                    }

                    // Navigation demandée (aller à la ligne, résultat de recherche).
                    let mut select = self.pending_selection.take();
                    if let Some(line) = self.goto_line.take() {
                        let c = line_start_char(&self.text, line);
                        select = Some((c, c));
                    }
                    if let Some((a, b)) = select {
                        let range = CCursorRange::two(CCursor::new(a), CCursor::new(b));
                        output.state.cursor.set_char_range(Some(range));
                        output.state.clone().store(ui.ctx(), id);
                        ui.memory_mut(|m| m.request_focus(id));
                        let r = output.galley.pos_from_cursor(CCursor::new(a));
                        let screen = r.translate(output.galley_pos.to_vec2()).expand(row_h * 3.0);
                        ui.scroll_to_rect(screen, Some(egui::Align::Center));
                    }

                    // Soulignement des diagnostics.
                    for d in &self.diagnostics {
                        let a = line_col_to_char(&self.text, d.line_start, d.column_start);
                        let mut b = line_col_to_char(&self.text, d.line_end, d.column_end);
                        if b <= a {
                            b = a + 1;
                        }
                        let ra = output.galley.pos_from_cursor(CCursor::new(a));
                        let rb = output.galley.pos_from_cursor(CCursor::new(b));
                        let offset = output.galley_pos.to_vec2();
                        let color = severity_color(d.severity, ui.visuals());
                        let (x0, x1) = if (rb.min.y - ra.min.y).abs() < 1.0 {
                            (ra.min.x, rb.min.x.max(ra.min.x + 6.0))
                        } else {
                            (ra.min.x, ra.min.x + 60.0)
                        };
                        let y = ra.max.y + offset.y;
                        squiggle(ui.painter(), x0 + offset.x, x1 + offset.x, y, color);
                        let hover = egui::Rect::from_min_max(
                            egui::pos2(x0 + offset.x, ra.min.y + offset.y),
                            egui::pos2(x1 + offset.x, y + 2.0),
                        );
                        let rid = id.with(("diag", d.line_start, d.column_start));
                        ui.interact(hover, rid, egui::Sense::hover())
                            .on_hover_text(&d.message);
                    }

                    if let Some(range) = output.cursor_range {
                        out.cursor = Some(char_to_line_col(&self.text, range.primary.index.0));
                    }
                });
            });
        out
    }

    fn find_bar(&mut self, ui: &mut egui::Ui, i18n: &I18n) {
        let matches = find_all(&self.text, &self.find_query);
        ui.horizontal(|ui| {
            let r = ui.add(
                egui::TextEdit::singleline(&mut self.find_query)
                    .hint_text(i18n.tr("editor-find-placeholder"))
                    .desired_width(180.0),
            );
            if self.find_focus {
                r.request_focus();
                self.find_focus = false;
            }
            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            ui.add(
                egui::TextEdit::singleline(&mut self.replace_text)
                    .hint_text(i18n.tr("editor-replace-placeholder"))
                    .desired_width(160.0),
            );
            ui.label(i18n.trf("editor-matches", &[("total", matches.len().into())]));
            let current = self.pending_selection.map(|(a, _)| a).unwrap_or(0);
            if (ui
                .button("⏶")
                .on_hover_text(i18n.tr("button-previous"))
                .clicked())
                && !matches.is_empty()
            {
                let m = matches
                    .iter()
                    .rev()
                    .find(|(a, _)| *a < current)
                    .or(matches.last())
                    .copied();
                self.pending_selection = m;
            }
            if (ui
                .button("⏷")
                .on_hover_text(i18n.tr("button-next"))
                .clicked()
                || enter)
                && !matches.is_empty()
            {
                let m = matches
                    .iter()
                    .find(|(a, _)| *a > current)
                    .or(matches.first())
                    .copied();
                self.pending_selection = m;
                if enter {
                    r.request_focus();
                }
            }
            if ui.button(i18n.tr("button-replace")).clicked() && !matches.is_empty() {
                let (a, b) = matches
                    .iter()
                    .find(|(a, _)| *a >= current)
                    .or(matches.first())
                    .copied()
                    .expect("non vide");
                text::replace_chars(&mut self.text, a, b, &self.replace_text);
                let end = a + self.replace_text.chars().count();
                self.pending_selection = Some((end, end));
            }
            if ui.button(i18n.tr("button-replace-all")).clicked() && !self.find_query.is_empty() {
                self.text = self.text.replace(&self.find_query, &self.replace_text);
            }
            if ui
                .button("✖")
                .on_hover_text(i18n.tr("button-close"))
                .clicked()
                || ui.input(|i| i.key_pressed(Key::Escape))
            {
                self.find_open = false;
            }
        });
    }
}

/// Ajustements après une saisie : tabulation → espaces, indentation automatique.
fn post_edit(
    ui: &egui::Ui,
    text: &mut String,
    output: &mut egui::text_edit::TextEditOutput,
    tab_width: usize,
) {
    let Some(range) = output.cursor_range else {
        return;
    };
    let cursor = range.primary.index.0;
    let mut new_cursor = None;
    let before: String = text.chars().take(cursor).collect();

    if before.ends_with('\t') {
        let spaces = " ".repeat(tab_width);
        text::replace_chars(text, cursor - 1, cursor, &spaces);
        new_cursor = Some(cursor - 1 + tab_width);
    } else if before.ends_with('\n') && ui.input(|i| i.key_pressed(Key::Enter)) {
        let prev_line = before[..before.len() - 1]
            .rsplit('\n')
            .next()
            .unwrap_or_default();
        let mut indent: String = prev_line
            .chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .collect();
        if prev_line.trim_end().ends_with(['{', '(', '[']) {
            indent.push_str(&" ".repeat(tab_width));
        }
        // Ne pas doubler l'indentation si egui l'a déjà insérée.
        let after: String = text
            .chars()
            .skip(cursor)
            .take_while(|c| *c == ' ')
            .collect();
        if !indent.is_empty() && after.is_empty() {
            text::replace_chars(text, cursor, cursor, &indent);
            new_cursor = Some(cursor + indent.chars().count());
        }
    }

    if let Some(c) = new_cursor {
        output
            .state
            .cursor
            .set_char_range(Some(CCursorRange::one(CCursor::new(c))));
        output.state.clone().store(ui.ctx(), output.response.id);
    }
}

fn severity_color(s: Severity, v: &egui::Visuals) -> Color32 {
    match s {
        Severity::Error => v.error_fg_color,
        Severity::Warning => v.warn_fg_color,
        Severity::Info => Color32::from_rgb(80, 160, 255),
    }
}

/// Trait ondulé sous le texte.
fn squiggle(painter: &egui::Painter, x0: f32, x1: f32, y: f32, color: Color32) {
    let mut pts = Vec::new();
    let mut x = x0;
    let mut up = false;
    while x <= x1 {
        pts.push(egui::pos2(x, if up { y - 1.5 } else { y + 1.0 }));
        x += 3.0;
        up = !up;
    }
    if pts.len() >= 2 {
        painter.add(egui::Shape::line(pts, Stroke::new(1.2, color)));
    }
}

/// Le fichier est-il généré par VRS ? Retourne alors le nom du fichier utilisateur associé.
fn detect_generated(path: &Path, text: &str) -> Option<String> {
    if !text.starts_with("// ----") || !text.contains("<auto-generated> Visual Rust Studio") {
        return None;
    }
    let name = path.file_name()?.to_string_lossy().to_string();
    Some(match name.strip_suffix("_designer.rs") {
        Some(base) => format!("{base}.rs"),
        None => "src/main.rs".to_string(),
    })
}

/// Met en forme du code Rust avec rustfmt (édition 2024). Retourne le texte formaté
/// ou le message d'erreur de rustfmt.
pub fn format_with_rustfmt(rustfmt: &Path, source: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(rustfmt)
        .args(["--edition", "2024", "--emit", "stdout", "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: {e}", rustfmt.display()))?;
    let mut stdin = child.stdin.take().ok_or("stdin")?;
    let input = source.to_owned();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let _ = writer.join();
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_detection() {
        let gen_text = "// ----\n// <auto-generated> Visual Rust Studio\n";
        assert_eq!(
            detect_generated(Path::new("src/forms/form1_designer.rs"), gen_text).as_deref(),
            Some("form1.rs")
        );
        assert!(detect_generated(Path::new("src/forms/form1.rs"), "//! mon code").is_none());
    }

    #[test]
    fn save_and_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.rs");
        std::fs::write(&p, "fn main() {}\n").unwrap();
        let mut e = CodeEditor::open(&p).unwrap();
        assert!(!e.is_dirty());
        e.text.push_str("// x\n");
        assert!(e.is_dirty());
        e.save().unwrap();
        assert!(!e.is_dirty());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "fn main() {}\n// x\n");
        std::fs::write(&p, "changé").unwrap();
        assert!(e.reload_if_clean());
        assert_eq!(e.text, "changé");
    }

    #[test]
    fn renders_headless() {
        let i18n = I18n::with_embedded("fr-CA");
        let mut e = CodeEditor::from_text(
            Path::new("main.rs"),
            "fn main() {\n    let x: i32 = \"a\";\n}\n".into(),
        );
        e.diagnostics.push(EditorDiagnostic {
            severity: Severity::Error,
            line_start: 2,
            column_start: 18,
            line_end: 2,
            column_end: 21,
            message: "types incompatibles".into(),
        });
        e.open_find();
        e.go_to_line(2);
        let mut harness = egui_kittest::Harness::builder()
            .with_accessibility_check(false)
            .build_ui_state(
                |ui, e: &mut CodeEditor| {
                    e.show(ui, &i18n, &EditorOptions::default());
                },
                e,
            );
        harness.run();
        let e = harness.into_state();
        assert_eq!(e.text.lines().count(), 3);
    }

    #[test]
    fn rustfmt_formats_when_available() {
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        let candidates = [
            PathBuf::from(&home).join(".cargo/bin/rustfmt"),
            PathBuf::from("/usr/bin/rustfmt"),
        ];
        let Some(rustfmt) = candidates.iter().find(|p| p.is_file()) else {
            return;
        };
        let out = format_with_rustfmt(rustfmt, "fn  main( ){let x=1;}").unwrap();
        assert_eq!(out, "fn main() {\n    let x = 1;\n}\n");
        assert!(format_with_rustfmt(rustfmt, "fn main( {").is_err());
    }
}
