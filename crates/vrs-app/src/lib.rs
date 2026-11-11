//! Application principale de Visual Rust Studio.
//!
//! L'IDE est construit avec egui/eframe. Tous les textes affichés passent par
//! `vrs-i18n` ; aucune chaîne d'interface n'est écrite en dur dans ce crate.

mod actions;
mod app;
mod commands;
mod debug;
mod export;
mod lang;
mod lsp;
mod tabs;
mod ui;

pub use app::{BottomTab, VrsApp};
pub use commands::Command;
pub use debug::{DebugAction, Watch};
pub use export::{EXPORT_LANGUAGES, export_examples};
pub use tabs::Tab;
