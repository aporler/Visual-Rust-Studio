# Visual Rust Studio — aide au code (Français, Canada)

editor-quick-fix = Correction rapide
editor-quick-fix-hint = Cliquez pour voir la correction rapide.
editor-completion-keys = ↑ ↓ pour choisir · Entrée ou Tab pour insérer · Échap pour fermer · Ctrl+Espace pour ouvrir
fix-replace = Remplacer par `{ $replacement }`
fix-apply-suggestion = Appliquer la suggestion du compilateur : `{ $replacement }`

## Autocomplétion

completion-keyword = Mot-clé de Rust.
completion-control = Contrôle { $kind } de cette fenêtre.
api-self = La fenêtre elle-même (Me en VB.NET). Exemple : `self.text_box1.text`.
api-app = L'application : `app.exit()` la ferme, `app.set_title("…")` change le titre de la fenêtre.
api-app-exit = Ferme l'application.
api-app-set-title = Change le titre de la fenêtre.
api-app-egui-ctx = Contexte egui, pour les usages avancés (redessin, styles…).
api-text = Texte du contrôle (String). Exemple : `self.text_box1.text = String::from("Bonjour");`
api-visible = Affiche (`true`) ou masque (`false`) le contrôle.
api-enabled = Active (`true`) ou désactive (`false`) le contrôle.
api-tooltip = Texte affiché au survol de la souris (String).
api-bounds = Position et taille du contrôle : `x`, `y`, `width`, `height`.
api-bounds-field = `{ $field }` en points (f32). Exemple : `self.button1.bounds.x = 20.0;`
api-string-push-str = Ajoute du texte à la fin.
api-string-clear = Vide le texte.
api-string-len = Longueur en octets.
api-string-is-empty = `true` si le texte est vide.
api-string-trim = Le texte sans espaces au début et à la fin.
api-string-to-uppercase = Le texte en majuscules.
api-string-to-lowercase = Le texte en minuscules.
api-string-contains = `true` si le texte contient le texte donné.
api-string-parse = Convertit le texte en nombre : donne `Ok(nombre)` ou `Err(erreur)`.
snippet-println = Écrit une ligne dans la console (comme Console.WriteLine).
snippet-format = Construit une String avec des valeurs : `format!("Total : {"{"}{"}"}", total)`.
snippet-string-from = Crée une String à partir d'un texte entre guillemets.
snippet-if = Condition.
snippet-for = Boucle sur une collection ou un intervalle (`0..10`).
snippet-some = Une valeur présente (Option).
snippet-none = Aucune valeur (Option) — l'équivalent de Nothing.
snippet-ok = Succès (Result).
snippet-err = Erreur (Result).
snippet-vec-new = Nouvelle liste vide (Vec).

## Équivalences VB.NET

hint-close = Dans Visual Rust Studio, `app.exit()` ferme l'application (l'équivalent de Me.Close() sur la fenêtre principale).
hint-me = En Rust, la fenêtre elle-même s'appelle `self` (Me en VB.NET).
hint-title = Pour changer le titre de la fenêtre (Me.Text en VB.NET) : `app.set_title("…")`.
hint-control-name = En Rust, le contrôle { $vb } s'écrit `self.{ $name }` (les noms sont en snake_case).
hint-dim = En Rust, on déclare une variable avec `let` (`let mut` si sa valeur change).
hint-bool = En Rust, les booléens s'écrivent en minuscules : `true` et `false`.
hint-nothing = Rust n'a pas de Nothing/null : une valeur optionnelle est une `Option`, et son absence s'écrit `None`.
hint-print = Pour écrire dans la console : `println!("…");` (Console.WriteLine en VB.NET).

api-app-close = Ferme cette fenêtre (l'application reste ouverte s'il reste d'autres fenêtres).
api-app-hide = Masque cette fenêtre ; elle garde son état.
api-app-set-size = Change la taille de cette fenêtre : `app.set_size(400.0, 300.0)`.
api-app-show-form = Ouvre une nouvelle fenêtre { $form }. Retourne un FormId utilisable avec `close_form`, `hide_form` ou `with_form`.
api-app-show-dialog = Ouvre { $form } en fenêtre modale : les autres fenêtres sont désactivées jusqu'à sa fermeture. Utilisez `show_dialog_then` pour récupérer ses données.
api-app-show-hidden = Affiche de nouveau une fenêtre masquée (FormId).
api-app-with-form = Lit ou modifie les contrôles d'une autre fenêtre ouverte : `app.with_form::<Form2, _>(id, |f| f.text_box1.text.clone())`.
hint-show-form = Dans Visual Rust Studio, on ouvre une fenêtre avec `app.show_form({ $form }::default())` (Form.Show() en VB.NET).
hint-show-dialog = Fenêtre modale : `app.show_dialog({ $form }::default())` (ShowDialog() en VB.NET). Pour récupérer ses données à la fermeture : `app.show_dialog_then(…)`.
hint-hide = Pour masquer la fenêtre en cours : `app.hide()` (Me.Hide() en VB.NET).


api-checked = `true` si la case est cochée (bool).
api-group = Groupe de boutons d'option qui s'excluent (String).
api-items = Liste des éléments (Vec<String>). Exemple : `self.list_box1.items.push(String::from("Nouveau"));`
api-selected = Index de l'élément sélectionné (`Option<usize>`) : `Some(0)` pour le premier, `None` si rien n'est sélectionné.
api-selected-text = Texte de l'élément sélectionné (`Option<&str>`).
api-value = Valeur actuelle (f32).
api-minimum = Valeur minimale (f32).
api-maximum = Valeur maximale (f32).
api-app-message-box = Affiche un message dans une boîte modale avec un bouton OK (MsgBox en VB.NET).
hint-msgbox = Dans Visual Rust Studio, on affiche un message avec `app.message_box("…")` (MsgBox / MessageBox.Show en VB.NET).
editor-rename = Nouveau nom (Entrée pour renommer partout, Échap pour annuler)

api-back-color = Couleur de fond (`Option<vrs::Color>`), `None` pour la couleur du thème.
api-fore-color = Couleur du texte (`Option<vrs::Color>`), `None` pour la couleur du thème.
api-font-size = Taille de la police en points (`Option<f32>`).
api-anchor = Côtés auxquels le contrôle reste attaché : `vrs::Anchor::new(gauche, haut, droite, bas)`.
api-image = Image affichée (`Option<egui::ImageSource>`), intégrée au programme avec `egui::include_image!`.
