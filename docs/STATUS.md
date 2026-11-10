# Status — version 0.1 / État d'avancement — version 0.1

*[English](#english) · [Français](#français)*

---

## English

### Platforms

- Developed and tested on **Linux x86_64**.
- **Windows, macOS and ARM64**: supported by the architecture (cross-platform
  dependencies), but not tested yet.
- UI tests use `egui_kittest`, which runs egui's drawing and interaction code
  without a window.

### Automated checks

| Command | Result |
|---|---|
| `cargo test --workspace` | unit tests and UI tests |
| `cargo test -p vrs-project --test standalone -- --ignored` | generated graphical, console and library projects build **outside the VRS workspace**, with no warnings |
| `cargo test -p vrs-app --test gui -- --ignored` | in the IDE: type error → `cargo check` → E0308 diagnostic + French explanation + underline in the editor |
| `cargo clippy --workspace --all-targets` | no warnings |
| `cargo fmt --all -- --check` | clean |
| `vrs-i18n-check` | no errors; fr-CA and en-US 100 % complete |

### MVP acceptance criteria

| # | Criterion | Status | Details |
|---|---|---|---|
| 1 | Start VRS | ✅ | `cargo run -p vrs-app` |
| 2 | Choose fr-CA or en-US | ✅ | Tools → Settings → Language; applies immediately, no restart (tested) |
| 3 | Create a graphical project | ✅ | New Project wizard (graphical, console, library, existing Cargo project) |
| 4 | Show Form1 | ✅ | opened automatically in the designer |
| 5 | Add a button and a text box | ✅ | drag and drop or click from the toolbox; Label too |
| 6 | Edit their properties | ✅ | Name, Text, X, Y, Width, Height, Visible, Enabled, Tooltip; window title and size |
| 7 | Save and reopen the project | ✅ | `.vrsproj` + `forms/*.vrsform`; recent projects |
| 8 | Generate valid Rust code | ✅ | `syn`/`quote`/`prettyplease`; builds without warnings |
| 9 | Create a Click event | ✅ | double-click the control, or the button in the Properties panel |
| 10 | Change the text box text on click | ✅ | `self.text_box1.text = …` in the handler; verified by a runtime test (simulated click) |
| 11 | Add Form2 | ✅ | Project → Add Window |
| 12 | Open and close Form2 | ⏳ **to do (phase 5)** | Form2 can be designed and builds, but the runtime cannot open it at run time yet. No placeholder method is exposed. |
| 13 | Build with Cargo | ✅ | Build, Run, Check, Clippy, Tests, Clean, Stop, Release mode |
| 14 | Run independently of VRS | ✅ | standalone Cargo project (copied runtime, only dependency: eframe) |
| 15 | Rust diagnostics and explanations | ✅ partial | `cargo check` diagnostics (list, underline, tooltip, original message, documentation link, rustc suggestions); explanations for 17 error codes and 4 lints; automatic check on save; compiler suggestions applicable as quick fixes. **No rust-analyzer yet.** |

### Other available features

- **Designer**: move, resize with 8 handles, snap-to-grid, arrow keys,
  multiple selection (Shift/Ctrl), copy/cut/paste, delete, undo/redo, Format
  menu (align, make same size, distribute, bring to front/send to back), window
  resizing, interactive preview, hidden controls drawn semi-transparent.
- **Editor**: Rust syntax highlighting, line numbers, auto-indent, tabs
  replaced by spaces, find/replace, go to line, rustfmt (command and on save),
  warning on generated files.
- **Code assistance**: completion while typing or with Ctrl+Space (window
  controls `self.text_box1`, their properties `.text`, `.visible`…, String
  methods, `app.exit()`, `app.set_title()`, Rust keywords and snippets), with a
  translated explanation of each item. VB.NET equivalents: `close` →
  `app.exit()`, `Me` → `self`, `TextBox1` → `self.text_box1`, `Dim` → `let`,
  `True` → `true`, `Nothing` → `None`, `Console.WriteLine` → `println!`.
  These habits are underlined as you type, with a 💡 quick fix.
- **Security**: confirmation before building a project not created by VRS;
  atomic writes; `.bak` copy before any insertion into user code; no change if
  that code does not parse.
- **Languages**: fr-CA and en-US complete. fr-FR complete in practice (its own
  main interface; designer and explanations come from fr-CA). es-ES: main
  interface translated, designer and explanations in English. en-CA: partial
  variant based on en-US. External translations load without recompiling.

### Known limitations

- No rust-analyzer/LSP yet: completion covers VRS controls and APIs, keywords
  and snippets, but not arbitrary types or crates; no go to definition or
  rename (phase 6).
- No debugger (phase 7). The Debug menu says so.
- Panels are resizable but not dockable (egui_dock is not compatible with
  egui 0.37).
- No bracket matching, no minimap.
- Renaming a window (Form1 → MainForm) is not possible yet.
- Background/Foreground/Font, Anchor and Layout are not properties yet.

---

## Français

### Plateformes

- Développé et testé sous **Linux x86_64**.
- **Windows, macOS et ARM64** : prévus par l'architecture (dépendances
  multiplateformes), mais pas encore testés.
- Les tests d'interface utilisent `egui_kittest`, qui exécute le dessin et les
  interactions d'egui sans fenêtre.

### Vérifications automatisées

| Commande | Résultat |
|---|---|
| `cargo test --workspace` | tests unitaires et tests d'interface |
| `cargo test -p vrs-project --test standalone -- --ignored` | projets graphique, console et bibliothèque générés, compilés **hors du workspace VRS**, sans aucun avertissement |
| `cargo test -p vrs-app --test gui -- --ignored` | dans l'IDE : erreur de type → `cargo check` → diagnostic E0308 + explication française + soulignement dans l'éditeur |
| `cargo clippy --workspace --all-targets` | aucun avertissement |
| `cargo fmt --all -- --check` | conforme |
| `vrs-i18n-check` | aucune erreur ; fr-CA et en-US complets à 100 % |

### Critères d'acceptation du MVP

| # | Critère | État | Détail |
|---|---|---|---|
| 1 | Démarrer VRS | ✅ | `cargo run -p vrs-app` |
| 2 | Choisir fr-CA ou en-US | ✅ | Outils → Paramètres → Langue ; changement immédiat sans redémarrage (testé) |
| 3 | Créer un projet graphique | ✅ | assistant Nouveau projet (graphique, console, bibliothèque, Cargo existant) |
| 4 | Afficher Form1 | ✅ | ouvert automatiquement dans le concepteur |
| 5 | Ajouter un bouton et un champ texte | ✅ | glisser-déposer ou clic depuis la boîte à outils ; Label aussi |
| 6 | Modifier leurs propriétés | ✅ | Name, Text, X, Y, Width, Height, Visible, Enabled, Tooltip ; titre et taille de la fenêtre |
| 7 | Sauvegarder et rouvrir le projet | ✅ | `.vrsproj` + `forms/*.vrsform` ; projets récents |
| 8 | Générer du code Rust valide | ✅ | `syn`/`quote`/`prettyplease` ; compile sans avertissement |
| 9 | Créer un événement Click | ✅ | double-clic sur le contrôle, ou bouton dans le panneau Propriétés |
| 10 | Modifier le texte du champ lors du clic | ✅ | `self.text_box1.text = …` dans le gestionnaire ; vérifié par un test du runtime (clic simulé) |
| 11 | Ajouter Form2 | ✅ | Projet → Ajouter une fenêtre |
| 12 | Ouvrir et fermer Form2 | ⏳ **à faire (phase 5)** | Form2 se conçoit et compile, mais le runtime ne sait pas encore l'ouvrir à l'exécution. Aucune méthode fictive n'est exposée. |
| 13 | Compiler avec Cargo | ✅ | Compiler, Exécuter, Vérifier, Clippy, Tests, Nettoyer, Arrêter, mode Release |
| 14 | Exécuter indépendamment de VRS | ✅ | projet Cargo autonome (runtime copié, seule dépendance : eframe) |
| 15 | Diagnostics Rust et explications | ✅ partiel | diagnostics de `cargo check` (liste, soulignement, infobulle, message original, lien vers la documentation, suggestions de rustc) ; explications pour 17 codes d'erreur et 4 lints ; vérification automatique à l'enregistrement ; suggestions du compilateur applicables en correction rapide. **Pas encore de rust-analyzer.** |

### Autres fonctions disponibles

- **Concepteur** : déplacement, redimensionnement par 8 poignées, grille
  magnétique, flèches du clavier, sélection multiple (Maj/Ctrl), copier/
  couper/coller, supprimer, annuler/rétablir, menu Format (aligner,
  uniformiser, répartir, premier plan/arrière-plan), redimensionnement de la
  fenêtre, aperçu interactif, contrôles masqués affichés en transparence.
- **Éditeur** : coloration Rust, numéros de ligne, indentation automatique,
  tabulation remplacée par des espaces, rechercher/remplacer, aller à la ligne,
  rustfmt (commande et à l'enregistrement), avertissement sur les fichiers
  générés.
- **Aide au code** : autocomplétion pendant la frappe ou avec Ctrl+Espace
  (contrôles de la fenêtre `self.text_box1`, leurs propriétés `.text`,
  `.visible`…, méthodes de String, `app.exit()`, `app.set_title()`, mots-clés
  et extraits Rust), avec une explication traduite de chaque proposition.
  Équivalences VB.NET : `close` → `app.exit()`, `Me` → `self`, `TextBox1` →
  `self.text_box1`, `Dim` → `let`, `True` → `true`, `Nothing` → `None`,
  `Console.WriteLine` → `println!`. Ces habitudes sont soulignées pendant la
  frappe, avec une correction rapide 💡.
- **Sécurité** : confirmation avant de compiler un projet qui n'a pas été créé
  par VRS ; écritures atomiques ; copie `.bak` avant toute insertion dans le
  code utilisateur ; aucune modification si ce code ne s'analyse pas.
- **Langues** : fr-CA et en-US complets. fr-FR complet en pratique (l'interface
  principale lui est propre, le concepteur et les explications viennent de
  fr-CA). es-ES : interface principale traduite, concepteur et explications
  en anglais. en-CA : variante partielle qui s'appuie sur en-US.
  Traductions externes chargées sans recompilation.

### Limites connues

- Pas encore de rust-analyzer/LSP : l'autocomplétion couvre les contrôles et
  les API de VRS, les mots-clés et les extraits, mais pas les types et crates
  quelconques ; pas de navigation vers les définitions ni de renommage
  (phase 6).
- Pas de débogueur (phase 7). Le menu Débogage l'indique clairement.
- Panneaux redimensionnables mais non ancrables (egui_dock incompatible avec
  egui 0.37).
- Pas de correspondance des parenthèses ni de minimap.
- Le renommage d'une fenêtre (Form1 → MainForm) n'est pas encore possible.
- Les polices Background/Foreground/Font, Anchor et Layout ne sont pas encore
  des propriétés.
