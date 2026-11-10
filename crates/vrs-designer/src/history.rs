//! Annuler / rétablir par instantanés du document.

use vrs_components::FormDocument;

const LIMIT: usize = 200;
/// Délai (secondes) pendant lequel des modifications successives du même champ
/// sont regroupées en une seule étape d'annulation.
const COALESCE_SECONDS: f64 = 1.5;

/// Pile d'annulation.
#[derive(Default)]
pub struct History {
    undo: Vec<FormDocument>,
    redo: Vec<FormDocument>,
    last_key: Option<String>,
    last_time: f64,
}

impl History {
    /// Mémorise l'état `before` (avant une modification).
    ///
    /// Si `key` est identique à la clé précédente et que la modification
    /// précédente est récente, les deux sont regroupées (saisie au clavier).
    pub fn record(&mut self, before: FormDocument, key: Option<String>, now: f64) {
        if key.is_some() && key == self.last_key && now - self.last_time < COALESCE_SECONDS {
            self.last_time = now;
            return;
        }
        self.undo.push(before);
        if self.undo.len() > LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.last_key = key;
        self.last_time = now;
    }

    /// Termine le regroupement en cours.
    pub fn break_coalescing(&mut self) {
        self.last_key = None;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Annule : `current` reprend l'état précédent.
    pub fn undo(&mut self, current: &mut FormDocument) -> bool {
        let Some(prev) = self.undo.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(current, prev));
        self.last_key = None;
        true
    }

    /// Rétablit.
    pub fn redo(&mut self, current: &mut FormDocument) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(std::mem::replace(current, next));
        self.last_key = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vrs_components::ComponentKind;

    #[test]
    fn undo_redo() {
        let mut h = History::default();
        let mut doc = FormDocument::new("Form1");
        let before = doc.clone();
        doc.add_control(ComponentKind::Button, 0.0, 0.0);
        h.record(before, None, 0.0);
        assert!(h.undo(&mut doc));
        assert!(doc.controls.is_empty());
        assert!(h.redo(&mut doc));
        assert_eq!(doc.controls.len(), 1);
        assert!(!h.redo(&mut doc));
    }

    #[test]
    fn coalescing() {
        let mut h = History::default();
        let mut doc = FormDocument::new("Form1");
        for (i, t) in ["a", "ab", "abc"].iter().enumerate() {
            let before = doc.clone();
            doc.title = (*t).into();
            h.record(before, Some("title".into()), i as f64 * 0.2);
        }
        assert!(h.undo(&mut doc));
        assert_eq!(doc.title, "Form1");
        assert!(!h.can_undo());
    }
}
