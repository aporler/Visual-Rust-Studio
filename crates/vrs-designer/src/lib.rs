//! Concepteur visuel de Visual Rust Studio.
//!
//! Le concepteur édite un [`FormDocument`](vrs_components::FormDocument)
//! (modèle déclaratif) et le dessine avec **les mêmes fonctions** que le runtime
//! des applications générées (`vrs-runtime`) : l'aperçu est fidèle au résultat,
//! sans jamais exécuter de code de l'utilisateur.

mod canvas;
mod history;
mod properties;
mod state;
mod toolbox;

pub use canvas::canvas_ui;
pub use history::History;
pub use properties::properties_ui;
pub use state::{DesignerOutput, DesignerState, HandlerRequest};
pub use toolbox::toolbox_ui;
