# Visual Rust Studio architecture / Architecture de Visual Rust Studio

*[English](#english) · [Français](#français)*

---

## English

### 1. Feasibility analysis and technology choices

#### Key constraints

1. **Rust stays Rust**: application code must be standard Rust, buildable by
   Cargo without VRS. No mandatory UI DSL.
2. **The designer must never run user code** to display a window.
3. Cross-platform: Windows, Linux, macOS (x86_64 and ARM64).
4. Multiple windows, modal windows.

#### Frameworks evaluated (state checked in November 2026)

| Criterion | egui / eframe 0.37 | Slint | iced |
|---|---|---|---|
| UI 100 % in Rust | **yes** | no: `.slint` DSL (or `slint!` macro) | yes |
| Generated code readable by a beginner | yes (immediate mode) | DSL to learn | Elm architecture (messages, `update`), more abstract |
| Native multiple windows | yes: viewports (`show_viewport_immediate` / `deferred`) | yes | yes (`multi-window`) |
| Modal | `egui::Modal` inside a window; no native OS modality | yes | partial |
| Identical rendering designer ↔ application | **yes** (same widgets in the IDE and the app) | possible, through the Slint interpreter | no (IDE in another toolkit) |
| Maturity / maintenance | very active, regular releases | active, dual license (GPL/royalty-free/commercial) | active |

**Choice: egui/eframe 0.37**, for both the IDE and the generated applications.

- The generated code is ordinary Rust. A button is a `ButtonState` field and a
  function call, with no DSL and no macro.
- The designer draws controls with **the same functions** as the final
  application (`vrs-runtime`). The preview is therefore faithful, without
  building or running any user code.
- Slint was ruled out because its `.slint` language contradicts the "Rust
  stays Rust" rule. iced was ruled out because its Elm model is harder for a
  beginner coming from VB.NET, and because the IDE would then have to draw
  components with another toolkit.

#### Known egui limitations

- **No true OS-level modality.** A "modal" window will be simulated by the
  runtime: the application's other windows are disabled, and `egui::Modal` is
  used inside a window.
- **Wayland**: applications cannot position their windows.
- **egui_dock 0.21 targets egui 0.36.** It is not compatible with egui 0.37:
  the IDE panels therefore use `egui::Panel` (resizable) rather than a full
  docking system.
- **Accessibility**: egui exposes AccessKit. Generated text boxes do not have
  an associated accessible name yet.
- **Fonts**: only built-in fonts are loaded. Missing glyphs (some Unicode
  symbols) are caught by the UI tests.

#### Other technologies

| Need | Choice | Reason |
|---|---|---|
| Translations | Fluent (`fluent-bundle` 0.16) | plurals, parameters, variants, safe data format (no code) |
| System language | `sys-locale` | Windows, macOS, Linux |
| Settings | TOML + `directories` | native configuration folders |
| Designed windows | RON (`.vrsform`) | readable, serde-compatible, metadata only |
| Code generation | `syn` + `quote` + `prettyplease` | valid by construction, formatted |
| Insertion into user code | `syn` (positions via `proc-macro2/span-locations`) | targeted insertion, refused if the file does not parse |
| Highlighting | `egui_extras` (syntect) | Rust highlighting |
| Building | Cargo `--message-format=json` | structured diagnostics (code, file, line, suggestions) |
| File dialogs | `rfd` | native, XDG portal on Linux |
| UI tests | `egui_kittest` | UI tested without a window |

### 2. Workspace layout

```
crates/
  vrs-core        atomic writes, .bak backups, Rust identifiers
  vrs-i18n        Fluent engine, fallback chain, validation (vrs-i18n-check)
  vrs-settings    TOML settings (language, theme, editor, designer, toolchain, trust)
  vrs-components  ComponentKind, EventKind, FormDocument (.vrsform), Format operations
  vrs-runtime     runtime of generated applications (copied to src/vrs/mod.rs)
  vrs-codegen     window code generation, handler insertion
  vrs-project     .vrsproj, project templates, file tree
  vrs-compiler    cargo build/run/check/clippy/test/clean, JSON diagnostics
  vrs-editor      code editor (highlighting, search, diagnostics, rustfmt)
  vrs-designer    visual designer (surface, properties, toolbox, undo)
  vrs-app         `vrs` binary: main window, menus, panels, dialogs
locales/<BCP 47>/*.ftl
```

Dependencies (no cycles): `vrs-app` → `vrs-designer`, `vrs-editor`,
`vrs-project`, `vrs-compiler`, `vrs-settings`, `vrs-i18n`. `vrs-project` →
`vrs-codegen` → `vrs-components` → `vrs-core`. `vrs-designer` → `vrs-runtime`.

`vrs-debugger` does not exist yet: it will be created with the DAP
integration.

### 3. Generated project (graphical application)

```
MyApp/
  Cargo.toml                    eframe = "0.37" (only dependency)
  MyApp.vrsproj                 VRS settings (not needed to build)
  forms/Form1.vrsform           design (RON metadata)
  src/main.rs                   created once — belongs to the user
  src/vrs/mod.rs                runtime (regenerated)
  src/forms/mod.rs              window list (regenerated)
  src/forms/form1_designer.rs   fields + drawing + event dispatch (regenerated)
  src/forms/form1.rs            USER CODE — never rewritten
```

#### Generated code / user code separation

- Regenerated files carry an `<auto-generated>` header. The editor shows a
  warning when they are opened.
- `form1.rs` is **never rewritten**. When an event is added, VRS parses the
  file with `syn`:
  - if the method already exists, nothing is done;
  - otherwise, the method is inserted before the closing brace of the last
    `impl Form1`, after a `.bak` copy and through an atomic write;
  - if the file has a syntax error, **nothing is changed** and the error is
    reported.

#### Event model (borrowing)

```rust
fn show(&mut self, ui: &mut egui::Ui, app: &mut AppContext) {
    let button1 = vrs::button(ui, "button1", &self.button1);   // 1. drawing
    let _ = vrs::text_box(ui, "text_box1", &mut self.text_box1);
    if button1.clicked() {                                      // 2. dispatch
        self.button1_click(app);                                //    &mut self is free
    }
}
```

egui responses are owned values: once drawing is done, no borrow is active.
The handler therefore receives `&mut self` and can change any control
(`self.text_box1.text = …`).

### 4. Security

- The designer draws the declarative model and runs no code.
- `.ftl` files are data, never code.
- A project not created by VRS asks for confirmation before any Cargo command
  (`build.rs` scripts and procedural macros are executable code). The choice
  is remembered in `trusted_projects`.
- All writes are atomic. A `.bak` copy is made before any change to a user
  file.
- No telemetry.

---

## Français

### 1. Analyse de faisabilité et choix technologiques

#### Contraintes déterminantes

1. **Rust reste Rust** : le code des applications doit être du Rust standard,
   compilable par Cargo sans VRS. Aucun DSL d'interface obligatoire.
2. **Le concepteur ne doit jamais exécuter le code de l'utilisateur** pour
   afficher une fenêtre.
3. Multiplateforme : Windows, Linux, macOS (x86_64 et ARM64).
4. Multifenêtre, fenêtres modales.

#### Frameworks évalués (état vérifié en novembre 2026)

| Critère | egui / eframe 0.37 | Slint | iced |
|---|---|---|---|
| Interface 100 % en Rust | **oui** | non : DSL `.slint` (ou macro `slint!`) | oui |
| Code généré lisible par un débutant | oui (mode immédiat) | DSL à apprendre | architecture Elm (messages, `update`) plus abstraite |
| Multifenêtre natif | oui : viewports (`show_viewport_immediate` / `deferred`) | oui | oui (`multi-window`) |
| Modal | `egui::Modal` dans la fenêtre ; pas de modalité OS native | oui | partiel |
| Rendu identique concepteur ↔ application | **oui** (mêmes widgets dans l'IDE et l'appli) | possible mais via l'interpréteur Slint | non (IDE dans un autre toolkit) |
| Maturité / maintenance | très active, publications régulières | active, licence double (GPL/royalty-free/commerciale) | active |

**Choix : egui/eframe 0.37**, à la fois pour l'IDE et pour les applications
générées.

- Le code généré est du Rust ordinaire. Un bouton, c'est un champ
  `ButtonState` et un appel de fonction, sans DSL ni macro.
- Le concepteur dessine les contrôles avec **les mêmes fonctions** que
  l'application finale (`vrs-runtime`). L'aperçu est donc fidèle, sans compiler
  ni exécuter de code utilisateur.
- Slint a été écarté parce que son langage `.slint` contredit la règle
  « Rust reste Rust ». iced a été écarté parce que son modèle Elm est plus
  difficile pour un débutant venant de VB.NET, et parce que l'IDE devrait alors
  dessiner les composants avec un autre toolkit.

#### Limites connues d'egui, documentées honnêtement

- **Pas de vraie modalité au niveau du système d'exploitation.** Une fenêtre
  « modale » sera simulée par le runtime : les autres fenêtres de l'application
  sont désactivées, et `egui::Modal` est utilisé à l'intérieur d'une fenêtre.
- **Wayland** : les applications ne peuvent pas positionner leurs fenêtres.
- **egui_dock 0.21 cible egui 0.36.** Il est incompatible avec egui 0.37 :
  les panneaux de l'IDE utilisent donc `egui::Panel` (redimensionnables) plutôt
  qu'un système d'ancrage complet.
- **Accessibilité** : egui expose AccessKit. Les zones de texte générées n'ont
  pas encore de nom accessible associé.
- **Polices** : seules les polices intégrées sont chargées. Les glyphes absents
  (certains symboles Unicode) sont détectés par les tests sans affichage.

#### Autres technologies

| Besoin | Choix | Raison |
|---|---|---|
| Traductions | Fluent (`fluent-bundle` 0.16) | pluriels, paramètres, variantes, format de données sûr (pas de code) |
| Langue du système | `sys-locale` | Windows, macOS, Linux |
| Paramètres | TOML + `directories` | répertoires de configuration natifs |
| Fenêtres conçues | RON (`.vrsform`) | lisible, compatible avec serde, uniquement des métadonnées |
| Génération de code | `syn` + `quote` + `prettyplease` | code valide par construction, mis en forme |
| Insertion dans le code utilisateur | `syn` (positions via `proc-macro2/span-locations`) | insertion ciblée, refus si le fichier ne s'analyse pas |
| Coloration | `egui_extras` (syntect) | coloration Rust |
| Compilation | Cargo `--message-format=json` | diagnostics structurés (code, fichier, ligne, suggestions) |
| Dialogues de fichiers | `rfd` | natifs, portail XDG sous Linux |
| Tests d'interface | `egui_kittest` | interface testée sans écran |

### 2. Organisation du workspace

```
crates/
  vrs-core        écriture atomique, sauvegardes .bak, identifiants Rust
  vrs-i18n        moteur Fluent, chaîne de repli, validation (vrs-i18n-check)
  vrs-settings    paramètres TOML (langue, thème, éditeur, concepteur, toolchain, confiance)
  vrs-components  ComponentKind, EventKind, FormDocument (.vrsform), opérations Format
  vrs-runtime     runtime des applications générées (copié dans src/vrs/mod.rs)
  vrs-codegen     génération du code des fenêtres, insertion des gestionnaires
  vrs-project     .vrsproj, modèles de projets, arborescence
  vrs-compiler    cargo build/run/check/clippy/test/clean, diagnostics JSON
  vrs-editor      éditeur de code (coloration, recherche, diagnostics, rustfmt)
  vrs-designer    concepteur visuel (surface, propriétés, boîte à outils, annuler)
  vrs-app         binaire `vrs` : fenêtre principale, menus, panneaux, dialogues
locales/<BCP 47>/*.ftl
```

Dépendances (sans cycle) : `vrs-app` → `vrs-designer`, `vrs-editor`,
`vrs-project`, `vrs-compiler`, `vrs-settings`, `vrs-i18n`. `vrs-project` →
`vrs-codegen` → `vrs-components` → `vrs-core`. `vrs-designer` → `vrs-runtime`.

`vrs-debugger` n'existe pas encore : il sera créé avec l'intégration DAP. Il
n'y a pas de crate séparé pour un « noyau » d'IDE au-delà de `vrs-core` : le
découpage actuel suffit.

### 3. Projet généré (application graphique)

```
MonApp/
  Cargo.toml                    eframe = "0.37" (seule dépendance)
  MonApp.vrsproj                paramètres VRS (non nécessaire pour compiler)
  forms/Form1.vrsform           conception (métadonnées RON)
  src/main.rs                   créé une fois — appartient à l'utilisateur
  src/vrs/mod.rs                runtime (régénéré)
  src/forms/mod.rs              liste des fenêtres (régénéré)
  src/forms/form1_designer.rs   champs + rendu + répartition des événements (régénéré)
  src/forms/form1.rs            CODE UTILISATEUR — jamais réécrit
```

#### Séparation code généré / code utilisateur

- Les fichiers régénérés portent un en-tête `<auto-generated>`. L'éditeur
  affiche un avertissement quand on les ouvre.
- `form1.rs` n'est **jamais réécrit**. Quand un événement est ajouté, VRS
  analyse le fichier avec `syn` :
  - si la méthode existe déjà, rien n'est fait ;
  - sinon, la méthode est insérée avant l'accolade fermante du dernier
    `impl Form1`, après une copie `.bak` et par écriture atomique ;
  - si le fichier contient une erreur de syntaxe, **rien n'est modifié** et
    l'erreur est signalée.

#### Modèle d'événements (emprunts)

```rust
fn show(&mut self, ui: &mut egui::Ui, app: &mut AppContext) {
    let button1 = vrs::button(ui, "button1", &self.button1);   // 1. dessin
    let _ = vrs::text_box(ui, "text_box1", &mut self.text_box1);
    if button1.clicked() {                                      // 2. répartition
        self.button1_click(app);                                //    &mut self libre
    }
}
```

Les réponses egui sont des valeurs possédées : une fois le dessin terminé,
aucun emprunt n'est actif. Le gestionnaire reçoit donc `&mut self` et peut
modifier n'importe quel contrôle (`self.text_box1.text = …`).

### 4. Sécurité

- Le concepteur dessine le modèle déclaratif et n'exécute aucun code.
- Les fichiers `.ftl` sont des données, jamais du code.
- Un projet qui n'a pas été créé par VRS demande une confirmation avant toute
  commande Cargo (les scripts `build.rs` et les macros procédurales sont du
  code exécutable). Le choix est mémorisé dans `trusted_projects`.
- Toutes les écritures sont atomiques. Une copie `.bak` est faite avant toute
  modification d'un fichier utilisateur.
- Aucune télémétrie.
