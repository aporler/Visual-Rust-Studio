//! Intégration de Cargo dans Visual Rust Studio.
//!
//! Les commandes (`build`, `run`, `check`, `clippy`, `test`, `clean`) sont lancées
//! dans un processus séparé. La sortie est transmise ligne par ligne à l'IDE via
//! un canal ; les messages JSON de Cargo (`--message-format=json`) sont convertis
//! en [`Diagnostic`] (fichier, ligne, colonne, code d'erreur, message original).

mod diagnostics;
mod process;

pub use diagnostics::{CargoLine, Diagnostic, Level, Span, Suggestion, parse_cargo_line};
pub use process::{
    BuildEvent, BuildHandle, CargoCommand, OutputStream, find_cargo, find_tool, spawn,
};
