//! Éditeur de code Rust de Visual Rust Studio.
//!
//! Fonctions : coloration syntaxique (syntect), numéros de ligne, indentation
//! automatique, recherche et remplacement, aller à une ligne, autocomplétion
//! (contrôles de la fenêtre, API `vrs`, mots-clés et extraits Rust),
//! équivalences VB.NET, corrections rapides, soulignement des diagnostics de
//! `cargo check` avec infobulle, mise en forme par rustfmt.
//!
//! La navigation vers les définitions et le renommage viendront avec
//! l'intégration de rust-analyzer (LSP).

pub mod assist;
mod text;

use std::path::{Path, PathBuf};

use assist::{CompletionContext, CompletionItem, Fix};
use egui::text::{CCursor, CCursorRange};
use egui::{Color32, FontId, Key, Modifiers, RichText, Stroke};
use vrs_i18n::{FluentArgs, I18n};

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
    /// Correction rapide proposée.
    pub fix: Option<Fix>,
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

/// Modification de texte : (début, fin, nouveau texte), positions
/// (ligne, colonne) 0-based en caractères.
pub type CharEdit = ((usize, usize), (usize, usize), String);

/// Demandes destinées au serveur de langage (rust-analyzer), lues par l'IDE.
/// Les positions sont des index de caractères dans le texte.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LspRequests {
    pub completion: Option<usize>,
    pub hover: Option<usize>,
    pub definition: Option<usize>,
    pub rename: Option<(usize, String)>,
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
    /// Contexte d'autocomplétion (contrôles de la fenêtre).
    pub completion: CompletionContext,
    /// Liste de propositions ouverte : index sélectionné.
    popup: Option<usize>,
    last_cursor: Option<usize>,
    lints: Vec<assist::Lint>,
    lints_text_len: usize,
    lints_dirty: bool,
    /// Menu de correction rapide ouvert : (diagnostic, position à l'écran).
    fix_menu: Option<(EditorDiagnostic, egui::Pos2)>,
    /// rust-analyzer est actif : ses diagnostics remplacent ceux de `cargo check`.
    pub lsp_active: bool,
    pub lsp_diagnostics: Vec<EditorDiagnostic>,
    lsp_requests: LspRequests,
    /// Propositions de rust-analyzer pour le mot commençant à cet index.
    external: Option<(usize, Vec<CompletionItem>)>,
    hover_char: Option<(usize, f64, bool)>,
    hover_info: Option<(usize, String)>,
    /// Renommage en cours : (index, nouveau nom, focus à donner).
    rename: Option<(usize, String, bool)>,
    /// Points d'arrêt (lignes 1-based).
    pub breakpoints: std::collections::BTreeSet<usize>,
    breakpoints_changed: bool,
    /// Ligne en cours d'exécution pendant le débogage (1-based).
    pub exec_line: Option<usize>,
    exec_scroll: bool,
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
            completion: CompletionContext::default(),
            popup: None,
            last_cursor: None,
            lints: Vec::new(),
            lints_text_len: usize::MAX,
            lints_dirty: true,
            fix_menu: None,
            lsp_active: false,
            lsp_diagnostics: Vec::new(),
            lsp_requests: LspRequests::default(),
            external: None,
            hover_char: None,
            hover_info: None,
            rename: None,
            breakpoints: Default::default(),
            breakpoints_changed: false,
            exec_line: None,
            exec_scroll: false,
        }
    }

    /// Ouvre la liste de propositions à la position du curseur (Ctrl+Espace).
    pub fn trigger_completion(&mut self) {
        self.popup = Some(0);
        // Nouvelle demande à rust-analyzer.
        self.external = None;
    }

    /// Propositions actuelles (début du mot remplacé, éléments).
    pub fn completion_items(&self) -> (usize, Vec<CompletionItem>) {
        let cursor = self.last_cursor.unwrap_or(self.text.chars().count());
        self.items_at(cursor)
    }

    /// Propositions de VRS, complétées par celles de rust-analyzer.
    fn items_at(&self, cursor: usize) -> (usize, Vec<CompletionItem>) {
        let (start, mut items) = assist::complete(&self.text, cursor, &self.completion);
        if let Some((at, ext)) = &self.external
            && *at == start
        {
            let (_, partial, _) = assist::word_context(&self.text, cursor);
            let lower = partial.to_lowercase();
            for e in ext {
                let matches = lower.is_empty() || e.label.to_lowercase().starts_with(&lower);
                if matches
                    && !items
                        .iter()
                        .any(|i| i.label == e.label || i.insert == e.insert)
                {
                    items.push(e.clone());
                }
            }
        }
        (start, items)
    }

    /// Demandes en attente pour le serveur de langage (vidées par l'appel).
    pub fn take_lsp_requests(&mut self) -> LspRequests {
        std::mem::take(&mut self.lsp_requests)
    }

    /// Reçoit les propositions de rust-analyzer demandées pour `cursor`.
    pub fn set_external_completions(&mut self, cursor: usize, items: Vec<CompletionItem>) {
        let (start, _, _) = assist::word_context(&self.text, cursor);
        self.external = Some((start, items));
    }

    /// Reçoit l'infobulle de survol pour le caractère `index`.
    pub fn set_hover(&mut self, index: usize, text: String) {
        self.hover_info = Some((index, text));
    }

    /// Applique des modifications (ligne, colonne 0-based en caractères), par
    /// exemple un renommage. Les modifications sont appliquées de la fin vers le début.
    pub fn apply_text_edits(&mut self, mut edits: Vec<CharEdit>) {
        edits.sort_by_key(|e| std::cmp::Reverse(e.0));
        for ((l0, c0), (l1, c1), new_text) in edits {
            let a = line_col_to_char(&self.text, l0 + 1, c0 + 1);
            let b = line_col_to_char(&self.text, l1 + 1, c1 + 1).max(a);
            text::replace_chars(&mut self.text, a, b, &new_text);
        }
        self.lints_dirty = true;
    }

    /// Ouvre la saisie du nouveau nom (F2) pour le symbole sous le curseur.
    pub fn start_rename(&mut self) {
        let cursor = self.last_cursor.unwrap_or(0);
        let (start, partial, _) = assist::word_context(&self.text, cursor);
        let rest: String = self
            .text
            .chars()
            .skip(cursor)
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        self.rename = Some((start, format!("{partial}{rest}"), true));
    }

    /// Active ou désactive un point d'arrêt (ligne 1-based).
    pub fn toggle_breakpoint(&mut self, line: usize) {
        if !self.breakpoints.remove(&line) {
            self.breakpoints.insert(line);
        }
        self.breakpoints_changed = true;
    }

    /// Indique la ligne en cours d'exécution et la rend visible.
    pub fn set_exec_line(&mut self, line: Option<usize>) {
        self.exec_scroll = line.is_some() && line != self.exec_line;
        self.exec_line = line;
    }

    /// Les points d'arrêt ont-ils changé depuis le dernier appel ?
    pub fn take_breakpoints_changed(&mut self) -> bool {
        std::mem::take(&mut self.breakpoints_changed)
    }

    /// Demande la définition du symbole sous le curseur (F12).
    pub fn go_to_definition(&mut self) {
        if let Some(c) = self.last_cursor {
            self.lsp_requests.definition = Some(c);
        }
    }

    /// Accepte la proposition `index`.
    pub fn accept_completion(&mut self, index: usize) {
        let cursor = self.last_cursor.unwrap_or(self.text.chars().count());
        let (start, items) = self.items_at(cursor);
        if let Some(item) = items.get(index) {
            let c = assist::apply_completion(&mut self.text, start, cursor, &item.insert);
            self.pending_selection = Some((c, c));
            self.last_cursor = Some(c);
            self.lints_dirty = true;
        }
        self.popup = None;
    }

    /// La liste de propositions est-elle ouverte ?
    pub fn completion_open(&self) -> bool {
        self.popup.is_some()
    }

    /// Place le curseur (index de caractère), comme si l'utilisateur avait cliqué.
    pub fn set_cursor(&mut self, index: usize) {
        self.last_cursor = Some(index);
        self.pending_selection = Some((index, index));
    }

    /// Habitudes VB.NET repérées dans le texte, sous forme de diagnostics avec correction.
    pub fn hint_diagnostics(&mut self, i18n: &I18n) -> Vec<EditorDiagnostic> {
        if self.lints_dirty || self.lints_text_len != self.text.len() {
            self.lints = assist::lint_vb_habits(&self.text, &self.completion);
            self.lints_text_len = self.text.len();
            self.lints_dirty = false;
        }
        self.lints
            .iter()
            .map(|l| EditorDiagnostic {
                severity: Severity::Info,
                line_start: l.line,
                column_start: l.col_start,
                line_end: l.line,
                column_end: l.col_end,
                message: i18n.trf(&l.message_key, &[("vb", l.fix.replacement.clone().into())]),
                fix: Some(l.fix.clone()),
            })
            .collect()
    }

    /// Applique la correction d'un diagnostic.
    pub fn apply_fix(&mut self, d: &EditorDiagnostic) {
        let Some(fix) = &d.fix else { return };
        let (l0, c0, l1, c1) =
            fix.range
                .unwrap_or((d.line_start, d.column_start, d.line_end, d.column_end));
        let a = line_col_to_char(&self.text, l0, c0);
        let b = line_col_to_char(&self.text, l1, c1).max(a);
        text::replace_chars(&mut self.text, a, b, &fix.replacement);
        let c = a + fix.replacement.chars().count();
        self.pending_selection = Some((c, c));
        self.lints_dirty = true;
        // Le diagnostic du compilateur ne correspond plus au texte.
        self.diagnostics
            .retain(|x| !(x.line_start == d.line_start && x.column_start == d.column_start));
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
        let has_focus = ui.memory(|m| m.has_focus(id));
        if has_focus && ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Space)) {
            self.trigger_completion();
        }
        if let (true, Some(sel)) = (has_focus, self.popup) {
            let n = self.completion_items().1.len();
            if n == 0 {
                // Avec rust-analyzer, les propositions peuvent arriver plus tard.
                if !self.lsp_active {
                    self.popup = None;
                }
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                self.popup = Some((sel + 1) % n);
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                self.popup = Some((sel + n - 1) % n);
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
                self.popup = None;
            } else if ui.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::Enter)
                    || i.consume_key(Modifiers::NONE, Key::Tab)
            }) {
                self.accept_completion(sel.min(n - 1));
                out.changed = true;
            }
        }
        if has_focus && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F9)) {
            let line = self
                .last_cursor
                .map(|c| char_to_line_col(&self.text, c).0)
                .unwrap_or(1);
            self.toggle_breakpoint(line);
        }
        if has_focus && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F12)) {
            self.go_to_definition();
        }
        if has_focus && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F2)) && self.lsp_active
        {
            self.start_rename();
        }
        let mut underlines = if self.lsp_active {
            self.lsp_diagnostics.clone()
        } else {
            self.diagnostics.clone()
        };
        underlines.extend(self.hint_diagnostics(i18n));
        let line_count = self.text.split('\n').count().max(1);
        let digits = line_count.to_string().len().max(3);
        let diag_lines: Vec<(usize, Severity)> = underlines
            .iter()
            .map(|d| (d.line_start, d.severity))
            .collect();
        let mut fix_request: Option<(EditorDiagnostic, egui::Pos2)> = None;

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
                            egui::pos2(gutter_rect.left() + 2.0, y),
                            2.0,
                            severity_color(*sev, ui.visuals()),
                        );
                    }
                    // Points d'arrêt : clic dans la gouttière.
                    let bp_resp =
                        ui.interact(gutter_rect, id.with("breakpoints"), egui::Sense::click());
                    if bp_resp.clicked()
                        && let Some(p) = bp_resp.interact_pointer_pos()
                    {
                        let line = ((p.y - gutter_rect.top() - 2.0) / row_h).floor() as usize + 1;
                        if line <= line_count {
                            self.toggle_breakpoint(line);
                        }
                    }
                    for line in &self.breakpoints {
                        let y = gutter_rect.top() + 2.0 + (*line as f32 - 0.5) * row_h;
                        ui.painter().circle_filled(
                            egui::pos2(gutter_rect.left() + 9.0, y),
                            (row_h * 0.3).min(5.0),
                            Color32::from_rgb(200, 40, 40),
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
                    let cursor_now = output.cursor_range.map(|r| r.primary.index.0);
                    if output.response.changed() {
                        // Ouvre les propositions après un caractère d'identifiant ou un point.
                        let typed = cursor_now
                            .and_then(|c| c.checked_sub(1))
                            .and_then(|c| self.text.chars().nth(c));
                        match typed {
                            Some(ch) if ch.is_alphanumeric() || ch == '_' || ch == '.' => {
                                if self.popup.is_none() {
                                    self.popup = Some(0);
                                }
                            }
                            _ => self.popup = None,
                        }
                    } else if cursor_now != self.last_cursor
                        && self.popup.is_some()
                        && cursor_now.is_some()
                    {
                        // Le curseur a bougé (clic, flèches) : fermer.
                        self.popup = None;
                    }
                    if cursor_now.is_some() {
                        self.last_cursor = cursor_now;
                    }
                    // rust-analyzer : propositions pour le mot en cours.
                    if self.lsp_active
                        && self.popup.is_some()
                        && (output.response.changed() || self.external.is_none())
                        && let Some(c) = cursor_now.or(self.last_cursor)
                    {
                        let (start, _, _) = assist::word_context(&self.text, c);
                        if output.response.changed()
                            || self.external.as_ref().is_none_or(|(s, _)| *s != start)
                        {
                            self.external = Some((start, Vec::new()));
                            self.lsp_requests.completion = Some(c);
                        }
                    }
                    // Survol : infobulle de rust-analyzer après un court délai ;
                    // Ctrl+clic : aller à la définition.
                    if self.lsp_active
                        && let Some(p) = output.response.hover_pos()
                    {
                        let local = p - output.galley_pos;
                        let idx = output.galley.cursor_from_pos(local).index.0;
                        let now = ui.input(|i| i.time);
                        match self.hover_char {
                            Some((i, t, requested)) if i == idx => {
                                if !requested && now - t > 0.5 {
                                    self.lsp_requests.hover = Some(idx);
                                    self.hover_char = Some((i, t, true));
                                } else if !requested {
                                    ui.ctx().request_repaint_after(
                                        std::time::Duration::from_millis(550),
                                    );
                                }
                            }
                            _ => {
                                self.hover_char = Some((idx, now, false));
                                ui.ctx()
                                    .request_repaint_after(std::time::Duration::from_millis(550));
                            }
                        }
                        if output.response.clicked() && ui.input(|i| i.modifiers.command) {
                            self.lsp_requests.definition = Some(idx);
                        }
                        if let Some((i, text)) = &self.hover_info
                            && *i == idx
                        {
                            let resp: egui::Response = (*output.response).clone();
                            resp.on_hover_ui_at_pointer(|ui| {
                                ui.set_max_width(520.0);
                                ui.label(
                                    RichText::new(text).monospace().size(opts.font_size * 0.85),
                                );
                            });
                        }
                    } else {
                        self.hover_char = None;
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

                    // Ligne en cours d'exécution (débogage).
                    if let Some(line) = self.exec_line {
                        let y = output.galley_pos.y + (line as f32 - 1.0) * row_h;
                        let r = egui::Rect::from_min_max(
                            egui::pos2(gutter_rect.left(), y),
                            egui::pos2(output.response.rect.right(), y + row_h),
                        );
                        ui.painter().rect_filled(
                            r,
                            0.0,
                            Color32::from_rgba_unmultiplied(255, 220, 0, 50),
                        );
                        ui.painter().text(
                            egui::pos2(gutter_rect.left() + 9.0, y + row_h / 2.0),
                            egui::Align2::CENTER_CENTER,
                            "▶",
                            FontId::proportional(row_h * 0.7),
                            Color32::from_rgb(230, 170, 0),
                        );
                        if self.exec_scroll {
                            self.exec_scroll = false;
                            ui.scroll_to_rect(r.expand(row_h * 3.0), Some(egui::Align::Center));
                        }
                    }

                    // Soulignement des diagnostics.
                    for d in &underlines {
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
                        let rid = id.with(("diag", d.line_start, d.column_start, d.column_end));
                        let sense = if d.fix.is_some() {
                            egui::Sense::click()
                        } else {
                            egui::Sense::hover()
                        };
                        let mut tip = d.message.clone();
                        if d.fix.is_some() {
                            tip.push_str(&format!("\n\n💡 {}", i18n.tr("editor-quick-fix-hint")));
                        }
                        let r = ui.interact(hover, rid, sense).on_hover_text(tip);
                        if r.clicked() {
                            fix_request = Some((d.clone(), hover.left_bottom()));
                        }
                    }

                    // Ampoule dans la gouttière pour les lignes qui ont une correction.
                    for d in underlines.iter().filter(|d| d.fix.is_some()) {
                        let y = gutter_rect.top() + 2.0 + (d.line_start as f32 - 1.0) * row_h;
                        let r = egui::Rect::from_min_size(
                            egui::pos2(gutter_rect.right() - row_h, y),
                            egui::vec2(row_h, row_h),
                        );
                        let resp = ui
                            .interact(
                                r,
                                id.with(("bulb", d.line_start, d.column_start)),
                                egui::Sense::click(),
                            )
                            .on_hover_text(i18n.tr("editor-quick-fix"));
                        ui.painter().text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            "💡",
                            FontId::proportional(row_h * 0.8),
                            ui.visuals().warn_fg_color,
                        );
                        if resp.clicked() {
                            fix_request = Some((d.clone(), r.right_bottom()));
                        }
                    }

                    // Liste de propositions sous le curseur.
                    if let (Some(sel), Some(c), true) = (
                        self.popup,
                        cursor_now.or(self.last_cursor),
                        has_focus || output.response.has_focus(),
                    ) {
                        let (_, items) = self.items_at(c);
                        if items.is_empty() {
                            if !self.lsp_active {
                                self.popup = None;
                            }
                        } else {
                            let pos = output.galley.pos_from_cursor(CCursor::new(c));
                            let anchor = output.galley_pos + pos.left_bottom().to_vec2();
                            if let Some(i) = completion_popup(
                                ui,
                                i18n,
                                id,
                                anchor,
                                &items,
                                sel.min(items.len() - 1),
                            ) {
                                self.accept_completion(i);
                                out.changed = true;
                            }
                        }
                    }

                    if let Some(range) = output.cursor_range {
                        out.cursor = Some(char_to_line_col(&self.text, range.primary.index.0));
                    }
                });
            });

        let just_opened = fix_request.is_some();
        if just_opened {
            self.fix_menu = fix_request;
        }
        if let Some((d, pos)) = self.fix_menu.clone() {
            let mut close = false;
            let area = egui::Area::new(id.with("fix-menu"))
                .order(egui::Order::Foreground)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        if let Some(fix) = &d.fix {
                            let label = i18n.trf(
                                &fix.label_key,
                                &[("replacement", fix.replacement.clone().into())],
                            );
                            if ui.button(format!("💡 {label}")).clicked() {
                                self.apply_fix(&d);
                                out.changed = true;
                                close = true;
                            }
                        }
                    });
                });
            if close
                || ui.input(|i| i.key_pressed(Key::Escape))
                || (ui.input(|i| i.pointer.any_click())
                    && !area.response.contains_pointer()
                    && !just_opened)
            {
                self.fix_menu = None;
            }
        }
        if let Some((start, mut name, focus)) = self.rename.take() {
            let mut keep = true;
            egui::Area::new(id.with("rename"))
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 60.0))
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(i18n.tr("editor-rename"));
                        let r = ui.add(egui::TextEdit::singleline(&mut name).desired_width(260.0));
                        if focus {
                            r.request_focus();
                        }
                        if r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                            if !name.trim().is_empty() {
                                self.lsp_requests.rename = Some((start, name.trim().to_owned()));
                            }
                            keep = false;
                        }
                        if ui.input(|i| i.key_pressed(Key::Escape)) {
                            keep = false;
                        }
                    });
                });
            if keep {
                self.rename = Some((start, name, false));
            }
        }
        if out.changed {
            self.lints_dirty = true;
        }
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

/// Affiche la liste de propositions. Retourne l'index cliqué.
fn completion_popup(
    ui: &egui::Ui,
    i18n: &I18n,
    id: egui::Id,
    anchor: egui::Pos2,
    items: &[CompletionItem],
    selected: usize,
) -> Option<usize> {
    let mut clicked = None;
    egui::Area::new(id.with("completion"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor + egui::vec2(0.0, 2.0))
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(260.0);
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        for (i, item) in items.iter().enumerate() {
                            let text =
                                RichText::new(format!("{}  {}", item.kind.icon(), item.label))
                                    .monospace();
                            let r = ui.selectable_label(i == selected, text);
                            if i == selected {
                                r.scroll_to_me(None);
                            }
                            if r.clicked() {
                                clicked = Some(i);
                            }
                        }
                    });
                if let Some(item) = items.get(selected) {
                    ui.separator();
                    let mut args = FluentArgs::new();
                    for (k, v) in &item.doc_args {
                        args.set(k.as_str(), v.clone());
                    }
                    ui.set_max_width(360.0);
                    let doc = match &item.doc_text {
                        Some(t) => t.clone(),
                        None if item.doc_key.is_empty() => String::new(),
                        None => i18n.tr_args(&item.doc_key, &args),
                    };
                    if !doc.is_empty() {
                        ui.label(RichText::new(doc).small());
                    }
                    ui.weak(RichText::new(i18n.tr("editor-completion-keys")).small());
                }
            });
        });
    clicked
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
            fix: None,
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
