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
}

impl Level {
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::Beginner => "examples-beginner",
            Self::Intermediate => "examples-intermediate",
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
    Example {
        id: "hello-terminal",
        level: Level::Beginner,
        kind: ProjectKind::Console,
        project_name: "HelloWorldTerminal",
    },
    Example {
        id: "hello-window",
        level: Level::Beginner,
        kind: ProjectKind::Gui,
        project_name: "HelloWorldWindow",
    },
    Example {
        id: "counter",
        level: Level::Beginner,
        kind: ProjectKind::Gui,
        project_name: "ClickCounter",
    },
    Example {
        id: "temperature",
        level: Level::Beginner,
        kind: ProjectKind::Gui,
        project_name: "TemperatureConverter",
    },
    Example {
        id: "todo",
        level: Level::Intermediate,
        kind: ProjectKind::Gui,
        project_name: "TodoList",
    },
    Example {
        id: "guess",
        level: Level::Intermediate,
        kind: ProjectKind::Gui,
        project_name: "GuessTheNumber",
    },
    Example {
        id: "two-windows",
        level: Level::Intermediate,
        kind: ProjectKind::Gui,
        project_name: "TwoWindows",
    },
];

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
        _ => {}
    }
    project.sync_code(&[], comments)?;
    Ok(project)
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
