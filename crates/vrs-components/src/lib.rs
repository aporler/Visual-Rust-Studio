//! Composants et modèle déclaratif des fenêtres de Visual Rust Studio.
//!
//! Une fenêtre conçue dans VRS est décrite par un [`FormDocument`], enregistré
//! dans `forms/<Nom>.vrsform` (format RON). Ce fichier contient **uniquement des
//! métadonnées de présentation** : positions, tailles, textes, noms des
//! gestionnaires. Ce n'est pas un langage de programmation : la logique reste
//! entièrement écrite en Rust par l'utilisateur.
//!
//! Le concepteur n'exécute jamais de code utilisateur pour afficher une fenêtre :
//! il dessine ce modèle déclaratif.

mod component;
mod form;
pub mod ops;

pub use component::{ComponentKind, EventKind};
pub use form::{Control, FORM_EXTENSION, FORM_FORMAT_VERSION, FormDocument, FormError, NameError};
