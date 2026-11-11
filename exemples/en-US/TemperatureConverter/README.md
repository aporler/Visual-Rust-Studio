# TemperatureConverter

*[English](#english) · [Français](#français)*

## English

Project created with [Visual Rust Studio](https://github.com/aporler/Visual-Rust-Studio).

It is a standard Cargo project: it builds and runs **without** Visual Rust Studio.

```sh
cargo run          # build and run
cargo build --release
```

The code of this project belongs to you: license it as you wish.

### Layout

| File | Role |
|---|---|
| `src/main.rs` | entry point (yours) |
| `src/forms/form1.rs` | **your code**: Form1 event handlers |
| `src/forms/form1_designer.rs` | generated from the designer (do not edit) |
| `src/forms/mod.rs` | generated: window list |
| `src/vrs/mod.rs` | small runtime on top of egui/eframe (do not edit) |
| `forms/*.vrsform` | window designs (metadata, no code) |
| `*.vrsproj` | project settings for Visual Rust Studio |

## Français

Projet créé avec [Visual Rust Studio](https://github.com/aporler/Visual-Rust-Studio).

C'est un projet Cargo standard : il compile et s'exécute **sans** Visual Rust Studio.

```sh
cargo run          # compiler et exécuter
cargo build --release
```

Le code de ce projet vous appartient : choisissez librement sa licence.

### Organisation

| Fichier | Rôle |
|---|---|
| `src/main.rs` | point d'entrée (le vôtre) |
| `src/forms/form1.rs` | **votre code** : gestionnaires d'événements de Form1 |
| `src/forms/form1_designer.rs` | généré à partir du concepteur (ne pas modifier) |
| `src/forms/mod.rs` | généré : liste des fenêtres |
| `src/vrs/mod.rs` | petit runtime au-dessus d'egui/eframe (ne pas modifier) |
| `forms/*.vrsform` | conception des fenêtres (métadonnées, pas de code) |
| `*.vrsproj` | paramètres du projet pour Visual Rust Studio |
