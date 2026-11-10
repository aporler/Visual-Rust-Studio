//! Commandes de l'IDE.
//!
//! L'interface ne modifie pas directement l'état : elle émet des commandes,
//! exécutées après le dessin de l'image (pas de conflits d'emprunt, et chaque
//! action est disponible depuis les menus, la barre d'outils et le clavier).

use std::path::PathBuf;

use vrs_compiler::CargoCommand;
use vrs_components::ops::LayoutOp;
use vrs_designer::HandlerRequest;

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    NewProject,
    OpenProject,
    OpenPath(PathBuf),
    Save,
    SaveAll,
    CloseProject,
    Exit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    Find,
    FormatDocument,
    ToggleExplorer,
    ToggleProperties,
    ToggleOutput,
    ViewCode,
    ViewDesigner,
    AddForm,
    GenerateCode,
    ReloadProject,
    Cargo(CargoCommand),
    Stop,
    ToggleRelease,
    Settings,
    CloseTab(usize),
    CloseAllTabs,
    About,
    OpenUrl(String),
    Layout(LayoutOp),
    Reorder { to_front: bool },
    OpenForm(String),
    OpenFile { path: PathBuf, line: Option<usize> },
    OpenHandler(HandlerRequest),
    ShowStart,
}
