# Visual Rust Studio — code assistance (English, United States)

editor-quick-fix = Quick fix
editor-quick-fix-hint = Click to see the quick fix.
editor-completion-keys = ↑ ↓ to choose · Enter or Tab to insert · Esc to close · Ctrl+Space to open
fix-replace = Replace with `{ $replacement }`
fix-apply-suggestion = Apply the compiler's suggestion: `{ $replacement }`

## Completion

completion-keyword = Rust keyword.
completion-control = { $kind } control of this window.
api-self = The window itself (Me in VB.NET). Example: `self.text_box1.text`.
api-app = The application: `app.exit()` closes it, `app.set_title("…")` changes the window title.
api-app-exit = Closes the application.
api-app-set-title = Changes the window title.
api-app-egui-ctx = egui context, for advanced uses (repaint, styles…).
api-text = Text of the control (String). Example: `self.text_box1.text = String::from("Hello");`
api-visible = Shows (`true`) or hides (`false`) the control.
api-enabled = Enables (`true`) or disables (`false`) the control.
api-tooltip = Text shown when the mouse hovers the control (String).
api-bounds = Position and size of the control: `x`, `y`, `width`, `height`.
api-bounds-field = `{ $field }` in points (f32). Example: `self.button1.bounds.x = 20.0;`
api-string-push-str = Appends text at the end.
api-string-clear = Empties the text.
api-string-len = Length in bytes.
api-string-is-empty = `true` if the text is empty.
api-string-trim = The text without leading and trailing spaces.
api-string-to-uppercase = The text in uppercase.
api-string-to-lowercase = The text in lowercase.
api-string-contains = `true` if the text contains the given text.
api-string-parse = Converts the text to a number: returns `Ok(number)` or `Err(error)`.
snippet-println = Writes a line to the console (like Console.WriteLine).
snippet-format = Builds a String with values: `format!("Total: {"{"}{"}"}", total)`.
snippet-string-from = Creates a String from text in quotes.
snippet-if = Condition.
snippet-for = Loop over a collection or a range (`0..10`).
snippet-some = A present value (Option).
snippet-none = No value (Option) — the equivalent of Nothing.
snippet-ok = Success (Result).
snippet-err = Error (Result).
snippet-vec-new = New empty list (Vec).

## VB.NET equivalents

hint-close = In Visual Rust Studio, `app.exit()` closes the application (the equivalent of Me.Close() on the main window).
hint-me = In Rust, the window itself is `self` (Me in VB.NET).
hint-title = To change the window title (Me.Text in VB.NET): `app.set_title("…")`.
hint-control-name = In Rust, the control { $vb } is written `self.{ $name }` (names are in snake_case).
hint-dim = In Rust, declare a variable with `let` (`let mut` if its value changes).
hint-bool = Rust booleans are written in lowercase: `true` and `false`.
hint-nothing = Rust has no Nothing/null: an optional value is an `Option`, and its absence is `None`.
hint-print = To write to the console: `println!("…");` (Console.WriteLine in VB.NET).

api-app-close = Closes this window (the application stays open if other windows remain).
api-app-hide = Hides this window; it keeps its state.
api-app-set-size = Changes the size of this window: `app.set_size(400.0, 300.0)`.
api-app-show-form = Opens a new { $form } window. It returns a FormId to use with `close_form`, `hide_form` or `with_form`.
api-app-show-dialog = Opens { $form } as a modal dialog: the other windows are disabled until it closes. Use `show_dialog_then` to get its data back.
api-app-show-hidden = Shows a hidden window again (FormId).
api-app-with-form = Reads or changes the controls of another open window: `app.with_form::<Form2, _>(id, |f| f.text_box1.text.clone())`.
hint-show-form = In Visual Rust Studio, a window is opened with `app.show_form({ $form }::default())` (Form.Show() in VB.NET).
hint-show-dialog = Modal window: `app.show_dialog({ $form }::default())` (ShowDialog() in VB.NET). To get data back when it closes: `app.show_dialog_then(…)`.
hint-hide = To hide the current window: `app.hide()` (Me.Hide() in VB.NET).


api-checked = `true` if checked (bool).
api-group = Group of mutually exclusive radio buttons (String).
api-items = List of items (Vec<String>). Example: `self.list_box1.items.push(String::from("New"));`
api-selected = Index of the selected item (`Option<usize>`): `Some(0)` for the first, `None` if nothing is selected.
api-selected-text = Text of the selected item (`Option<&str>`).
api-value = Current value (f32).
api-minimum = Minimum value (f32).
api-maximum = Maximum value (f32).
api-app-message-box = Shows a message in a modal box with an OK button (MsgBox in VB.NET).
hint-msgbox = In Visual Rust Studio, a message is shown with `app.message_box("…")` (MsgBox / MessageBox.Show in VB.NET).
editor-rename = New name (Enter to rename everywhere, Esc to cancel)

api-back-color = Background color (`Option<vrs::Color>`), `None` for the theme color.
api-fore-color = Text color (`Option<vrs::Color>`), `None` for the theme color.
api-font-size = Font size in points (`Option<f32>`).
api-anchor = Sides the control stays attached to: `vrs::Anchor::new(left, top, right, bottom)`.
