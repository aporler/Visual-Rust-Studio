//! Test d'intégration clé : un projet créé par VRS compile **sans VRS**.
//!
//! Le projet est généré dans un dossier temporaire (hors du workspace VRS), un
//! bouton, une zone de texte et un gestionnaire Click y sont ajoutés comme le
//! ferait le concepteur, puis `cargo build` est lancé.
//!
//! Ces tests téléchargent/compilent eframe : ils sont ignorés par défaut.
//! Lancer avec : `cargo test -p vrs-project --test standalone -- --ignored --nocapture`

use std::path::{Path, PathBuf};
use std::process::Command;

use vrs_codegen::EnglishComments;
use vrs_components::{ComponentKind, EventKind};
use vrs_project::{ProjectKind, create_project, open_project};

fn cargo() -> PathBuf {
    std::env::var_os("CARGO")
        .map(PathBuf::from)
        .unwrap_or_else(|| "cargo".into())
}

/// Répertoire cible partagé pour ne compiler eframe qu'une fois.
fn shared_target() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/standalone-test")
}

fn build(root: &Path) -> (bool, String) {
    let out = Command::new(cargo())
        .arg("build")
        .current_dir(root)
        .env("CARGO_TARGET_DIR", shared_target())
        .env_remove("RUSTFLAGS")
        .output()
        .expect("lancement de cargo");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (out.status.success(), stderr)
}

#[test]
#[ignore = "lent : compile eframe"]
fn gui_project_compiles_without_vrs() {
    let dir = tempfile::tempdir().unwrap();
    let project =
        create_project(dir.path(), "DemoApp", ProjectKind::Gui, &EnglishComments).unwrap();

    // Ce que fait l'utilisateur dans le concepteur.
    let mut form = project.load_form("Form1").unwrap();
    form.title = "Démo VRS".into();
    let button = form.add_control(ComponentKind::Button, 16.0, 16.0);
    let text_box = form.add_control(ComponentKind::TextBox, 16.0, 56.0);
    form.add_control(ComponentKind::Label, 16.0, 96.0);
    // Tous les autres composants, avec leurs propriétés et événements.
    for (i, kind) in ComponentKind::ALL.iter().skip(3).enumerate() {
        let name = form.add_control(
            *kind,
            200.0 + (i % 2) as f32 * 180.0,
            16.0 + (i / 2) as f32 * 110.0,
        );
        let c = form.control_mut(&name).unwrap();
        c.items = vec!["Un".into(), "Deux".into()];
        c.selected = Some(0);
        c.value = 40.0;
        c.checked = true;
        c.back_color = Some([240, 240, 255, 255]);
        c.fore_color = Some([200, 30, 30, 128]);
        c.font_size = Some(16.0);
        c.anchor.right = true;
        if let Some(ev) = kind.default_event() {
            c.events.insert(ev, ev.default_handler(&name));
        }
    }
    form.add_control(ComponentKind::RadioButton, 200.0, 460.0);
    // Image intégrée (PNG 1×1) : les dépendances d'image sont ajoutées au Cargo.toml.
    const PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
        0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0xF8, 0xCF, 0xC0, 0xF0,
        0x1F, 0x00, 0x05, 0x00, 0x01, 0xFF, 0x89, 0x99, 0x3D, 0x1D, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
        0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    std::fs::create_dir_all(project.root.join("resources")).unwrap();
    std::fs::write(project.root.join("resources/logo.png"), PNG).unwrap();
    let pic = form
        .controls
        .iter()
        .find(|c| c.kind == ComponentKind::PictureBox)
        .unwrap()
        .name
        .clone();
    form.control_mut(&pic).unwrap().image = "resources/logo.png".into();
    form.control_mut(&button).unwrap().text = "Dire bonjour".into();
    // Double-clic sur le bouton : création du gestionnaire Click.
    form.control_mut(&button)
        .unwrap()
        .events
        .insert(EventKind::Click, EventKind::Click.default_handler(&button));
    form.save(&project.form_path("Form1")).unwrap();

    // Enregistrement et réouverture du projet, puis génération du code.
    let project = open_project(&project.root).unwrap();
    let report = project.sync_code(&[], &EnglishComments).unwrap();
    assert!(report.handlers_added.contains(&"button1_click".to_string()));
    assert!(
        report
            .handlers_added
            .contains(&"slider1_value_changed".to_string())
    );

    // L'utilisateur écrit son code dans le gestionnaire.
    let user = project.root.join("src/forms/form1.rs");
    let src = std::fs::read_to_string(&user).unwrap();
    let src = src
        .replace(
            "// TODO: write your code here.",
            &format!("self.{text_box}.text = String::from(\"Bonjour depuis Rust !\");"),
        )
        .replacen(
            "// TODO: write your code here.",
            "self.progress_bar1.value = self.slider1.value;",
            1,
        );
    std::fs::write(&user, &src).unwrap();

    // Ajout de Form2 : une zone de texte et un bouton OK qui ferme la fenêtre.
    let mut project = open_project(&project.root).unwrap();
    project.add_form("Form2", &EnglishComments).unwrap();
    // Le code utilisateur n'a pas été touché.
    assert_eq!(std::fs::read_to_string(&user).unwrap(), src);
    let mut form2 = project.load_form("Form2").unwrap();
    form2.add_control(ComponentKind::TextBox, 16.0, 16.0);
    let ok = form2.add_control(ComponentKind::Button, 16.0, 56.0);
    form2
        .control_mut(&ok)
        .unwrap()
        .events
        .insert(EventKind::Click, EventKind::Click.default_handler(&ok));
    form2.events.insert(
        EventKind::Closing,
        EventKind::Closing.default_handler("form2"),
    );
    form2.save(&project.form_path("Form2")).unwrap();
    project.sync_code(&[], &EnglishComments).unwrap();
    let user2 = project.root.join("src/forms/form2.rs");
    let src2 = std::fs::read_to_string(&user2).unwrap();
    assert!(src2.contains("pub fn form2_closing(&mut self, app: &mut AppContext) -> bool"));
    let src2 = src2.replacen("// TODO: write your code here.", "app.close();", 1);
    std::fs::write(&user2, src2).unwrap();
    // Form1 ouvre Form2 en dialogue et récupère son texte à la fermeture.
    let src = std::fs::read_to_string(&user).unwrap().replace(
        "self.text_box1.text = String::from(\"Bonjour depuis Rust !\");",
        "self.text_box1.text = String::from(\"Bonjour depuis Rust !\");\n        \
         app.show_dialog_then(super::Form2::default(), |me: &mut Form1, f2: super::Form2, _app| {\n            \
         me.text_box1.text = f2.text_box1.text;\n        });",
    );
    std::fs::write(&user, &src).unwrap();

    let (ok, stderr) = build(&project.root);
    assert!(ok, "échec de compilation :\n{stderr}");
    let own_warnings: Vec<&str> = stderr
        .lines()
        .filter(|l| l.starts_with("warning") && !l.contains("generated"))
        .collect();
    assert!(own_warnings.is_empty(), "avertissements :\n{stderr}");
}

#[test]
#[ignore = "lent : lance cargo"]
fn console_and_library_projects_compile() {
    let dir = tempfile::tempdir().unwrap();
    for (name, kind) in [
        ("cli", ProjectKind::Console),
        ("lib1", ProjectKind::Library),
    ] {
        let p = create_project(dir.path(), name, kind, &EnglishComments).unwrap();
        let (ok, stderr) = build(&p.root);
        assert!(ok, "{name} : {stderr}");
    }
}
