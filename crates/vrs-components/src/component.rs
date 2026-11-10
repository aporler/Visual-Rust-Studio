//! Types de composants et événements.

use serde::{Deserialize, Serialize};

/// Type de composant disponible dans la boîte à outils.
///
/// Les premiers composants sont Button, Label et TextBox ; les autres
/// (CheckBox, ComboBox, Panel…) viendront dans les phases suivantes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComponentKind {
    Button,
    Label,
    TextBox,
}

impl ComponentKind {
    /// Tous les composants, dans l'ordre de la boîte à outils.
    pub const ALL: [ComponentKind; 3] = [Self::Button, Self::Label, Self::TextBox];

    /// Nom du type (identique au nom anglais de Windows Forms).
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Label => "Label",
            Self::TextBox => "TextBox",
        }
    }

    /// Préfixe des noms générés automatiquement (`button1`, `label1`, `text_box1`).
    pub fn name_prefix(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Label => "label",
            Self::TextBox => "text_box",
        }
    }

    /// Type d'état Rust généré pour ce composant (défini dans le runtime `vrs`).
    pub fn state_type(self) -> &'static str {
        match self {
            Self::Button => "ButtonState",
            Self::Label => "LabelState",
            Self::TextBox => "TextBoxState",
        }
    }

    /// Fonction de rendu du runtime `vrs`.
    pub fn render_fn(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Label => "label",
            Self::TextBox => "text_box",
        }
    }

    /// Clé de traduction du nom affiché.
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::Button => "component-button",
            Self::Label => "component-label",
            Self::TextBox => "component-textbox",
        }
    }

    /// Clé de traduction de la description (infobulle de la boîte à outils).
    pub fn i18n_desc_key(self) -> &'static str {
        match self {
            Self::Button => "component-button-desc",
            Self::Label => "component-label-desc",
            Self::TextBox => "component-textbox-desc",
        }
    }

    /// Taille par défaut (largeur, hauteur) en points.
    pub fn default_size(self) -> (f32, f32) {
        match self {
            Self::Button => (96.0, 28.0),
            Self::Label => (96.0, 20.0),
            Self::TextBox => (160.0, 24.0),
        }
    }

    /// Texte initial (`Button1`, `Label1`… comme dans Windows Forms ; vide pour TextBox).
    pub fn default_text(self, index: usize) -> String {
        match self {
            Self::Button => format!("Button{index}"),
            Self::Label => format!("Label{index}"),
            Self::TextBox => String::new(),
        }
    }

    /// Événements pris en charge.
    pub fn events(self) -> &'static [EventKind] {
        match self {
            Self::Button => &[EventKind::Click],
            Self::Label => &[EventKind::Click],
            Self::TextBox => &[EventKind::TextChanged],
        }
    }

    /// Événement ouvert par un double-clic dans le concepteur.
    pub fn default_event(self) -> EventKind {
        self.events()[0]
    }
}

/// Événement auquel un gestionnaire Rust peut être associé.
///
/// Les autres événements du cahier des charges (DoubleClick, SelectionChanged,
/// Closing, Resize, KeyDown…) seront ajoutés avec les composants qui les utilisent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventKind {
    Click,
    TextChanged,
    /// Chargement de la fenêtre (premier affichage).
    Load,
}

impl EventKind {
    /// Nom de l'événement (style Windows Forms).
    pub fn name(self) -> &'static str {
        match self {
            Self::Click => "Click",
            Self::TextChanged => "TextChanged",
            Self::Load => "Load",
        }
    }

    /// Suffixe du nom du gestionnaire (`button1_click`).
    pub fn handler_suffix(self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::TextChanged => "text_changed",
            Self::Load => "load",
        }
    }

    /// Clé de traduction.
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::Click => "event-click",
            Self::TextChanged => "event-text-changed",
            Self::Load => "event-load",
        }
    }

    /// Nom de gestionnaire proposé pour un contrôle (`button1` → `button1_click`).
    pub fn default_handler(self, owner: &str) -> String {
        format!("{owner}_{}", self.handler_suffix())
    }
}
