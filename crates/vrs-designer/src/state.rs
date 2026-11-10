//! État d'un onglet de conception.

use std::path::{Path, PathBuf};

use vrs_components::ops::{self, LayoutOp};
use vrs_components::{ComponentKind, Control, EventKind, FormDocument, FormError};

use crate::History;

/// Demande d'ouverture d'un gestionnaire d'événement (double-clic, bouton du
/// panneau des propriétés).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandlerRequest {
    /// Contrôle concerné ; `None` pour la fenêtre elle-même.
    pub control: Option<String>,
    pub event: EventKind,
}

/// Résultat d'une image du concepteur.
#[derive(Debug, Default)]
pub struct DesignerOutput {
    pub open_handler: Option<HandlerRequest>,
    /// Importer une image dans `resources/` pour ce contrôle (PictureBox).
    pub import_image: Option<String>,
    /// Le document a changé pendant cette image.
    pub changed: bool,
}

/// Opération de glisser en cours.
#[derive(Clone, Debug)]
pub(crate) enum DragOp {
    Move {
        start: Vec<(String, f32, f32)>,
        total: egui::Vec2,
    },
    Resize {
        name: String,
        handle: (i8, i8),
        start: [f32; 4],
        total: egui::Vec2,
    },
    ResizeForm {
        start: (f32, f32),
        total: egui::Vec2,
    },
}

/// Un formulaire ouvert dans le concepteur.
pub struct DesignerState {
    pub doc: FormDocument,
    pub path: PathBuf,
    /// Racine du projet (pour les ressources : images…).
    pub project_root: PathBuf,
    /// Noms des contrôles sélectionnés (le premier est la référence d'alignement).
    pub selection: Vec<String>,
    /// Composant choisi dans la boîte à outils, en attente de placement.
    pub pending_tool: Option<ComponentKind>,
    /// Aperçu interactif (les contrôles réagissent, sans gestionnaires).
    pub preview: bool,
    pub history: History,
    saved: FormDocument,
    pub(crate) drag: Option<DragOp>,
    /// Tampon d'édition du nom (validé à la perte du focus).
    pub(crate) name_edit: Option<(String, String)>,
    pub(crate) preview_states: std::collections::HashMap<String, Control>,
    /// Texte du menu en cours d'édition (une ligne par élément).
    pub(crate) menu_text: Option<String>,
}

impl DesignerState {
    /// Ouvre un fichier `.vrsform`.
    pub fn open(path: &Path) -> Result<Self, FormError> {
        let doc = FormDocument::load(path)?;
        Ok(Self::new(doc, path))
    }

    /// Crée l'état à partir d'un document déjà chargé.
    pub fn new(doc: FormDocument, path: &Path) -> Self {
        Self {
            saved: doc.clone(),
            doc,
            path: path.to_path_buf(),
            project_root: path
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_default(),
            selection: Vec::new(),
            pending_tool: None,
            preview: false,
            history: History::default(),
            drag: None,
            name_edit: None,
            preview_states: Default::default(),
            menu_text: None,
        }
    }

    /// Modifié depuis le dernier enregistrement ?
    pub fn is_dirty(&self) -> bool {
        self.doc != self.saved
    }

    /// Enregistre le `.vrsform`.
    pub fn save(&mut self) -> Result<(), FormError> {
        self.doc.save(&self.path)?;
        self.saved = self.doc.clone();
        self.history.break_coalescing();
        Ok(())
    }

    /// Exécute une modification annulable.
    pub fn edit(&mut self, f: impl FnOnce(&mut FormDocument)) {
        let before = self.doc.clone();
        f(&mut self.doc);
        if self.doc != before {
            self.history.record(before, None, 0.0);
            self.cleanup_selection();
        }
    }

    fn cleanup_selection(&mut self) {
        let doc = &self.doc;
        self.selection.retain(|n| doc.index_of(n).is_some());
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        let ok = self.history.undo(&mut self.doc);
        self.cleanup_selection();
        self.name_edit = None;
        ok
    }

    pub fn redo(&mut self) -> bool {
        let ok = self.history.redo(&mut self.doc);
        self.cleanup_selection();
        self.name_edit = None;
        ok
    }

    /// Ajoute un contrôle et le sélectionne.
    pub fn add_control(&mut self, kind: ComponentKind, x: f32, y: f32) -> String {
        let mut name = String::new();
        self.edit(|d| name = d.add_control(kind, x.max(0.0), y.max(0.0)));
        self.selection = vec![name.clone()];
        self.pending_tool = None;
        name
    }

    pub fn select_all(&mut self) {
        self.selection = self.doc.controls.iter().map(|c| c.name.clone()).collect();
    }

    pub fn has_selection(&self) -> bool {
        !self.selection.is_empty()
    }

    /// Supprime la sélection.
    pub fn delete_selection(&mut self) -> bool {
        if self.selection.is_empty() {
            return false;
        }
        let sel = std::mem::take(&mut self.selection);
        self.edit(|d| d.controls.retain(|c| !sel.contains(&c.name)));
        true
    }

    /// Copie la sélection.
    pub fn copy_selection(&self) -> Vec<Control> {
        self.doc
            .controls
            .iter()
            .filter(|c| self.selection.contains(&c.name))
            .cloned()
            .collect()
    }

    /// Colle des contrôles (renommés si nécessaire, décalés pour rester visibles).
    pub fn paste(&mut self, controls: &[Control], offset: f32) {
        if controls.is_empty() {
            return;
        }
        let mut names = Vec::new();
        self.edit(|d| {
            for c in controls {
                let mut c = c.clone();
                if d.check_name(&c.name, None).is_err() {
                    let (n, _) = d.next_name(c.kind);
                    c.name = n;
                }
                // Les gestionnaires appartiennent au contrôle d'origine.
                c.events.clear();
                c.x += offset;
                c.y += offset;
                names.push(c.name.clone());
                d.controls.push(c);
            }
        });
        self.selection = names;
    }

    /// Menu Format.
    pub fn apply_layout(&mut self, op: LayoutOp) {
        let sel = self.selection.clone();
        self.edit(|d| {
            ops::apply(d, &sel, op);
        });
    }

    /// Premier plan / arrière-plan.
    pub fn reorder(&mut self, to_front: bool) {
        let sel = self.selection.clone();
        self.edit(|d| {
            ops::reorder(d, &sel, to_front);
        });
    }

    /// Déplace la sélection (flèches du clavier).
    pub fn nudge(&mut self, dx: f32, dy: f32) {
        let sel = self.selection.clone();
        let before = self.doc.clone();
        for c in self
            .doc
            .controls
            .iter_mut()
            .filter(|c| sel.contains(&c.name))
        {
            c.x = (c.x + dx).max(0.0);
            c.y = (c.y + dy).max(0.0);
        }
        if self.doc != before {
            self.history.record(before, Some("nudge".into()), now());
        }
    }

    /// Associe (si nécessaire) un gestionnaire à l'événement et retourne son nom.
    /// C'est ce que fait un double-clic sur un contrôle.
    pub fn ensure_event(&mut self, req: &HandlerRequest) -> Option<String> {
        let module = self.doc.module_name();
        let is_menu = req
            .control
            .as_ref()
            .is_some_and(|n| self.doc.control(n).is_none());
        let existing = match &req.control {
            Some(name) if is_menu => self.doc.menu_item_mut(name)?.click.clone(),
            Some(name) => self.doc.control(name)?.events.get(&req.event).cloned(),
            None => self.doc.events.get(&req.event).cloned(),
        };
        if let Some(h) = existing {
            return Some(h);
        }
        let owner = req.control.clone().unwrap_or(module);
        let mut handler = req.event.default_handler(&owner);
        // Éviter une collision avec un gestionnaire existant.
        let taken: Vec<String> = self.doc.handlers().into_iter().map(|(h, ..)| h).collect();
        let base = handler.clone();
        let mut i = 2;
        while taken.contains(&handler) {
            handler = format!("{base}{i}");
            i += 1;
        }
        let h = handler.clone();
        let req = req.clone();
        self.edit(move |d| match &req.control {
            Some(name) if is_menu => {
                if let Some(m) = d.menu_item_mut(name) {
                    m.click = Some(h);
                }
            }
            Some(name) => {
                if let Some(c) = d.control_mut(name) {
                    c.events.insert(req.event, h);
                }
            }
            None => {
                d.events.insert(req.event, h);
            }
        });
        Some(handler)
    }
}

/// Horloge pour le regroupement des modifications.
pub(crate) fn now() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> DesignerState {
        DesignerState::new(FormDocument::new("Form1"), Path::new("/tmp/Form1.vrsform"))
    }

    #[test]
    fn add_delete_undo() {
        let mut s = state();
        let n = s.add_control(ComponentKind::Button, 8.0, 8.0);
        assert_eq!(s.selection, vec![n]);
        assert!(s.is_dirty());
        assert!(s.delete_selection());
        assert!(s.doc.controls.is_empty());
        assert!(s.undo());
        assert_eq!(s.doc.controls.len(), 1);
        assert!(s.undo());
        assert!(s.doc.controls.is_empty());
        assert!(!s.is_dirty());
        assert!(s.redo());
        assert_eq!(s.doc.controls.len(), 1);
    }

    #[test]
    fn copy_paste_renames_and_drops_events() {
        let mut s = state();
        s.add_control(ComponentKind::Button, 8.0, 8.0);
        s.ensure_event(&HandlerRequest {
            control: Some("button1".into()),
            event: EventKind::Click,
        });
        let copied = s.copy_selection();
        s.paste(&copied, 16.0);
        assert_eq!(s.selection, vec!["button2".to_string()]);
        let b2 = s.doc.control("button2").unwrap();
        assert_eq!((b2.x, b2.y), (24.0, 24.0));
        assert!(b2.events.is_empty());
    }

    #[test]
    fn ensure_event_is_idempotent() {
        let mut s = state();
        s.add_control(ComponentKind::Button, 0.0, 0.0);
        let req = HandlerRequest {
            control: Some("button1".into()),
            event: EventKind::Click,
        };
        assert_eq!(s.ensure_event(&req).as_deref(), Some("button1_click"));
        assert_eq!(s.ensure_event(&req).as_deref(), Some("button1_click"));
        let load = HandlerRequest {
            control: None,
            event: EventKind::Load,
        };
        assert_eq!(s.ensure_event(&load).as_deref(), Some("form1_load"));
        assert!(s.doc.validate().is_empty());
    }

    #[test]
    fn save_clears_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Form1.vrsform");
        let mut s = DesignerState::new(FormDocument::new("Form1"), &path);
        s.add_control(ComponentKind::Label, 0.0, 0.0);
        s.save().unwrap();
        assert!(!s.is_dirty());
        let reopened = DesignerState::open(&path).unwrap();
        assert_eq!(reopened.doc, s.doc);
    }
}
