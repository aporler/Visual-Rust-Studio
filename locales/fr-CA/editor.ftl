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
