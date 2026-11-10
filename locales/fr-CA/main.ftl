# Visual Rust Studio — Français (Canada)

language-name = Français (Canada)
app-title = Visual Rust Studio
app-title-project = { $project } — Visual Rust Studio
app-tagline = La simplicité du développement visuel, la puissance de Rust.

## Menus

menu-file = Fichier
menu-edit = Édition
menu-view = Affichage
menu-project = Projet
menu-build = Compiler
menu-debug = Déboguer
menu-format = Format
menu-tools = Outils
menu-window = Fenêtre
menu-help = Aide
menu-recent-projects = Projets récents

## Commandes

cmd-new-project = Nouveau projet…
cmd-open-project = Ouvrir un projet…
cmd-save = Enregistrer
cmd-save-all = Enregistrer tout
cmd-close-project = Fermer le projet
cmd-exit = Quitter
cmd-undo = Annuler
cmd-redo = Rétablir
cmd-cut = Couper
cmd-copy = Copier
cmd-paste = Coller
cmd-delete = Supprimer
cmd-select-all = Tout sélectionner
cmd-find = Rechercher et remplacer…
cmd-format-document = Mettre en forme le document (rustfmt)
cmd-view-explorer = Explorateur de projet
cmd-view-properties = Propriétés
cmd-view-toolbox = Boîte à outils
cmd-view-output = Sortie et erreurs
cmd-view-code = Afficher le code
cmd-view-designer = Afficher le concepteur
cmd-add-form = Ajouter une fenêtre…
cmd-generate-code = Générer le code
cmd-reload-project = Recharger le projet
cmd-build = Compiler
cmd-run = Exécuter
cmd-check = Vérifier (cargo check)
cmd-clippy = Analyser (Clippy)
cmd-test = Exécuter les tests
cmd-clean = Nettoyer
cmd-stop = Arrêter
cmd-release-mode = Mode Release
cmd-debug-start = Démarrer le débogage
cmd-settings = Paramètres…
cmd-close-tab = Fermer l'onglet
cmd-close-all-tabs = Fermer tous les onglets
cmd-about = À propos de Visual Rust Studio
cmd-rust-book = Le livre de Rust (en ligne)
cmd-align-left = Aligner à gauche
cmd-align-right = Aligner à droite
cmd-align-top = Aligner en haut
cmd-align-bottom = Aligner en bas
cmd-align-center-h = Centrer horizontalement
cmd-align-center-v = Centrer verticalement
cmd-same-width = Uniformiser la largeur
cmd-same-height = Uniformiser la hauteur
cmd-same-size = Uniformiser la taille
cmd-distribute-h = Répartir horizontalement
cmd-distribute-v = Répartir verticalement
cmd-bring-front = Mettre au premier plan
cmd-send-back = Mettre à l'arrière-plan
debug-not-available = Le débogueur arrivera dans une version ultérieure (LLDB/GDB via DAP).

## Boutons communs

button-save = Enregistrer
button-cancel = Annuler
button-ok = OK
button-close = Fermer
button-browse = Parcourir…
button-create = Créer
button-open = Ouvrir
button-discard = Ne pas enregistrer
button-trust = Faire confiance et continuer
button-next = Suivant
button-previous = Précédent
button-replace = Remplacer
button-replace-all = Tout remplacer
button-clear = Effacer

## Panneaux

panel-explorer = Explorateur de projet
panel-properties = Propriétés
panel-toolbox = Boîte à outils
panel-output = Sortie
panel-errors = Liste d'erreurs
tab-start = Page de démarrage
explorer-forms = Fenêtres
explorer-files = Fichiers
explorer-no-project = Aucun projet ouvert.
explorer-open-designer = Ouvrir le concepteur
explorer-view-code = Afficher le code

## Page de démarrage

start-welcome = Bienvenue dans Visual Rust Studio
start-new-project = Créer un nouveau projet
start-open-project = Ouvrir un projet
start-recent = Projets récents
start-no-recent = Aucun projet récent.
start-learn = Chaque fenêtre que vous concevez devient du code Rust lisible, qui compile avec Cargo, avec ou sans Visual Rust Studio.

## Assistant de nouveau projet

newproj-title = Nouveau projet
newproj-template = Modèle
template-gui = Application graphique Rust
template-gui-desc = Une fenêtre (Form1) conçue visuellement, avec du code Rust généré (egui/eframe).
template-console = Application console Rust
template-console-desc = Un programme en ligne de commande.
template-library = Bibliothèque Rust
template-library-desc = Une crate réutilisable avec un test unitaire.
template-existing = Projet Cargo existant
template-existing-desc = Ouvrir un dossier qui contient déjà un Cargo.toml.
newproj-name = Nom du projet
newproj-location = Emplacement
newproj-path-preview = Le projet sera créé dans : { $path }
newproj-invalid-name = Utilisez des lettres, des chiffres, « - » ou « _ », en commençant par une lettre.
newproj-exists = Ce dossier existe déjà.
newproj-choose-folder = Choisir un dossier

## Ajout d'une fenêtre

addform-title = Ajouter une fenêtre
addform-name = Nom de la fenêtre
addform-invalid = Le nom doit commencer par une majuscule et ne contenir que des lettres et des chiffres (ex. : Form2).
addform-exists = Une fenêtre porte déjà ce nom.

## Paramètres

settings-title = Paramètres
settings-language = Langue
settings-language-auto = Automatique (système : { $lang })
settings-language-chain = Ordre de repli : { $chain }
settings-language-external = Fichiers de traduction supplémentaires : { $path }
settings-missing-keys = { $count ->
    [0] Aucune traduction manquante.
    [one] Une traduction manquante (l'anglais est affiché).
   *[other] { $count } traductions manquantes (l'anglais est affiché).
}
settings-appearance = Apparence
settings-theme = Thème
theme-system = Suivre le système
theme-light = Clair
theme-dark = Sombre
settings-ui-scale = Zoom de l'interface
settings-editor = Éditeur de code
settings-editor-font-size = Taille de police
settings-tab-width = Largeur d'indentation
settings-format-on-save = Mettre en forme avec rustfmt à l'enregistrement
settings-check-on-save = Vérifier le code (cargo check) à l'enregistrement
settings-lsp = Utiliser rust-analyzer (diagnostics en direct, autocomplétion, infobulles, définition avec F12, renommage avec F2)
settings-lsp-missing = rust-analyzer n'est pas installé : rustup component add rust-analyzer
settings-toolchain = Chaîne d'outils Rust
settings-cargo-path = Chemin de cargo
settings-cargo-path-hint = Laisser vide pour la détection automatique.
settings-cargo-detected = Détecté : { $path }
settings-cargo-not-found = Cargo est introuvable. Installez Rust depuis https://rustup.rs
settings-designer = Concepteur
settings-grid-size = Taille de la grille
settings-snap-to-grid = Aligner sur la grille
settings-show-grid = Afficher la grille
settings-config-file = Fichier de paramètres : { $path }

## Barre d'état

status-ready = Prêt
status-errors = { $count ->
    [0] 0 erreur
    [one] { $count } erreur
   *[other] { $count } erreurs
}
status-warnings = { $count ->
    [0] 0 avertissement
    [one] { $count } avertissement
   *[other] { $count } avertissements
}
status-running = En cours : { $command }
status-build-succeeded = Terminé avec succès.
status-build-failed = Échec (code { $code }).
status-process-stopped = Processus arrêté.
status-saved = Enregistré : { $file }
status-project-opened = Projet ouvert : { $name }
status-project-created = Projet créé : { $name }
status-code-generated = Code généré ({ $count } fichier(s) mis à jour).
status-language-changed = Langue modifiée.
status-background-check = Vérification du code…
status-release = Release
status-debug-build = Debug

## Messages et erreurs

error-title = Erreur
error-open-project = Impossible d'ouvrir le projet : { $error }
error-create-project = Impossible de créer le projet : { $error }
error-save = Impossible d'enregistrer : { $error }
error-cargo-not-found = Cargo est introuvable. Vérifiez Outils → Paramètres → Chaîne d'outils Rust.
error-codegen = La génération du code a échoué : { $error }
error-already-running = Une commande est déjà en cours. Arrêtez-la d'abord.
error-no-project = Ouvrez ou créez d'abord un projet.
error-read-file = Impossible de lire le fichier : { $error }

trust-title = Projet non approuvé
trust-message = Ce projet n'a pas été créé par Visual Rust Studio. Le compiler peut exécuter du code (scripts build.rs, macros procédurales, dépendances). Ne continuez que si vous faites confiance à sa provenance.
    { $path }

unsaved-title = Modifications non enregistrées
unsaved-message = Des fichiers ont été modifiés. Voulez-vous les enregistrer avant de fermer ?

about-title = À propos de Visual Rust Studio
about-version = Version { $version }
about-description = Un environnement de développement visuel libre pour Rust.
about-license = Logiciel libre sous licence GNU AGPL-3.0. Le code de vos applications vous appartient.
about-trademarks = Microsoft, Visual Studio, Visual Basic, .NET et Windows Forms sont des marques de Microsoft Corporation. Rust est une marque de la Rust Foundation. Visual Rust Studio est un projet indépendant, non affilié à Microsoft ni à la Rust Foundation, et non approuvé par elles.

## Sortie et diagnostics

output-empty = Aucune sortie pour l'instant. Utilisez Compiler → Compiler (Ctrl+Maj+B) ou Exécuter (F5).
errors-empty = Aucun diagnostic. Lancez Compiler → Vérifier pour analyser le code.
diag-level-error = Erreur
diag-level-warning = Avertissement
diag-level-note = Note
diag-level-help = Aide
diag-explanation = Explication
diag-original = Message original du compilateur
diag-open-location = Aller à l'emplacement
diag-suggestion = Correction suggérée : { $replacement }
filter-errors = Erreurs
filter-warnings = Avertissements

## Éditeur de code

editor-find-placeholder = Rechercher
editor-replace-placeholder = Remplacer par
editor-matches = { $total ->
    [0] Aucun résultat
    [one] 1 résultat
   *[other] { $total } résultats
}
editor-line-col = Ligne { $line }, Col { $col }
editor-modified = Modifié
editor-generated-warning = Ce fichier est généré par Visual Rust Studio et sera écrasé. Écrivez votre code dans { $file }.

## Commentaires insérés dans le code généré

codegen-handler-doc = Gère l'événement { $event } de `{ $control }`.
codegen-todo = À FAIRE : écrivez votre code ici.
codegen-user-file-doc = Code de la fenêtre { $form }. Ce fichier vous appartient : Visual Rust Studio y ajoute seulement de nouveaux gestionnaires d'événements et ne modifie jamais le code existant.

cmd-set-startup-form = Définir comme fenêtre de démarrage
menu-close-strategy = À la fermeture de la fenêtre principale
close-strategy-main = Fermer l'application
close-strategy-last = La masquer ; fermer l'application avec la dernière fenêtre
