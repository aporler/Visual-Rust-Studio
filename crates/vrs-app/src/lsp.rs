//! Intégration de rust-analyzer : diagnostics en direct, autocomplétion,
//! infobulles, aller à la définition, renommage.

use std::path::{Path, PathBuf};

use vrs_editor::assist::{CompletionItem, ItemKind};
use vrs_editor::{EditorDiagnostic, Severity, char_to_line_col, line_col_to_char};
use vrs_lsp::{LspClient, Pos};

use crate::app::{LineKind, VrsApp};
use crate::tabs::{Tab, same_path};

/// Requête envoyée au serveur, en attente de réponse.
#[derive(Clone, Debug)]
pub enum Pending {
    Completion { path: PathBuf, cursor: usize },
    Hover { path: PathBuf, index: usize },
    Definition,
    Rename,
}

/// rust-analyzer est-il installé et fonctionnel ?
pub fn find_rust_analyzer() -> Option<PathBuf> {
    let p = vrs_compiler::find_tool("rust-analyzer", None)?;
    // Le proxy rustup existe parfois sans le composant installé.
    let ok = std::process::Command::new(&p)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    ok.then_some(p)
}

fn pos_of(text: &str, index: usize) -> Pos {
    let (l, c) = char_to_line_col(text, index);
    Pos {
        line: l - 1,
        col: c - 1,
    }
}

fn completion_kind(kind: u32) -> ItemKind {
    match kind {
        2..=4 => ItemKind::Method,
        14 => ItemKind::Keyword,
        15 => ItemKind::Snippet,
        _ => ItemKind::Field,
    }
}

impl VrsApp {
    /// Démarre rust-analyzer pour le projet ouvert (si activé et installé).
    pub fn start_lsp(&mut self) {
        self.stop_lsp();
        if !self.settings.lsp_enabled {
            return;
        }
        let Some(project) = &self.project else { return };
        let Some(server) = find_rust_analyzer() else {
            log::info!("rust-analyzer introuvable : diagnostics par cargo check");
            return;
        };
        let root = project.root.clone();
        let trusted = self.created_here.contains(&root) || self.settings.is_trusted(&root);
        let ctx = self.repaint_ctx.clone();
        match LspClient::start(&server, &root, trusted, move || {
            if let Some(c) = &ctx {
                c.request_repaint();
            }
        }) {
            Ok(c) => {
                self.lsp = Some(c);
                self.lsp_generation = u64::MAX;
            }
            Err(e) => self.log(LineKind::Stderr, format!("rust-analyzer : {e}")),
        }
    }

    /// Arrête rust-analyzer.
    pub fn stop_lsp(&mut self) {
        if let Some(c) = self.lsp.take() {
            c.shutdown();
        }
        self.lsp_pending.clear();
        for t in &mut self.tabs {
            if let Tab::Code(e) = t {
                e.lsp_active = false;
                e.lsp_diagnostics.clear();
            }
        }
    }

    /// Le serveur est-il actif pour ce fichier ?
    fn lsp_handles(path: &Path) -> bool {
        path.extension().is_some_and(|e| e == "rs")
    }

    /// Signale l'enregistrement d'un fichier au serveur.
    pub fn lsp_did_save(&self, path: &Path) {
        if let Some(c) = &self.lsp
            && Self::lsp_handles(path)
        {
            c.did_save(path);
        }
    }

    /// Une image : synchronisation, requêtes, réponses, diagnostics.
    pub fn poll_lsp(&mut self) {
        let Some(client) = self.lsp.as_mut() else {
            return;
        };
        if client.has_exited() {
            self.lsp = None;
            self.log(LineKind::Stderr, "rust-analyzer s'est arrêté.");
            return;
        }
        if !client.is_ready() {
            return;
        }
        let client = self.lsp.as_ref().expect("présent");

        // Synchronisation des documents ouverts et demandes de l'onglet actif.
        let mut new_pending = Vec::new();
        for (i, t) in self.tabs.iter_mut().enumerate() {
            let Tab::Code(e) = t else { continue };
            if !Self::lsp_handles(&e.path) {
                continue;
            }
            e.lsp_active = true;
            if i != self.active && self.lsp_synced.contains(&e.path) {
                continue;
            }
            client.sync_document(&e.path, &e.text);
            self.lsp_synced.insert(e.path.clone());
            if i != self.active {
                continue;
            }
            let req = e.take_lsp_requests();
            if let Some(c) = req.completion
                && let Some(id) = client.request_completion(&e.path, pos_of(&e.text, c))
            {
                new_pending.push((
                    id,
                    Pending::Completion {
                        path: e.path.clone(),
                        cursor: c,
                    },
                ));
            }
            if let Some(h) = req.hover
                && let Some(id) = client.request_hover(&e.path, pos_of(&e.text, h))
            {
                new_pending.push((
                    id,
                    Pending::Hover {
                        path: e.path.clone(),
                        index: h,
                    },
                ));
            }
            if let Some(d) = req.definition
                && let Some(id) = client.request_definition(&e.path, pos_of(&e.text, d))
            {
                new_pending.push((id, Pending::Definition));
            }
            if let Some((at, name)) = req.rename
                && let Some(id) = client.request_rename(&e.path, pos_of(&e.text, at), &name)
            {
                new_pending.push((id, Pending::Rename));
            }
        }
        self.lsp_pending.extend(new_pending);

        // Réponses.
        let mut done = Vec::new();
        for (i, (id, _)) in self.lsp_pending.iter().enumerate() {
            if let Some(r) = client.take_response(*id) {
                done.push((i, r));
            }
        }
        for (i, r) in done.into_iter().rev() {
            let (_, pending) = self.lsp_pending.remove(i);
            let Ok(result) = r else { continue };
            self.handle_lsp_response(pending, &result);
        }

        // Diagnostics.
        let Some(client) = self.lsp.as_ref() else {
            return;
        };
        let generation = client.diagnostics(Path::new("")).0;
        if generation != self.lsp_generation {
            self.lsp_generation = generation;
            let updates: Vec<(usize, Vec<EditorDiagnostic>)> = self
                .tabs
                .iter()
                .enumerate()
                .filter_map(|(i, t)| match t {
                    Tab::Code(e) if Self::lsp_handles(&e.path) => {
                        Some((i, self.lsp_editor_diagnostics(&e.path)))
                    }
                    _ => None,
                })
                .collect();
            for (i, d) in updates {
                if let Some(Tab::Code(e)) = self.tabs.get_mut(i) {
                    e.lsp_diagnostics = d;
                }
            }
        }
    }

    fn lsp_editor_diagnostics(&self, path: &Path) -> Vec<EditorDiagnostic> {
        let Some(client) = &self.lsp else {
            return Vec::new();
        };
        client
            .diagnostics(path)
            .1
            .into_iter()
            .filter(|d| d.severity != vrs_lsp::Severity::Hint)
            .map(|d| {
                let mut message = d.message.clone();
                if let Some(code) = &d.code {
                    message = format!("[{code}] {message}");
                    let key =
                        if code.starts_with('E') && code[1..].chars().all(|c| c.is_ascii_digit()) {
                            Some(format!("rustc-{code}"))
                        } else if code.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                            Some(format!("lint-{code}"))
                        } else {
                            None
                        };
                    if let Some(expl) = key.and_then(|k| self.i18n.try_tr(&k, None)) {
                        message = format!("{message}\n\n{expl}");
                    }
                }
                if let Some(src) = &d.source {
                    message = format!("{message}\n\n({src})");
                }
                EditorDiagnostic {
                    severity: match d.severity {
                        vrs_lsp::Severity::Error => Severity::Error,
                        vrs_lsp::Severity::Warning => Severity::Warning,
                        _ => Severity::Info,
                    },
                    line_start: d.range.start.line + 1,
                    column_start: d.range.start.col + 1,
                    line_end: d.range.end.line + 1,
                    column_end: d.range.end.col + 1,
                    message,
                    fix: None,
                }
            })
            .collect()
    }

    fn handle_lsp_response(&mut self, pending: Pending, result: &serde_json::Value) {
        let Some(client) = &self.lsp else { return };
        match pending {
            Pending::Completion { path, cursor } => {
                let items: Vec<CompletionItem> = client
                    .parse_completion(result)
                    .into_iter()
                    .map(|c| {
                        let doc = match (&c.detail, &c.documentation) {
                            (Some(d), Some(doc)) => Some(format!("{d}\n{doc}")),
                            (Some(d), None) => Some(d.clone()),
                            (None, doc) => doc.clone(),
                        };
                        CompletionItem::external(&c.label, &c.insert, completion_kind(c.kind), doc)
                    })
                    .collect();
                for t in &mut self.tabs {
                    if let Tab::Code(e) = t
                        && e.path == path
                    {
                        e.set_external_completions(cursor, items.clone());
                    }
                }
            }
            Pending::Hover { path, index } => {
                if let Some(text) = client.parse_hover(result) {
                    for t in &mut self.tabs {
                        if let Tab::Code(e) = t
                            && e.path == path
                        {
                            e.set_hover(index, text.clone());
                        }
                    }
                }
            }
            Pending::Definition => {
                if let Some(loc) = client.parse_locations(result).into_iter().next() {
                    self.open_file(&loc.path, Some(loc.range.start.line + 1));
                }
            }
            Pending::Rename => {
                let edits = client.parse_workspace_edit(result);
                self.apply_workspace_edit(edits);
            }
        }
    }

    /// Applique un renommage : onglets ouverts (modifiés, à enregistrer) et
    /// fichiers fermés (copie `.bak`, écriture atomique).
    fn apply_workspace_edit(&mut self, edits: Vec<(PathBuf, Vec<vrs_lsp::TextEdit>)>) {
        for (path, list) in edits {
            let changes: Vec<vrs_editor::CharEdit> = list
                .into_iter()
                .map(|e| {
                    (
                        (e.range.start.line, e.range.start.col),
                        (e.range.end.line, e.range.end.col),
                        e.new_text,
                    )
                })
                .collect();
            let open = self.tabs.iter_mut().find_map(|t| match t {
                Tab::Code(e) if same_path(&e.path, &path) => Some(e),
                _ => None,
            });
            if let Some(e) = open {
                e.apply_text_edits(changes);
                continue;
            }
            let Ok(mut text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let mut sorted = changes;
            sorted.sort_by_key(|e| std::cmp::Reverse(e.0));
            for ((l0, c0), (l1, c1), new_text) in sorted {
                let a = line_col_to_char(&text, l0 + 1, c0 + 1);
                let b = line_col_to_char(&text, l1 + 1, c1 + 1).max(a);
                let ba = text.char_indices().nth(a).map_or(text.len(), |(i, _)| i);
                let bb = text.char_indices().nth(b).map_or(text.len(), |(i, _)| i);
                text.replace_range(ba..bb, &new_text);
            }
            let _ = vrs_core::fs::backup_file(&path);
            if let Err(e) = vrs_core::fs::atomic_write(&path, text) {
                self.log(LineKind::Stderr, format!("{}: {e}", path.display()));
            }
        }
    }
}
