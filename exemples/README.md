# Examples / Exemples

*[English](#english) · [Français](#français)*

---

## English

Example projects created with Visual Rust Studio, in English (`en-US/`) and in
French (`fr-CA/`). Each folder is a standalone Cargo project: it builds and
runs without VRS.

```sh
cd exemples/en-US/Stopwatch
cargo run
```

You can also open them in VRS (File → Open Project) to see the windows in the
designer, or create a fresh copy with **File → Examples…**.

| Level | Project | What you learn |
|---|---|---|
| Beginner | `HelloWorldTerminal` | `println!`, your first program |
| Beginner | `HelloWorldWindow` | a window, a label, a button that closes the application |
| Beginner | `ClickCounter` | window variables (`self.data`), Click events |
| Beginner | `TemperatureConverter` | reading a text box, `parse()`, `Ok` / `Err` |
| Beginner | `Calculator` | ComboBox, `match`, `Option`, message box |
| Beginner | `ColorPicker` | Sliders, ValueChanged, BackColor |
| Beginner | `RustQuiz` | RadioButtons, constant arrays, score |
| Intermediate | `TodoList` | ListBox, `Vec`, helper method |
| Intermediate | `GuessTheNumber` | Load event, `Ordering`, message box |
| Intermediate | `TwoWindows` | modal dialog, data returned by another window |
| Intermediate | `Stopwatch` | Timer and Tick event, ProgressBar, formatting |
| Intermediate | `AddressBook` | two windows, list sorting |
| Advanced | `TicTacToe` | game state, winner detection, `&mut` to a field |
| Advanced | `Notepad` | multiline TextBox, menu bar, files with `std::fs` |

These folders are generated: after changing the examples in
`crates/vrs-project/src/examples.rs`, run
`cargo run -p vrs-app --bin vrs-export-examples -- exemples`.

---

## Français

Projets d'exemple créés avec Visual Rust Studio, en anglais (`en-US/`) et en
français (`fr-CA/`). Chaque dossier est un projet Cargo autonome : il compile
et s'exécute sans VRS.

```sh
cd exemples/fr-CA/Stopwatch
cargo run
```

Vous pouvez aussi les ouvrir dans VRS (Fichier → Ouvrir un projet) pour voir
les fenêtres dans le concepteur, ou en créer une copie avec
**Fichier → Exemples…**.

| Niveau | Projet | Ce que vous apprenez |
|---|---|---|
| Débutant | `HelloWorldTerminal` | `println!`, votre premier programme |
| Débutant | `HelloWorldWindow` | une fenêtre, une étiquette, un bouton qui ferme l'application |
| Débutant | `ClickCounter` | variables de fenêtre (`self.data`), événements Click |
| Débutant | `TemperatureConverter` | lire une zone de texte, `parse()`, `Ok` / `Err` |
| Débutant | `Calculator` | ComboBox, `match`, `Option`, boîte de message |
| Débutant | `ColorPicker` | curseurs, ValueChanged, BackColor |
| Débutant | `RustQuiz` | RadioButton, tableaux constants, score |
| Modéré | `TodoList` | ListBox, `Vec`, méthode utilitaire |
| Modéré | `GuessTheNumber` | événement Load, `Ordering`, boîte de message |
| Modéré | `TwoWindows` | boîte de dialogue modale, données renvoyées par une autre fenêtre |
| Modéré | `Stopwatch` | Timer et événement Tick, ProgressBar, mise en forme |
| Modéré | `AddressBook` | deux fenêtres, tri d'une liste |
| Avancé | `TicTacToe` | état du jeu, détection du gagnant, `&mut` vers un champ |
| Avancé | `Notepad` | TextBox multiligne, barre de menus, fichiers avec `std::fs` |

Ces dossiers sont générés : après une modification des exemples dans
`crates/vrs-project/src/examples.rs`, lancez
`cargo run -p vrs-app --bin vrs-export-examples -- exemples`.
