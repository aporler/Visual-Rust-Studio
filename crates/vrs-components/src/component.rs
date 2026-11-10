//! Types de composants et événements.

use serde::{Deserialize, Serialize};

/// Type de composant disponible dans la boîte à outils.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComponentKind {
    Button,
    Label,
    TextBox,
    CheckBox,
    RadioButton,
    ComboBox,
    ListBox,
    ProgressBar,
    Slider,
    GroupBox,
    Panel,
    PictureBox,
}

/// Propriétés propres à certains composants (en plus de Name, Text, position…).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feature {
    /// `text` est affiché.
    Text,
    /// Case cochée (`checked`).
    Checked,
    /// Groupe d'options exclusives (`group`).
    Group,
    /// Liste d'éléments (`items`, `selected`).
    Items,
    /// Valeur numérique (`value`, `minimum`, `maximum`).
    Value,
    /// Image du projet (`image`).
    Image,
}

impl ComponentKind {
    /// Tous les composants, dans l'ordre de la boîte à outils.
    pub const ALL: [ComponentKind; 12] = [
        Self::Button,
        Self::Label,
        Self::TextBox,
        Self::CheckBox,
        Self::RadioButton,
        Self::ComboBox,
        Self::ListBox,
        Self::ProgressBar,
        Self::Slider,
        Self::GroupBox,
        Self::Panel,
        Self::PictureBox,
    ];

    /// Nom du type (identique au nom anglais de Windows Forms).
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Label => "Label",
            Self::TextBox => "TextBox",
            Self::CheckBox => "CheckBox",
            Self::RadioButton => "RadioButton",
            Self::ComboBox => "ComboBox",
            Self::ListBox => "ListBox",
            Self::ProgressBar => "ProgressBar",
            Self::Slider => "Slider",
            Self::GroupBox => "GroupBox",
            Self::Panel => "Panel",
            Self::PictureBox => "PictureBox",
        }
    }

    /// Préfixe des noms générés automatiquement (`button1`, `text_box1`…).
    pub fn name_prefix(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Label => "label",
            Self::TextBox => "text_box",
            Self::CheckBox => "check_box",
            Self::RadioButton => "radio_button",
            Self::ComboBox => "combo_box",
            Self::ListBox => "list_box",
            Self::ProgressBar => "progress_bar",
            Self::Slider => "slider",
            Self::GroupBox => "group_box",
            Self::Panel => "panel",
            Self::PictureBox => "picture_box",
        }
    }

    /// Type d'état Rust généré pour ce composant (défini dans le runtime `vrs`).
    pub fn state_type(self) -> &'static str {
        match self {
            Self::Button => "ButtonState",
            Self::Label => "LabelState",
            Self::TextBox => "TextBoxState",
            Self::CheckBox => "CheckBoxState",
            Self::RadioButton => "RadioButtonState",
            Self::ComboBox => "ComboBoxState",
            Self::ListBox => "ListBoxState",
            Self::ProgressBar => "ProgressBarState",
            Self::Slider => "SliderState",
            Self::GroupBox => "GroupBoxState",
            Self::Panel => "PanelState",
            Self::PictureBox => "PictureBoxState",
        }
    }

    /// Fonction de rendu du runtime `vrs`.
    pub fn render_fn(self) -> &'static str {
        self.name_prefix()
    }

    /// La fonction de rendu modifie-t-elle l'état (`&mut`) ?
    pub fn render_mut(self) -> bool {
        matches!(
            self,
            Self::TextBox
                | Self::CheckBox
                | Self::RadioButton
                | Self::ComboBox
                | Self::ListBox
                | Self::Slider
        )
    }

    /// Clé de traduction du nom affiché.
    pub fn i18n_key(self) -> String {
        format!("component-{}", self.name_prefix().replace('_', ""))
    }

    /// Clé de traduction de la description (infobulle de la boîte à outils).
    pub fn i18n_desc_key(self) -> String {
        format!("{}-desc", self.i18n_key())
    }

    /// Taille par défaut (largeur, hauteur) en points.
    pub fn default_size(self) -> (f32, f32) {
        match self {
            Self::Button => (96.0, 28.0),
            Self::Label => (96.0, 20.0),
            Self::TextBox => (160.0, 24.0),
            Self::CheckBox | Self::RadioButton => (128.0, 22.0),
            Self::ComboBox => (160.0, 24.0),
            Self::ListBox => (160.0, 96.0),
            Self::ProgressBar => (160.0, 20.0),
            Self::Slider => (160.0, 22.0),
            Self::GroupBox => (200.0, 120.0),
            Self::Panel => (200.0, 120.0),
            Self::PictureBox => (120.0, 90.0),
        }
    }

    /// Texte initial (`Button1`, `CheckBox1`… comme dans Windows Forms).
    pub fn default_text(self, index: usize) -> String {
        match self {
            Self::TextBox
            | Self::ComboBox
            | Self::ListBox
            | Self::ProgressBar
            | Self::Slider
            | Self::Panel => String::new(),
            _ => format!("{}{index}", self.type_name()),
        }
    }

    /// Propriétés propres au composant.
    pub fn features(self) -> &'static [Feature] {
        match self {
            Self::Button | Self::Label | Self::TextBox | Self::GroupBox => &[Feature::Text],
            Self::CheckBox => &[Feature::Text, Feature::Checked],
            Self::RadioButton => &[Feature::Text, Feature::Checked, Feature::Group],
            Self::ComboBox | Self::ListBox => &[Feature::Items],
            Self::ProgressBar | Self::Slider => &[Feature::Value],
            Self::Panel => &[],
            Self::PictureBox => &[Feature::Image],
        }
    }

    pub fn has(self, f: Feature) -> bool {
        self.features().contains(&f)
    }

    /// Événements pris en charge.
    pub fn events(self) -> &'static [EventKind] {
        match self {
            Self::Button | Self::Label | Self::PictureBox => &[EventKind::Click],
            Self::TextBox => &[EventKind::TextChanged],
            Self::CheckBox | Self::RadioButton => &[EventKind::CheckedChanged],
            Self::ComboBox | Self::ListBox => &[EventKind::SelectionChanged],
            Self::Slider => &[EventKind::ValueChanged],
            Self::ProgressBar | Self::GroupBox | Self::Panel => &[],
        }
    }

    /// Événement ouvert par un double-clic dans le concepteur.
    pub fn default_event(self) -> Option<EventKind> {
        self.events().first().copied()
    }

    /// Conteneur visuel (dessiné derrière les autres contrôles).
    pub fn is_container(self) -> bool {
        matches!(self, Self::GroupBox | Self::Panel)
    }
}

/// Événement auquel un gestionnaire Rust peut être associé.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventKind {
    Click,
    TextChanged,
    /// Chargement de la fenêtre (premier affichage).
    Load,
    /// Demande de fermeture de la fenêtre (le gestionnaire peut l'annuler).
    Closing,
    /// Fenêtre fermée.
    Closed,
    /// Case cochée ou décochée.
    CheckedChanged,
    /// Élément sélectionné changé (liste, liste déroulante).
    SelectionChanged,
    /// Valeur changée (curseur).
    ValueChanged,
}

impl EventKind {
    /// Événements de la fenêtre elle-même.
    pub const FORM_EVENTS: [EventKind; 3] = [Self::Load, Self::Closing, Self::Closed];

    /// Nom de l'événement (style Windows Forms).
    pub fn name(self) -> &'static str {
        match self {
            Self::Click => "Click",
            Self::TextChanged => "TextChanged",
            Self::Load => "Load",
            Self::Closing => "Closing",
            Self::Closed => "Closed",
            Self::CheckedChanged => "CheckedChanged",
            Self::SelectionChanged => "SelectionChanged",
            Self::ValueChanged => "ValueChanged",
        }
    }

    /// Suffixe du nom du gestionnaire (`button1_click`).
    pub fn handler_suffix(self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::TextChanged => "text_changed",
            Self::Load => "load",
            Self::Closing => "closing",
            Self::Closed => "closed",
            Self::CheckedChanged => "checked_changed",
            Self::SelectionChanged => "selection_changed",
            Self::ValueChanged => "value_changed",
        }
    }

    /// Clé de traduction.
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::Click => "event-click",
            Self::TextChanged => "event-text-changed",
            Self::Load => "event-load",
            Self::Closing => "event-closing",
            Self::Closed => "event-closed",
            Self::CheckedChanged => "event-checked-changed",
            Self::SelectionChanged => "event-selection-changed",
            Self::ValueChanged => "event-value-changed",
        }
    }

    /// Type de retour du gestionnaire (`Closing` retourne `bool` : `false` annule).
    pub fn handler_return(self) -> Option<&'static str> {
        match self {
            Self::Closing => Some("bool"),
            _ => None,
        }
    }

    /// Nom de gestionnaire proposé pour un contrôle (`button1` → `button1_click`).
    pub fn default_handler(self, owner: &str) -> String {
        format!("{owner}_{}", self.handler_suffix())
    }
}
