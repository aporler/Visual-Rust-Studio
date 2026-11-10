# Visual Rust Studio — English (United States)
# Reference locale: every key must exist here.

language-name = English (United States)
app-title = Visual Rust Studio
app-title-project = { $project } — Visual Rust Studio
app-tagline = The simplicity of visual development, the power of Rust.

## Menus

menu-file = File
menu-edit = Edit
menu-view = View
menu-project = Project
menu-build = Build
menu-debug = Debug
menu-format = Format
menu-tools = Tools
menu-window = Window
menu-help = Help
menu-recent-projects = Recent Projects

## Commands

cmd-new-project = New Project…
cmd-open-project = Open Project…
cmd-save = Save
cmd-save-all = Save All
cmd-close-project = Close Project
cmd-exit = Exit
cmd-undo = Undo
cmd-redo = Redo
cmd-cut = Cut
cmd-copy = Copy
cmd-paste = Paste
cmd-delete = Delete
cmd-select-all = Select All
cmd-find = Find and Replace…
cmd-format-document = Format Document (rustfmt)
cmd-view-explorer = Project Explorer
cmd-view-properties = Properties
cmd-view-toolbox = Toolbox
cmd-view-output = Output and Errors
cmd-view-code = View Code
cmd-view-designer = View Designer
cmd-add-form = Add Window…
cmd-generate-code = Generate Code
cmd-reload-project = Reload Project
cmd-build = Build
cmd-run = Run
cmd-check = Check (cargo check)
cmd-clippy = Analyze (Clippy)
cmd-test = Run Tests
cmd-clean = Clean
cmd-stop = Stop
cmd-release-mode = Release Mode
cmd-debug-start = Start Debugging
cmd-settings = Settings…
cmd-close-tab = Close Tab
cmd-close-all-tabs = Close All Tabs
cmd-about = About Visual Rust Studio
cmd-rust-book = The Rust Book (online)
cmd-align-left = Align Lefts
cmd-align-right = Align Rights
cmd-align-top = Align Tops
cmd-align-bottom = Align Bottoms
cmd-align-center-h = Align Centers (horizontal)
cmd-align-center-v = Align Middles (vertical)
cmd-same-width = Make Same Width
cmd-same-height = Make Same Height
cmd-same-size = Make Same Size
cmd-distribute-h = Distribute Horizontally
cmd-distribute-v = Distribute Vertically
cmd-bring-front = Bring to Front
cmd-send-back = Send to Back

## Common buttons

button-save = Save
button-cancel = Cancel
button-ok = OK
button-close = Close
button-browse = Browse…
button-create = Create
button-open = Open
button-discard = Don't Save
button-trust = Trust and Continue
button-next = Next
button-previous = Previous
button-replace = Replace
button-replace-all = Replace All
button-clear = Clear

## Panels

panel-explorer = Project Explorer
panel-properties = Properties
panel-toolbox = Toolbox
panel-output = Output
panel-errors = Error List
tab-start = Start Page
explorer-forms = Windows
explorer-files = Files
explorer-no-project = No project is open.
explorer-open-designer = Open Designer
explorer-view-code = View Code

## Start page

start-welcome = Welcome to Visual Rust Studio
start-new-project = Create a new project
start-open-project = Open a project
start-recent = Recent projects
start-no-recent = No recent projects.
start-learn = Every window you design becomes readable Rust code that compiles with Cargo, with or without Visual Rust Studio.

## New project dialog

newproj-title = New Project
newproj-template = Template
template-gui = Rust Graphical Application
template-gui-desc = A window (Form1) designed visually, with generated Rust code (egui/eframe).
template-console = Rust Console Application
template-console-desc = A command-line program.
template-library = Rust Library
template-library-desc = A reusable crate with a unit test.
template-existing = Existing Cargo Project
template-existing-desc = Open a folder that already contains a Cargo.toml.
newproj-name = Project name
newproj-location = Location
newproj-path-preview = The project will be created in: { $path }
newproj-invalid-name = Use letters, digits, “-” or “_”, starting with a letter.
newproj-exists = This folder already exists.
newproj-choose-folder = Choose a folder

## Add window dialog

addform-title = Add Window
addform-name = Window name
addform-invalid = The name must start with an uppercase letter and contain only letters and digits (e.g. Form2).
addform-exists = A window with this name already exists.

## Settings

settings-title = Settings
settings-language = Language
settings-language-auto = Automatic (system: { $lang })
settings-language-chain = Fallback order: { $chain }
settings-language-external = Additional translation files: { $path }
settings-missing-keys = { $count ->
    [0] No missing translations.
    [one] One missing translation (English is displayed).
   *[other] { $count } missing translations (English is displayed).
}
settings-appearance = Appearance
settings-theme = Theme
theme-system = Follow the system
theme-light = Light
theme-dark = Dark
settings-ui-scale = Interface zoom
settings-editor = Code Editor
settings-editor-font-size = Font size
settings-tab-width = Indentation width
settings-format-on-save = Format with rustfmt on save
settings-check-on-save = Check the code (cargo check) on save
settings-lsp = Use rust-analyzer (live diagnostics, completion, hover, go to definition with F12, rename with F2)
settings-lsp-missing = rust-analyzer is not installed: rustup component add rust-analyzer
settings-toolchain = Rust Toolchain
settings-cargo-path = Path to cargo
settings-cargo-path-hint = Leave empty for automatic detection.
settings-cargo-detected = Detected: { $path }
settings-cargo-not-found = Cargo was not found. Install Rust from https://rustup.rs
settings-designer = Designer
settings-grid-size = Grid size
settings-snap-to-grid = Snap to grid
settings-show-grid = Show grid
settings-config-file = Settings file: { $path }

## Status bar

status-ready = Ready
status-errors = { $count ->
    [one] { $count } error
   *[other] { $count } errors
}
status-warnings = { $count ->
    [one] { $count } warning
   *[other] { $count } warnings
}
status-running = Running: { $command }
status-build-succeeded = Finished successfully.
status-build-failed = Failed (code { $code }).
status-process-stopped = Process stopped.
status-saved = Saved: { $file }
status-project-opened = Project opened: { $name }
status-project-created = Project created: { $name }
status-code-generated = Code generated ({ $count } files updated).
status-language-changed = Language changed.
status-background-check = Checking the code…
status-release = Release
status-debug-build = Debug

## Messages and errors

error-title = Error
error-open-project = Unable to open the project: { $error }
error-create-project = Unable to create the project: { $error }
error-save = Unable to save: { $error }
error-cargo-not-found = Cargo cannot be found. Check Tools → Settings → Rust Toolchain.
error-codegen = Code generation failed: { $error }
error-already-running = A command is already running. Stop it first.
error-no-project = Open or create a project first.
error-read-file = Unable to read the file: { $error }

trust-title = Untrusted project
trust-message = This project was not created by Visual Rust Studio. Building it can run code (build scripts, procedural macros, dependencies). Only continue if you trust its origin.
    { $path }

unsaved-title = Unsaved changes
unsaved-message = Some files have been modified. Save them before closing?

about-title = About Visual Rust Studio
about-version = Version { $version }
about-description = An open-source visual development environment for Rust.
about-license = Free software under the GNU AGPL-3.0 license. The code of your applications belongs to you.
about-trademarks = Microsoft, Visual Studio, Visual Basic, .NET and Windows Forms are trademarks of Microsoft Corporation. Rust is a trademark of the Rust Foundation. Visual Rust Studio is an independent project, not affiliated with or endorsed by Microsoft or the Rust Foundation.

## Output and diagnostics

output-empty = No output yet. Use Build → Build (Ctrl+Shift+B) or Run (F5).
errors-empty = No diagnostics. Run Build → Check to analyze the code.
diag-level-error = Error
diag-level-warning = Warning
diag-level-note = Note
diag-level-help = Help
diag-explanation = Explanation
diag-original = Original compiler message
diag-open-location = Go to location
diag-suggestion = Suggested fix: { $replacement }
filter-errors = Errors
filter-warnings = Warnings

## Code editor

editor-find-placeholder = Find
editor-replace-placeholder = Replace with
editor-matches = { $total ->
    [0] No results
    [one] 1 result
   *[other] { $total } results
}
editor-line-col = Line { $line }, Col { $col }
editor-modified = Modified
editor-generated-warning = This file is generated by Visual Rust Studio and will be overwritten. Write your code in { $file }.

## Comments inserted in generated code

codegen-handler-doc = Handles the { $event } event of `{ $control }`.
codegen-todo = TODO: write your code here.
codegen-user-file-doc = Code of window { $form }. This file belongs to you: Visual Rust Studio only adds new event handlers and never modifies existing code.

cmd-set-startup-form = Set as Startup Window
menu-close-strategy = When the Main Window Closes
close-strategy-main = Close the application
close-strategy-last = Hide it; close the application with the last window

## Debugging / Débogage

cmd-run-without-debugging = Run Without Debugging
cmd-debug-continue = Continue
cmd-debug-pause = Pause
cmd-debug-stop = Stop Debugging
cmd-step-over = Step Over
cmd-step-into = Step Into
cmd-step-out = Step Out
cmd-toggle-breakpoint = Toggle Breakpoint
cmd-clear-breakpoints = Delete All Breakpoints
panel-debug = Debug
debug-idle = No debugging session. Set breakpoints by clicking in the margin (or F9), then press F5.
debug-call-stack = Call Stack
debug-threads = Threads
debug-locals = Locals
debug-watch = Watch
debug-watch-add = Expression to watch
debug-running = Running…
debug-paused = Paused.
debug-panic = Stopped on a Rust panic: see the call stack.
debug-stopped = Debugging stopped.
debug-started = Debugging with { $adapter }.
debug-exited = The program exited (code { $code }).
debug-no-adapter = No debugger found. Install GDB 14 or newer, lldb-dap or CodeLLDB, or set its path in Tools → Settings → Rust Toolchain.
debug-no-binary = The program to debug cannot be found.
settings-debugger-path = Path to the debugger
settings-debugger-detected = Debugger: { $adapter }
cmd-add-resource = Add an Image Resource…
cmd-rename-form = Rename Window…
rename-form-hint = The files and the Rust type are renamed; your code is updated (a .bak copy is kept).
codegen-data-doc = Variables of this window (like the variables declared in a VB.NET form). Add your fields here and use them with `self.data`.
