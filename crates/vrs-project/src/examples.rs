//! Projets d'exemple, classés par niveau.
//!
//! Chaque exemple est un vrai projet VRS (fenêtres conçues + code Rust écrit à
//! la main), créé dans le dossier choisi par l'utilisateur. Les textes affichés
//! et les commentaires sont traduits dans la langue de l'IDE (`tr`).

use std::path::Path;

use vrs_codegen::Comments;
use vrs_components::{ComponentKind, EventKind, FormDocument};

use crate::{Project, ProjectError, ProjectKind, create_project};

/// Niveau d'un exemple.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
}

impl Level {
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::Beginner => "examples-beginner",
            Self::Intermediate => "examples-intermediate",
            Self::Advanced => "examples-advanced",
        }
    }
}

/// Exemple disponible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Example {
    /// Identifiant (clés de traduction `example-<id>-title` et `-desc`).
    pub id: &'static str,
    pub level: Level,
    pub kind: ProjectKind,
    /// Nom du projet créé.
    pub project_name: &'static str,
}

impl Example {
    pub fn title_key(&self) -> String {
        format!("example-{}-title", self.id)
    }
    pub fn desc_key(&self) -> String {
        format!("example-{}-desc", self.id)
    }
}

/// Catalogue des exemples.
pub const EXAMPLES: &[Example] = &[
    ex(
        "hello-terminal",
        Level::Beginner,
        ProjectKind::Console,
        "HelloWorldTerminal",
    ),
    ex(
        "hello-window",
        Level::Beginner,
        ProjectKind::Gui,
        "HelloWorldWindow",
    ),
    ex("counter", Level::Beginner, ProjectKind::Gui, "ClickCounter"),
    ex(
        "temperature",
        Level::Beginner,
        ProjectKind::Gui,
        "TemperatureConverter",
    ),
    ex(
        "calculator",
        Level::Beginner,
        ProjectKind::Gui,
        "Calculator",
    ),
    ex("color", Level::Beginner, ProjectKind::Gui, "ColorPicker"),
    ex("quiz", Level::Beginner, ProjectKind::Gui, "RustQuiz"),
    ex("todo", Level::Intermediate, ProjectKind::Gui, "TodoList"),
    ex(
        "guess",
        Level::Intermediate,
        ProjectKind::Gui,
        "GuessTheNumber",
    ),
    ex(
        "two-windows",
        Level::Intermediate,
        ProjectKind::Gui,
        "TwoWindows",
    ),
    ex(
        "stopwatch",
        Level::Intermediate,
        ProjectKind::Gui,
        "Stopwatch",
    ),
    ex(
        "address-book",
        Level::Intermediate,
        ProjectKind::Gui,
        "AddressBook",
    ),
    ex(
        "tic-tac-toe",
        Level::Advanced,
        ProjectKind::Gui,
        "TicTacToe",
    ),
    ex("notepad", Level::Advanced, ProjectKind::Gui, "Notepad"),
];

const fn ex(
    id: &'static str,
    level: Level,
    kind: ProjectKind,
    project_name: &'static str,
) -> Example {
    Example {
        id,
        level,
        kind,
        project_name,
    }
}

/// Littéral de chaîne Rust (guillemets et caractères spéciaux échappés).
fn lit(s: &str) -> String {
    format!("{s:?}")
}

/// Échappe les accolades d'un texte traduit inséré dans une chaîne `format!`.
fn esc(s: &str) -> String {
    s.replace('{', "{{").replace('}', "}}")
}

/// En-tête d'un fichier de code de fenêtre.
fn header(tr: &dyn Fn(&str) -> String, form: &str, id: &str) -> String {
    let mut s = String::new();
    for l in tr(&format!("example-{id}-desc")).lines() {
        s.push_str(&format!("//! {l}\n"));
    }
    s.push_str(&format!(
        "#![allow(unused_imports, unused_variables)]\n\nuse super::{form};\nuse crate::vrs::{{AppContext, egui}};\n\n"
    ));
    s
}

/// Crée l'exemple `id` dans `parent` (nom de dossier libre choisi automatiquement).
pub fn create_example(
    id: &str,
    parent: &Path,
    tr: &dyn Fn(&str) -> String,
    comments: &dyn Comments,
) -> Result<Project, ProjectError> {
    let ex = EXAMPLES
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| ProjectError::InvalidName(id.to_owned()))?;
    let name = (1..)
        .map(|i| {
            if i == 1 {
                ex.project_name.to_owned()
            } else {
                format!("{}{i}", ex.project_name)
            }
        })
        .find(|n| !parent.join(n).exists())
        .expect("itérateur infini");
    let mut project = create_project(parent, &name, ex.kind, comments)?;
    let root = project.root.clone();
    let write = |rel: &str, text: String| -> Result<(), ProjectError> {
        vrs_core::fs::atomic_write(&root.join(rel), text)?;
        Ok(())
    };
    let save_form = |p: &Project, f: &FormDocument| -> Result<(), ProjectError> {
        f.save(&p.form_path(&f.name))
            .map_err(|e| ProjectError::Form(e.to_string()))
    };
    let click = |f: &mut FormDocument, name: &str| {
        if let Some(c) = f.control_mut(name) {
            c.events
                .insert(EventKind::Click, EventKind::Click.default_handler(name));
        }
    };

    match id {
        "hello-terminal" => {
            write(
                "src/main.rs",
                format!(
                    "//! {}\n\nfn main() {{\n    // {}\n    println!({});\n}}\n",
                    tr("example-hello-terminal-desc"),
                    tr("ex-comment-println"),
                    lit(&esc(&tr("ex-hello"))),
                ),
            )?;
        }
        "hello-window" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-hello-window-title");
            f.width = 420.0;
            f.height = 170.0;
            let l = f.add_control(ComponentKind::Label, 24.0, 24.0);
            let b = f.add_control(ComponentKind::Button, 24.0, 96.0);
            {
                let c = f.control_mut(&l).unwrap();
                c.text = tr("ex-hello");
                c.width = 360.0;
                c.height = 40.0;
                c.font_size = Some(24.0);
            }
            {
                let c = f.control_mut(&b).unwrap();
                c.text = tr("ex-close");
                c.width = 120.0;
                c.height = 32.0;
            }
            click(&mut f, &b);
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{}{data}impl Form1 {{\n    /// {doc}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        // {c}\n        app.exit();\n    }}\n}}\n",
                    header(tr, "Form1", id),
                    data = "#[derive(Default)]\npub struct Form1Data {}\n\n",
                    doc = comments.handler_doc("Click", "button1"),
                    c = tr("ex-comment-exit"),
                ),
            )?;
        }
        "counter" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-counter-title");
            f.width = 360.0;
            f.height = 200.0;
            let l = f.add_control(ComponentKind::Label, 24.0, 24.0);
            let plus = f.add_control(ComponentKind::Button, 24.0, 110.0);
            let reset = f.add_control(ComponentKind::Button, 152.0, 110.0);
            {
                let c = f.control_mut(&l).unwrap();
                c.text = "0".into();
                c.width = 300.0;
                c.height = 60.0;
                c.font_size = Some(40.0);
            }
            f.control_mut(&plus).unwrap().text = "+1".into();
            {
                let c = f.control_mut(&reset).unwrap();
                c.text = tr("ex-reset");
                c.width = 150.0;
            }
            click(&mut f, &plus);
            click(&mut f, &reset);
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}/// {vars}\n#[derive(Default)]\npub struct Form1Data {{\n    /// {count}\n    pub count: u32,\n}}\n\nimpl Form1 {{\n    /// {d1}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        // {c1}\n        self.data.count += 1;\n        self.label1.text = self.data.count.to_string();\n    }}\n\n    /// {d2}\n    pub fn button2_click(&mut self, app: &mut AppContext) {{\n        self.data.count = 0;\n        self.label1.text = String::from(\"0\");\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    vars = tr("codegen-data-doc"),
                    count = tr("ex-comment-count"),
                    d1 = comments.handler_doc("Click", "button1"),
                    c1 = tr("ex-comment-self-data"),
                    d2 = comments.handler_doc("Click", "button2"),
                ),
            )?;
        }
        "temperature" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-temperature-title");
            f.width = 420.0;
            f.height = 200.0;
            let l1 = f.add_control(ComponentKind::Label, 24.0, 24.0);
            let t = f.add_control(ComponentKind::TextBox, 24.0, 52.0);
            let b = f.add_control(ComponentKind::Button, 200.0, 50.0);
            let l2 = f.add_control(ComponentKind::Label, 24.0, 110.0);
            {
                let c = f.control_mut(&l1).unwrap();
                c.text = tr("ex-celsius");
                c.width = 300.0;
            }
            f.control_mut(&t).unwrap().width = 160.0;
            {
                let c = f.control_mut(&b).unwrap();
                c.text = tr("ex-convert");
                c.width = 120.0;
            }
            {
                let c = f.control_mut(&l2).unwrap();
                c.text = String::new();
                c.width = 360.0;
                c.height = 30.0;
                c.font_size = Some(18.0);
            }
            click(&mut f, &b);
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form1Data {{}}\n\nimpl Form1 {{\n    /// {d}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        // {c1}\n        let input = self.text_box1.text.trim().replace(',', \".\");\n        // {c2}\n        match input.parse::<f64>() {{\n            Ok(celsius) => {{\n                let fahrenheit = celsius * 9.0 / 5.0 + 32.0;\n                self.label2.text = format!(\"{{celsius}} °C = {{fahrenheit:.1}} °F\");\n            }}\n            Err(_) => {{\n                self.label2.text = String::from({err});\n            }}\n        }}\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    d = comments.handler_doc("Click", "button1"),
                    c1 = tr("ex-comment-read-text"),
                    c2 = tr("ex-comment-parse"),
                    err = lit(&tr("ex-not-a-number")),
                ),
            )?;
        }
        "todo" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-todo-title");
            f.width = 420.0;
            f.height = 340.0;
            let t = f.add_control(ComponentKind::TextBox, 16.0, 16.0);
            let add = f.add_control(ComponentKind::Button, 280.0, 14.0);
            let list = f.add_control(ComponentKind::ListBox, 16.0, 56.0);
            let remove = f.add_control(ComponentKind::Button, 280.0, 56.0);
            let count = f.add_control(ComponentKind::Label, 16.0, 300.0);
            f.control_mut(&t).unwrap().width = 250.0;
            {
                let c = f.control_mut(&add).unwrap();
                c.text = tr("ex-add");
                c.width = 120.0;
            }
            {
                let c = f.control_mut(&list).unwrap();
                c.width = 250.0;
                c.height = 230.0;
                c.anchor.bottom = true;
            }
            {
                let c = f.control_mut(&remove).unwrap();
                c.text = tr("ex-remove");
                c.width = 120.0;
            }
            {
                let c = f.control_mut(&count).unwrap();
                c.text = tr("ex-no-task");
                c.width = 250.0;
                c.anchor = vrs_components::Anchor {
                    left: true,
                    top: false,
                    right: false,
                    bottom: true,
                };
            }
            click(&mut f, &add);
            click(&mut f, &remove);
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form1Data {{}}\n\nimpl Form1 {{\n    /// {d1}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        let task = self.text_box1.text.trim().to_owned();\n        // {c1}\n        if task.is_empty() {{\n            return;\n        }}\n        self.list_box1.items.push(task);\n        self.text_box1.text.clear();\n        self.update_count();\n    }}\n\n    /// {d2}\n    pub fn button2_click(&mut self, app: &mut AppContext) {{\n        // {c2}\n        if let Some(i) = self.list_box1.selected {{\n            self.list_box1.items.remove(i);\n            self.list_box1.selected = None;\n            self.update_count();\n        }}\n    }}\n\n    /// {c3}\n    fn update_count(&mut self) {{\n        self.label1.text = format!({tasks}, self.list_box1.items.len());\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    d1 = comments.handler_doc("Click", "button1"),
                    c1 = tr("ex-comment-empty"),
                    d2 = comments.handler_doc("Click", "button2"),
                    c2 = tr("ex-comment-selected"),
                    c3 = tr("ex-comment-helper"),
                    tasks = lit(&format!("{{}} {}", esc(&tr("ex-tasks")))),
                ),
            )?;
        }
        "guess" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-guess-title");
            f.width = 420.0;
            f.height = 220.0;
            let l1 = f.add_control(ComponentKind::Label, 24.0, 24.0);
            let t = f.add_control(ComponentKind::TextBox, 24.0, 60.0);
            let b = f.add_control(ComponentKind::Button, 200.0, 58.0);
            let l2 = f.add_control(ComponentKind::Label, 24.0, 110.0);
            let l3 = f.add_control(ComponentKind::Label, 24.0, 150.0);
            {
                let c = f.control_mut(&l1).unwrap();
                c.text = tr("ex-guess-rules");
                c.width = 380.0;
            }
            f.control_mut(&t).unwrap().width = 160.0;
            {
                let c = f.control_mut(&b).unwrap();
                c.text = tr("ex-try");
                c.width = 120.0;
            }
            {
                let c = f.control_mut(&l2).unwrap();
                c.text = String::new();
                c.width = 380.0;
                c.font_size = Some(18.0);
            }
            {
                let c = f.control_mut(&l3).unwrap();
                c.text = String::new();
                c.width = 380.0;
            }
            click(&mut f, &b);
            f.events
                .insert(EventKind::Load, EventKind::Load.default_handler("form1"));
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form1Data {{\n    /// {cs}\n    pub secret: u32,\n    /// {ct}\n    pub tries: u32,\n}}\n\n/// {cr}\nfn random_1_to_100() -> u32 {{\n    let nanos = std::time::SystemTime::now()\n        .duration_since(std::time::UNIX_EPOCH)\n        .map(|d| d.subsec_nanos())\n        .unwrap_or(42);\n    nanos % 100 + 1\n}}\n\nimpl Form1 {{\n    /// {dl}\n    pub fn form1_load(&mut self, app: &mut AppContext) {{\n        self.data.secret = random_1_to_100();\n    }}\n\n    /// {d}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        let Ok(guess) = self.text_box1.text.trim().parse::<u32>() else {{\n            self.label2.text = String::from({nan});\n            return;\n        }};\n        self.data.tries += 1;\n        self.label3.text = format!({tries}, self.data.tries);\n        // {cm}\n        match guess.cmp(&self.data.secret) {{\n            std::cmp::Ordering::Less => self.label2.text = String::from({more}),\n            std::cmp::Ordering::Greater => self.label2.text = String::from({less}),\n            std::cmp::Ordering::Equal => {{\n                app.message_box(format!({won}, self.data.secret, self.data.tries));\n                // {cn}\n                self.data.secret = random_1_to_100();\n                self.data.tries = 0;\n                self.label2.text.clear();\n                self.label3.text.clear();\n            }}\n        }}\n        self.text_box1.text.clear();\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    cs = tr("ex-comment-secret"),
                    ct = tr("ex-comment-tries"),
                    cr = tr("ex-comment-random"),
                    dl = comments.handler_doc("Load", "Form1"),
                    d = comments.handler_doc("Click", "button1"),
                    nan = lit(&tr("ex-not-a-number")),
                    tries = lit(&format!("{} : {{}}", esc(&tr("ex-tries")))),
                    cm = tr("ex-comment-match"),
                    more = lit(&tr("ex-higher")),
                    less = lit(&tr("ex-lower")),
                    won = lit(&format!(
                        "{} {{}} ! {} : {{}}",
                        esc(&tr("ex-won")),
                        esc(&tr("ex-tries"))
                    )),
                    cn = tr("ex-comment-new-game"),
                ),
            )?;
        }
        "two-windows" => {
            let mut f1 = FormDocument::new("Form1");
            f1.title = tr("example-two-windows-title");
            f1.width = 420.0;
            f1.height = 180.0;
            let l = f1.add_control(ComponentKind::Label, 24.0, 24.0);
            let b = f1.add_control(ComponentKind::Button, 24.0, 90.0);
            {
                let c = f1.control_mut(&l).unwrap();
                c.text = tr("ex-hello");
                c.width = 360.0;
                c.height = 36.0;
                c.font_size = Some(20.0);
            }
            {
                let c = f1.control_mut(&b).unwrap();
                c.text = tr("ex-ask-name");
                c.width = 220.0;
                c.height = 32.0;
            }
            click(&mut f1, &b);
            save_form(&project, &f1)?;
            project.add_form("Form2", comments)?;
            let mut f2 = FormDocument::new("Form2");
            f2.title = tr("ex-your-name");
            f2.width = 320.0;
            f2.height = 150.0;
            let l2 = f2.add_control(ComponentKind::Label, 16.0, 16.0);
            let t2 = f2.add_control(ComponentKind::TextBox, 16.0, 44.0);
            let ok = f2.add_control(ComponentKind::Button, 16.0, 96.0);
            let cancel = f2.add_control(ComponentKind::Button, 124.0, 96.0);
            f2.control_mut(&l2).unwrap().text = tr("ex-your-name");
            f2.control_mut(&t2).unwrap().width = 280.0;
            f2.control_mut(&ok).unwrap().text = "OK".into();
            f2.control_mut(&cancel).unwrap().text = tr("ex-cancel");
            click(&mut f2, &ok);
            click(&mut f2, &cancel);
            save_form(&project, &f2)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form1Data {{}}\n\nimpl Form1 {{\n    /// {d}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        // {c1}\n        app.show_dialog_then(super::Form2::default(), |me: &mut Form1, form2: super::Form2, _app| {{\n            // {c2}\n            if form2.data.accepted && !form2.text_box1.text.trim().is_empty() {{\n                me.label1.text = format!({greet}, form2.text_box1.text.trim());\n            }}\n        }});\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    d = comments.handler_doc("Click", "button1"),
                    c1 = tr("ex-comment-dialog"),
                    c2 = tr("ex-comment-dialog-result"),
                    greet = lit(&format!("{} {{}} !", esc(&tr("ex-hello-name")))),
                ),
            )?;
            write(
                "src/forms/form2.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form2Data {{\n    /// {ca}\n    pub accepted: bool,\n}}\n\nimpl Form2 {{\n    /// {d1}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        self.data.accepted = true;\n        app.close();\n    }}\n\n    /// {d2}\n    pub fn button2_click(&mut self, app: &mut AppContext) {{\n        app.close();\n    }}\n}}\n",
                    h = header(tr, "Form2", id),
                    ca = tr("ex-comment-accepted"),
                    d1 = comments.handler_doc("Click", "button1"),
                    d2 = comments.handler_doc("Click", "button2"),
                ),
            )?;
        }
        "calculator" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-calculator-title");
            f.width = 460.0;
            f.height = 160.0;
            let a = f.add_control(ComponentKind::TextBox, 16.0, 24.0);
            let op = f.add_control(ComponentKind::ComboBox, 128.0, 24.0);
            let b = f.add_control(ComponentKind::TextBox, 200.0, 24.0);
            let eq = f.add_control(ComponentKind::Button, 312.0, 22.0);
            let res = f.add_control(ComponentKind::Label, 16.0, 80.0);
            f.control_mut(&a).unwrap().width = 100.0;
            f.control_mut(&b).unwrap().width = 100.0;
            {
                let c = f.control_mut(&op).unwrap();
                c.width = 64.0;
                c.items = vec!["+".into(), "−".into(), "×".into(), "÷".into()];
                c.selected = Some(0);
            }
            {
                let c = f.control_mut(&eq).unwrap();
                c.text = "=".into();
                c.width = 60.0;
            }
            {
                let c = f.control_mut(&res).unwrap();
                c.text = String::new();
                c.width = 420.0;
                c.height = 32.0;
                c.font_size = Some(22.0);
            }
            click(&mut f, &eq);
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form1Data {{}}\n\n/// {cp}\nfn number(text: &str) -> Option<f64> {{\n    text.trim().replace(',', \".\").parse().ok()\n}}\n\nimpl Form1 {{\n    /// {d}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        let (Some(a), Some(b)) = (number(&self.text_box1.text), number(&self.text_box2.text)) else {{\n            self.label1.text = String::from({nan});\n            return;\n        }};\n        // {cm}\n        let result = match self.combo_box1.selected {{\n            Some(0) => a + b,\n            Some(1) => a - b,\n            Some(2) => a * b,\n            Some(3) if b == 0.0 => {{\n                app.message_box({divz});\n                return;\n            }}\n            Some(3) => a / b,\n            _ => return,\n        }};\n        self.label1.text = format!(\"= {{result}}\");\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    cp = tr("ex-comment-parse-option"),
                    d = comments.handler_doc("Click", "button1"),
                    nan = lit(&tr("ex-not-a-number")),
                    cm = tr("ex-comment-operation"),
                    divz = lit(&tr("ex-div-zero")),
                ),
            )?;
        }
        "color" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-color-title");
            f.width = 460.0;
            f.height = 230.0;
            let names: Vec<String> = ["ex-red", "ex-green", "ex-blue"]
                .iter()
                .enumerate()
                .map(|(i, key)| {
                    let n = f.add_control(ComponentKind::Slider, 16.0, 20.0 + i as f32 * 36.0);
                    let c = f.control_mut(&n).unwrap();
                    c.text = tr(key);
                    c.width = 260.0;
                    c.minimum = 0.0;
                    c.maximum = 255.0;
                    c.value = 128.0;
                    c.events.insert(
                        EventKind::ValueChanged,
                        EventKind::ValueChanged.default_handler(&n),
                    );
                    n
                })
                .collect();
            let p = f.add_control(ComponentKind::Panel, 300.0, 20.0);
            {
                let c = f.control_mut(&p).unwrap();
                c.width = 140.0;
                c.height = 100.0;
                c.back_color = Some([128, 128, 128, 255]);
            }
            let l = f.add_control(ComponentKind::Label, 16.0, 150.0);
            {
                let c = f.control_mut(&l).unwrap();
                c.text = "#808080".into();
                c.width = 420.0;
                c.font_size = Some(18.0);
            }
            f.events
                .insert(EventKind::Load, EventKind::Load.default_handler("form1"));
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            let mut handlers = String::new();
            for n in &names {
                handlers.push_str(&format!(
                    "    /// {}\n    pub fn {n}_value_changed(&mut self, app: &mut AppContext) {{\n        self.update_color();\n    }}\n\n",
                    comments.handler_doc("ValueChanged", n)
                ));
            }
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form1Data {{}}\n\nimpl Form1 {{\n    /// {dl}\n    pub fn form1_load(&mut self, app: &mut AppContext) {{\n        self.update_color();\n    }}\n\n{handlers}    /// {cu}\n    fn update_color(&mut self) {{\n        // {cc}\n        let (r, g, b) = (\n            self.slider1.value as u8,\n            self.slider2.value as u8,\n            self.slider3.value as u8,\n        );\n        self.panel1.back_color = Some(crate::vrs::Color::from_rgb(r, g, b));\n        self.label1.text = format!(\"#{{r:02X}}{{g:02X}}{{b:02X}}  ({{r}}, {{g}}, {{b}})\");\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    dl = comments.handler_doc("Load", "Form1"),
                    cu = tr("ex-comment-update-color"),
                    cc = tr("ex-comment-as-u8"),
                ),
            )?;
        }
        "quiz" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-quiz-title");
            f.width = 480.0;
            f.height = 260.0;
            let q = f.add_control(ComponentKind::Label, 16.0, 16.0);
            {
                let c = f.control_mut(&q).unwrap();
                c.width = 440.0;
                c.height = 30.0;
                c.font_size = Some(17.0);
            }
            for i in 0..3 {
                let r = f.add_control(ComponentKind::RadioButton, 24.0, 60.0 + i as f32 * 30.0);
                let c = f.control_mut(&r).unwrap();
                c.width = 300.0;
                c.group = "answers".into();
            }
            let b = f.add_control(ComponentKind::Button, 16.0, 160.0);
            {
                let c = f.control_mut(&b).unwrap();
                c.text = tr("ex-validate");
                c.width = 120.0;
            }
            let fb = f.add_control(ComponentKind::Label, 150.0, 162.0);
            {
                let c = f.control_mut(&fb).unwrap();
                c.text = String::new();
                c.width = 310.0;
            }
            click(&mut f, &b);
            f.events
                .insert(EventKind::Load, EventKind::Load.default_handler("form1"));
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            let question = |n: usize, good: usize| {
                format!(
                    "    ({}, [{}, {}, {}], {good}),\n",
                    lit(&tr(&format!("ex-quiz-q{n}"))),
                    lit(&tr(&format!("ex-quiz-q{n}-a1"))),
                    lit(&tr(&format!("ex-quiz-q{n}-a2"))),
                    lit(&tr(&format!("ex-quiz-q{n}-a3"))),
                )
            };
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}/// {cq}\nconst QUESTIONS: [(&str, [&str; 3], usize); 3] = [\n{q1}{q2}{q3}];\n\n#[derive(Default)]\npub struct Form1Data {{\n    pub index: usize,\n    pub score: u32,\n}}\n\nimpl Form1 {{\n    /// {dl}\n    pub fn form1_load(&mut self, app: &mut AppContext) {{\n        self.show_question();\n    }}\n\n    /// {d}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        // {cc}\n        let answers = [\n            self.radio_button1.checked,\n            self.radio_button2.checked,\n            self.radio_button3.checked,\n        ];\n        let Some(chosen) = answers.iter().position(|checked| *checked) else {{\n            self.label2.text = String::from({choose});\n            return;\n        }};\n        let (_, _, good) = QUESTIONS[self.data.index];\n        if chosen == good {{\n            self.data.score += 1;\n            self.label2.text = String::from({right});\n        }} else {{\n            self.label2.text = String::from({wrong});\n        }}\n        self.data.index += 1;\n        if self.data.index == QUESTIONS.len() {{\n            app.message_box(format!({score}, self.data.score, QUESTIONS.len()));\n            self.data.index = 0;\n            self.data.score = 0;\n            self.label2.text.clear();\n        }}\n        self.show_question();\n    }}\n\n    /// {cs}\n    fn show_question(&mut self) {{\n        let (question, answers, _) = QUESTIONS[self.data.index];\n        self.label1.text = question.to_owned();\n        for (radio, answer) in [\n            &mut self.radio_button1,\n            &mut self.radio_button2,\n            &mut self.radio_button3,\n        ]\n        .into_iter()\n        .zip(answers)\n        {{\n            radio.text = answer.to_owned();\n            radio.checked = false;\n        }}\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    cq = tr("ex-comment-questions"),
                    q1 = question(1, 0),
                    q2 = question(2, 1),
                    q3 = question(3, 2),
                    dl = comments.handler_doc("Load", "Form1"),
                    d = comments.handler_doc("Click", "button1"),
                    cc = tr("ex-comment-which-radio"),
                    choose = lit(&tr("ex-choose")),
                    right = lit(&tr("ex-correct")),
                    wrong = lit(&tr("ex-wrong")),
                    score = lit(&format!("{} {{}} / {{}}", esc(&tr("ex-quiz-score")))),
                    cs = tr("ex-comment-show-question"),
                ),
            )?;
        }
        "stopwatch" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-stopwatch-title");
            f.width = 400.0;
            f.height = 210.0;
            let l = f.add_control(ComponentKind::Label, 24.0, 16.0);
            {
                let c = f.control_mut(&l).unwrap();
                c.text = "00:00.0".into();
                c.width = 340.0;
                c.height = 60.0;
                c.font_size = Some(44.0);
            }
            let pb = f.add_control(ComponentKind::ProgressBar, 24.0, 86.0);
            {
                let c = f.control_mut(&pb).unwrap();
                c.width = 350.0;
                c.maximum = 60.0;
            }
            for (i, key) in ["ex-start", "ex-stop", "ex-reset"].iter().enumerate() {
                let b = f.add_control(ComponentKind::Button, 24.0 + i as f32 * 120.0, 130.0);
                let c = f.control_mut(&b).unwrap();
                c.text = tr(key);
                c.width = 110.0;
                click(&mut f, &b);
            }
            let t = f.add_control(ComponentKind::Timer, 300.0, 175.0);
            {
                let c = f.control_mut(&t).unwrap();
                c.interval = 100;
                c.events
                    .insert(EventKind::Tick, EventKind::Tick.default_handler(&t));
            }
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form1Data {{\n    /// {ct}\n    pub tenths: u32,\n}}\n\nimpl Form1 {{\n    /// {dt}\n    pub fn timer1_tick(&mut self, app: &mut AppContext) {{\n        self.data.tenths += 1;\n        self.show_time();\n    }}\n\n    /// {d1}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        // {cs}\n        self.timer1.enabled = true;\n    }}\n\n    /// {d2}\n    pub fn button2_click(&mut self, app: &mut AppContext) {{\n        self.timer1.enabled = false;\n    }}\n\n    /// {d3}\n    pub fn button3_click(&mut self, app: &mut AppContext) {{\n        self.data.tenths = 0;\n        self.show_time();\n    }}\n\n    /// {cf}\n    fn show_time(&mut self) {{\n        let minutes = self.data.tenths / 600;\n        let seconds = (self.data.tenths / 10) % 60;\n        let tenths = self.data.tenths % 10;\n        self.label1.text = format!(\"{{minutes:02}}:{{seconds:02}}.{{tenths}}\");\n        self.progress_bar1.value = seconds as f32 + tenths as f32 / 10.0;\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    ct = tr("ex-comment-tenths"),
                    dt = comments.handler_doc("Tick", "timer1"),
                    d1 = comments.handler_doc("Click", "button1"),
                    cs = tr("ex-comment-timer-enabled"),
                    d2 = comments.handler_doc("Click", "button2"),
                    d3 = comments.handler_doc("Click", "button3"),
                    cf = tr("ex-comment-format-time"),
                ),
            )?;
        }
        "address-book" => {
            let mut f1 = FormDocument::new("Form1");
            f1.title = tr("example-address-book-title");
            f1.width = 440.0;
            f1.height = 300.0;
            let list = f1.add_control(ComponentKind::ListBox, 16.0, 16.0);
            {
                let c = f1.control_mut(&list).unwrap();
                c.width = 280.0;
                c.height = 260.0;
                c.anchor.bottom = true;
                c.anchor.right = true;
            }
            for (i, key) in ["ex-add", "ex-remove"].iter().enumerate() {
                let b = f1.add_control(ComponentKind::Button, 310.0, 16.0 + i as f32 * 40.0);
                let c = f1.control_mut(&b).unwrap();
                c.text = tr(key);
                c.width = 110.0;
                c.anchor = vrs_components::Anchor {
                    left: false,
                    top: true,
                    right: true,
                    bottom: false,
                };
                click(&mut f1, &b);
            }
            save_form(&project, &f1)?;
            project.add_form("Form2", comments)?;
            let mut f2 = FormDocument::new("Form2");
            f2.title = tr("ex-contact");
            f2.width = 320.0;
            f2.height = 190.0;
            for (i, key) in ["ex-name", "ex-phone"].iter().enumerate() {
                let l = f2.add_control(ComponentKind::Label, 16.0, 16.0 + i as f32 * 56.0);
                f2.control_mut(&l).unwrap().text = tr(key);
                let t = f2.add_control(ComponentKind::TextBox, 16.0, 40.0 + i as f32 * 56.0);
                f2.control_mut(&t).unwrap().width = 280.0;
            }
            for (i, key) in ["ex-ok", "ex-cancel"].iter().enumerate() {
                let b = f2.add_control(ComponentKind::Button, 16.0 + i as f32 * 110.0, 140.0);
                f2.control_mut(&b).unwrap().text = tr(key);
                click(&mut f2, &b);
            }
            save_form(&project, &f2)?;
            project.sync_code(&[], comments)?;
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form1Data {{}}\n\nimpl Form1 {{\n    /// {d1}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        // {c1}\n        app.show_dialog_then(super::Form2::default(), |me: &mut Form1, contact: super::Form2, _app| {{\n            let name = contact.text_box1.text.trim();\n            let phone = contact.text_box2.text.trim();\n            if contact.data.accepted && !name.is_empty() {{\n                me.list_box1.items.push(format!(\"{{name}} — {{phone}}\"));\n                me.list_box1.items.sort();\n            }}\n        }});\n    }}\n\n    /// {d2}\n    pub fn button2_click(&mut self, app: &mut AppContext) {{\n        if let Some(i) = self.list_box1.selected.take() {{\n            self.list_box1.items.remove(i);\n        }}\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    d1 = comments.handler_doc("Click", "button1"),
                    c1 = tr("ex-comment-dialog"),
                    d2 = comments.handler_doc("Click", "button2"),
                ),
            )?;
            write(
                "src/forms/form2.rs",
                format!(
                    "{h}#[derive(Default)]\npub struct Form2Data {{\n    /// {ca}\n    pub accepted: bool,\n}}\n\nimpl Form2 {{\n    /// {d1}\n    pub fn button1_click(&mut self, app: &mut AppContext) {{\n        self.data.accepted = true;\n        app.close();\n    }}\n\n    /// {d2}\n    pub fn button2_click(&mut self, app: &mut AppContext) {{\n        app.close();\n    }}\n}}\n",
                    h = header(tr, "Form2", id),
                    ca = tr("ex-comment-accepted"),
                    d1 = comments.handler_doc("Click", "button1"),
                    d2 = comments.handler_doc("Click", "button2"),
                ),
            )?;
        }
        "tic-tac-toe" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-tic-tac-toe-title");
            f.width = 300.0;
            f.height = 340.0;
            for i in 0..9 {
                let b = f.add_control(
                    ComponentKind::Button,
                    24.0 + (i % 3) as f32 * 84.0,
                    16.0 + (i / 3) as f32 * 84.0,
                );
                let c = f.control_mut(&b).unwrap();
                c.text = String::new();
                c.width = 76.0;
                c.height = 76.0;
                c.font_size = Some(32.0);
                click(&mut f, &b);
            }
            let l = f.add_control(ComponentKind::Label, 24.0, 272.0);
            {
                let c = f.control_mut(&l).unwrap();
                c.text = format!("{} X", tr("ex-ttt-turn"));
                c.width = 250.0;
            }
            let n = f.add_control(ComponentKind::Button, 24.0, 300.0);
            {
                let c = f.control_mut(&n).unwrap();
                c.text = tr("ex-new-game");
                c.width = 160.0;
            }
            click(&mut f, &n);
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            let mut cells = String::new();
            for i in 1..=9 {
                cells.push_str(&format!(
                    "    /// {}\n    pub fn button{i}_click(&mut self, app: &mut AppContext) {{\n        self.play({});\n    }}\n\n",
                    comments.handler_doc("Click", &format!("button{i}")),
                    i - 1
                ));
            }
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}use crate::vrs::ButtonState;\n\n/// {cl}\nconst LINES: [[usize; 3]; 8] = [\n    [0, 1, 2], [3, 4, 5], [6, 7, 8],\n    [0, 3, 6], [1, 4, 7], [2, 5, 8],\n    [0, 4, 8], [2, 4, 6],\n];\n\n#[derive(Default)]\npub struct Form1Data {{\n    /// {cb}\n    pub board: [Option<char>; 9],\n    pub player: Option<char>,\n    pub over: bool,\n}}\n\n/// {cw}\nfn winner(board: &[Option<char>; 9]) -> Option<char> {{\n    LINES.iter().find_map(|[a, b, c]| {{\n        (board[*a].is_some() && board[*a] == board[*b] && board[*b] == board[*c]).then(|| board[*a]).flatten()\n    }})\n}}\n\nimpl Form1 {{\n{cells}    /// {dn}\n    pub fn button10_click(&mut self, app: &mut AppContext) {{\n        self.data = Form1Data::default();\n        for i in 0..9 {{\n            self.cell(i).text.clear();\n        }}\n        self.label1.text = format!({turn}, 'X');\n    }}\n\n    /// {cc}\n    fn cell(&mut self, i: usize) -> &mut ButtonState {{\n        match i {{\n            0 => &mut self.button1,\n            1 => &mut self.button2,\n            2 => &mut self.button3,\n            3 => &mut self.button4,\n            4 => &mut self.button5,\n            5 => &mut self.button6,\n            6 => &mut self.button7,\n            7 => &mut self.button8,\n            _ => &mut self.button9,\n        }}\n    }}\n\n    /// {cp}\n    fn play(&mut self, i: usize) {{\n        if self.data.over || self.data.board[i].is_some() {{\n            return;\n        }}\n        let player = self.data.player.unwrap_or('X');\n        self.data.board[i] = Some(player);\n        self.cell(i).text = player.to_string();\n        if let Some(w) = winner(&self.data.board) {{\n            self.data.over = true;\n            self.label1.text = format!({wins}, w);\n        }} else if self.data.board.iter().all(Option::is_some) {{\n            self.data.over = true;\n            self.label1.text = String::from({draw});\n        }} else {{\n            let next = if player == 'X' {{ 'O' }} else {{ 'X' }};\n            self.data.player = Some(next);\n            self.label1.text = format!({turn}, next);\n        }}\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    cl = tr("ex-comment-lines"),
                    cb = tr("ex-comment-board"),
                    cw = tr("ex-comment-winner"),
                    dn = comments.handler_doc("Click", "button10"),
                    cc = tr("ex-comment-cell"),
                    cp = tr("ex-comment-play"),
                    turn = lit(&format!("{} {{}}", esc(&tr("ex-ttt-turn")))),
                    wins = lit(&format!("{{}} {}", esc(&tr("ex-ttt-wins")))),
                    draw = lit(&tr("ex-ttt-draw")),
                ),
            )?;
        }
        "notepad" => {
            let mut f = FormDocument::new("Form1");
            f.title = tr("example-notepad-title");
            f.width = 560.0;
            f.height = 400.0;
            let outline = format!(
                "{}\n  {}\n  {}\n  {}\n  {}\n",
                tr("ex-menu-file"),
                tr("ex-new"),
                tr("ex-open"),
                tr("ex-save"),
                tr("ex-quit")
            );
            f.menu = vrs_components::MenuItem::from_outline(&outline, &[], &[]);
            let items: Vec<String> = f.menu[0].children.iter().map(|m| m.name.clone()).collect();
            for (m, name) in f.menu[0].children.iter_mut().zip(&items) {
                m.click = Some(format!("{name}_click"));
            }
            let t = f.add_control(ComponentKind::TextBox, 8.0, 8.0);
            {
                let c = f.control_mut(&t).unwrap();
                c.multiline = true;
                c.width = 544.0;
                c.height = 330.0;
                c.anchor = vrs_components::Anchor {
                    left: true,
                    top: true,
                    right: true,
                    bottom: true,
                };
            }
            let l = f.add_control(ComponentKind::Label, 8.0, 346.0);
            {
                let c = f.control_mut(&l).unwrap();
                c.text = String::new();
                c.width = 540.0;
                c.anchor = vrs_components::Anchor {
                    left: true,
                    top: false,
                    right: true,
                    bottom: true,
                };
            }
            save_form(&project, &f)?;
            project.sync_code(&[], comments)?;
            let [new, open, save, quit] = [&items[0], &items[1], &items[2], &items[3]];
            write(
                "src/forms/form1.rs",
                format!(
                    "{h}/// {cf}\nconst FILE: &str = \"notes.txt\";\n\n#[derive(Default)]\npub struct Form1Data {{}}\n\nimpl Form1 {{\n    /// {dn}\n    pub fn {new}_click(&mut self, app: &mut AppContext) {{\n        self.text_box1.text.clear();\n        self.label1.text.clear();\n    }}\n\n    /// {dopen}\n    pub fn {open}_click(&mut self, app: &mut AppContext) {{\n        // {cr}\n        match std::fs::read_to_string(FILE) {{\n            Ok(text) => {{\n                self.text_box1.text = text;\n                self.label1.text = format!({opened}, FILE);\n            }}\n            Err(e) => self.label1.text = format!({error}, e),\n        }}\n    }}\n\n    /// {dsave}\n    pub fn {save}_click(&mut self, app: &mut AppContext) {{\n        match std::fs::write(FILE, &self.text_box1.text) {{\n            Ok(()) => self.label1.text = format!({saved}, FILE),\n            Err(e) => self.label1.text = format!({error}, e),\n        }}\n    }}\n\n    /// {dq}\n    pub fn {quit}_click(&mut self, app: &mut AppContext) {{\n        app.exit();\n    }}\n}}\n",
                    h = header(tr, "Form1", id),
                    cf = tr("ex-comment-file"),
                    dn = comments.handler_doc("Click", new),
                    dopen = comments.handler_doc("Click", open),
                    cr = tr("ex-comment-read-file"),
                    opened = lit(&format!("{} {{}}", esc(&tr("ex-opened")))),
                    error = lit(&format!("{} {{}}", esc(&tr("ex-file-error")))),
                    dsave = comments.handler_doc("Click", save),
                    saved = lit(&format!("{} {{}}", esc(&tr("ex-saved")))),
                    dq = comments.handler_doc("Click", quit),
                ),
            )?;
        }
        _ => {}
    }
    project.sync_code(&[], comments)?;
    // Les copies .bak des étapes intermédiaires n'ont pas d'intérêt dans un exemple.
    remove_backups(&project.root);
    Ok(project)
}

fn remove_backups(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            remove_backups(&p);
        } else if p.extension().is_some_and(|x| x == "bak") {
            let _ = std::fs::remove_file(&p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vrs_codegen::EnglishComments;

    fn tr(key: &str) -> String {
        key.to_owned()
    }

    #[test]
    fn every_example_is_created_and_parses() {
        let dir = tempfile::tempdir().unwrap();
        for ex in EXAMPLES {
            let p = create_example(ex.id, dir.path(), &tr, &EnglishComments).unwrap();
            for entry in std::fs::read_dir(p.root.join("src")).unwrap().flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    syn::parse_file(&text)
                        .unwrap_or_else(|e| panic!("{}: {e}\n{text}", path.display()));
                }
            }
            if ex.kind == ProjectKind::Gui {
                for f in &p.file.forms {
                    let user = vrs_codegen::form_paths(&p.root, f).user;
                    let text = std::fs::read_to_string(&user).unwrap();
                    syn::parse_file(&text)
                        .unwrap_or_else(|e| panic!("{}: {e}\n{text}", user.display()));
                    assert!(text.contains(&format!("pub struct {f}Data")));
                }
            }
        }
        // Un deuxième exemplaire reçoit un autre nom de dossier.
        let again = create_example("counter", dir.path(), &tr, &EnglishComments).unwrap();
        assert!(again.root.ends_with("ClickCounter2"));
    }
}
