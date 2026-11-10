# Roadmap / Feuille de route

*[English](#english) · [Français](#français)*

---

## English

Each phase must deliver verifiable features. Detailed status is in
[STATUS.md](STATUS.md).

### Phase 1 — Foundation ✅
Modular Cargo workspace, main window, menus, toolbar, explorer, settings,
multilingual system (fr-CA, fr-FR, en-US, en-CA, es-ES).

### Phase 2 — Rust projects ✅
Creation (graphical, console, library), opening an existing Cargo project,
file explorer, saving, Cargo build/run/check/clippy/test/clean, output and
error list.

### Phase 3 — First visual designer ✅
Form1, Button, Label, TextBox; selection, moving, resizing, grid; properties;
copy/paste; undo/redo; Format menu (align, make same size, distribute, order);
interactive preview; `.vrsform` saving.

### Phase 4 — Generated Rust code ✅
Window code generation, user code protection, Cargo build and execution
independent of VRS. The Click handler is created by double-click (ahead of
phase 5).

### Phase 5 — Events and multiple windows (next)
- Multi-window runtime: `app.show_form(Form2::default())`,
  `app.show_dialog(…)` (simulated modal), `app.close()`, multiple instances,
  close strategy (`close_strategy` in `.vrsproj`). Planned implementation:
  immediate egui viewports, managed by a central window manager in
  `vrs-runtime`.
- More events: DoubleClick, Closing/Closed, Resize, KeyDown/KeyUp.
- Passing data between windows (public fields, dialog return value).
- Choosing the startup window in the IDE.
- Renaming a window (files and types).

### Phase 6 — Smart editor (started)
Done: VRS-aware completion, VB.NET equivalents, quick fixes (VRS hints and
rustc suggestions), check on save.
- rust-analyzer over LSP (`lsp-types`, separate process): live diagnostics,
  completion, hover, go to definition, rename.
- Quick Fix: light bulb, change preview, undoable application (from rustc
  suggestions and LSP code actions).
- More teaching explanations per error code (`errors.ftl`).
- Bracket matching, minimap.

### Phase 7 — Advanced IDE
- Debugging via DAP (CodeLLDB / lldb-dap, GDB): breakpoints, stepping,
  variables, call stack, threads, panics. `vrs-debugger` crate.
- Components: CheckBox, RadioButton, ComboBox, ListBox, ProgressBar, Slider,
  Image, Panel, GroupBox, TabControl, Menu, ContextMenu.
- Adaptive layouts (Anchor/Dock, layouts).
- Resources (images), control fonts and colors (Background, Foreground, Font).
- Internationalization of generated applications (separate from the IDE's).
- Extension system.
- Dockable panels (when egui_dock supports egui 0.37, or an alternative).
- Right-to-left languages.
- Future targets: Android, iOS (not started).

---

## Français

Chaque phase doit produire des fonctionnalités vérifiables. L'état détaillé se
trouve dans [STATUS.md](STATUS.md).

### Phase 1 — Fondation ✅
Workspace Cargo modulaire, fenêtre principale, menus, barre d'outils,
explorateur, paramètres, système multilingue (fr-CA, fr-FR, en-US, en-CA,
es-ES).

### Phase 2 — Projets Rust ✅
Création (graphique, console, bibliothèque), ouverture d'un projet Cargo
existant, explorateur de fichiers, enregistrement, Cargo build/run/check/
clippy/test/clean, sortie et liste d'erreurs.

### Phase 3 — Premier concepteur visuel ✅
Form1, Button, Label, TextBox ; sélection, déplacement, redimensionnement,
grille ; propriétés ; copier/coller ; annuler/rétablir ; menu Format
(aligner, uniformiser, répartir, ordre) ; aperçu interactif ; sauvegarde
`.vrsform`.

### Phase 4 — Code Rust généré ✅
Génération du code des fenêtres, protection du code utilisateur, compilation
Cargo et exécution indépendante de VRS. Le gestionnaire Click est créé par
double-clic (une avance sur la phase 5).

### Phase 5 — Événements et multifenêtrage (prochaine étape)
- Runtime multifenêtre : `app.show_form(Form2::default())`,
  `app.show_dialog(…)` (modal simulé), `app.close()`, plusieurs instances,
  stratégie de fermeture (`close_strategy` du `.vrsproj`).
  Implémentation prévue : viewports egui *immediate*, gérés par un
  gestionnaire central de fenêtres dans `vrs-runtime`.
- Événements supplémentaires : DoubleClick, Closing/Closed, Resize, KeyDown/KeyUp.
- Transfert de données entre fenêtres (champs publics, valeur de retour d'un dialogue).
- Choix de la fenêtre de démarrage dans l'IDE.
- Renommage d'une fenêtre (fichiers et types).

### Phase 6 — Éditeur intelligent (commencée)
Fait : autocomplétion propre à VRS, équivalences VB.NET, corrections rapides
(conseils VRS et suggestions de rustc), vérification à l'enregistrement.
- rust-analyzer via LSP (`lsp-types`, processus séparé) : diagnostics en
  direct, autocomplétion, infobulles, aller à la définition, renommage.
- Quick Fix : ampoule, aperçu du changement, application annulable (à partir
  des suggestions de rustc et des *code actions* LSP).
- Davantage d'explications pédagogiques par code d'erreur (`errors.ftl`).
- Correspondance des parenthèses, minimap.

### Phase 7 — IDE avancé
- Débogage via DAP (CodeLLDB / lldb-dap, GDB) : points d'arrêt, pas à pas,
  variables, pile, threads, paniques. Crate `vrs-debugger`.
- Composants : CheckBox, RadioButton, ComboBox, ListBox, ProgressBar, Slider,
  Image, Panel, GroupBox, TabControl, Menu, ContextMenu.
- Mises en page adaptatives (ancrage, Anchor/Dock, layouts).
- Ressources (images), polices et couleurs des contrôles (Background,
  Foreground, Font).
- Internationalisation des applications générées (distincte de celle de l'IDE).
- Système d'extensions.
- Panneaux ancrables (quand egui_dock suivra egui 0.37, ou alternative).
- Prise en charge des langues écrites de droite à gauche.
- Cibles futures : Android, iOS (non commencées).
