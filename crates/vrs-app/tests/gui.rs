//! Tests de bout en bout de l'IDE, sans écran (egui_kittest).
//!
//! Ils pilotent la véritable interface de VRS : rendu des menus, des panneaux,
//! du concepteur et de l'éditeur dans plusieurs langues, création d'un projet,
//! ajout de contrôles, gestionnaire Click, Form2, sauvegarde et réouverture.

use std::path::Path;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use vrs_app::{Command, Tab, VrsApp};
use vrs_components::{ComponentKind, EventKind};
use vrs_designer::HandlerRequest;
use vrs_project::ProjectKind;
use vrs_settings::Settings;

fn app_in(dir: &Path, lang: &str) -> VrsApp {
    let settings = Settings {
        language: Some(lang.into()),
        ..Default::default()
    };
    VrsApp::with_settings(settings, Some(dir.join("settings.toml")))
}

fn harness(app: VrsApp) -> Harness<'static, VrsApp> {
    Harness::builder()
        .with_size(egui::vec2(1400.0, 900.0))
        .with_accessibility_check(false)
        .build_ui_state(|ui, app: &mut VrsApp| app.frame(ui), app)
}

/// Exécute les commandes en attente avec le contexte du harnais.
fn run_cmd(h: &mut Harness<'static, VrsApp>, c: Command) {
    h.state_mut().cmd(c);
    h.run();
}

#[test]
fn main_window_in_french_then_english_without_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness(app_in(dir.path(), "fr-CA"));
    h.run();
    for label in [
        "Fichier",
        "Édition",
        "Affichage",
        "Projet",
        "Compiler",
        "Déboguer",
        "Outils",
        "Fenêtre",
        "Aide",
    ] {
        assert!(
            h.query_all_by_label(label).next().is_some(),
            "menu « {label} » absent"
        );
    }
    assert!(
        h.query_by_label_contains("Bienvenue dans Visual Rust Studio")
            .is_some()
    );

    // Ouvrir le menu Fichier.
    h.get_by_label("Fichier").click();
    h.run();
    assert!(h.query_by_label_contains("Quitter").is_some());

    // Changement de langue à chaud (Outils → Paramètres → Langue).
    h.state_mut().i18n.set_language("en-US".parse().unwrap());
    h.run();
    for label in [
        "File", "Edit", "View", "Project", "Build", "Debug", "Tools", "Window", "Help",
    ] {
        assert!(
            h.query_all_by_label(label).next().is_some(),
            "menu « {label} » absent"
        );
    }
    assert!(
        h.state().i18n.missing_keys().is_empty(),
        "{:?}",
        h.state().i18n.missing_keys()
    );
}

#[test]
fn every_language_renders_all_screens_without_missing_keys() {
    for lang in ["fr-CA", "fr-FR", "en-US", "en-CA", "es-ES"] {
        let dir = tempfile::tempdir().unwrap();
        let mut h = harness(app_in(dir.path(), lang));
        h.run();
        // Dialogues.
        for c in [Command::NewProject, Command::Settings, Command::About] {
            run_cmd(&mut h, c);
            h.run();
            let app = h.state_mut();
            app.dialogs.new_project = None;
            app.dialogs.settings = None;
            app.dialogs.about = false;
        }
        h.state_mut().dialogs.error = Some("test".into());
        h.run();
        h.state_mut().dialogs.error = None;

        // Projet graphique + concepteur + éditeur.
        let parent = dir.path().join("p");
        std::fs::create_dir_all(&parent).unwrap();
        h.state_mut()
            .create_project(ProjectKind::Gui, &parent, "Demo");
        h.run();
        {
            let app = h.state_mut();
            if let Some(d) = app.active_designer() {
                d.add_control(ComponentKind::Button, 16.0, 16.0);
            }
        }
        h.run();
        run_cmd(&mut h, Command::ViewCode);
        h.run();
        let missing = h.state().i18n.missing_keys();
        assert!(missing.is_empty(), "{lang} : clés manquantes {missing:?}");
    }
}

#[test]
fn mvp_flow_project_form_button_click_handler_form2_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness(app_in(dir.path(), "fr-CA"));
    h.run();

    // 3. Créer un projet graphique → 4. Form1 s'affiche dans le concepteur.
    h.state_mut()
        .create_project(ProjectKind::Gui, dir.path(), "MonApp");
    h.run();
    assert!(matches!(h.state().active_tab(), Some(Tab::Designer(d)) if d.doc.name == "Form1"));
    assert!(h.query_by_label_contains("Form1 [Conception]").is_some());

    // 5. Ajouter un bouton et une zone de texte ; 6. modifier leurs propriétés.
    {
        let d = h.state_mut().active_designer().unwrap();
        d.add_control(ComponentKind::Button, 16.0, 16.0);
        d.add_control(ComponentKind::TextBox, 16.0, 56.0);
        d.edit(|doc| doc.control_mut("button1").unwrap().text = "Dire bonjour".into());
        d.selection = vec!["button1".into()];
    }
    h.run();
    // Le bouton est dessiné dans le concepteur et ses propriétés sont affichées.
    assert!(h.query_by_label("Dire bonjour").is_some());
    assert!(h.query_by_label_contains("Nom (Name)").is_some());

    // 9. Double-clic sur le bouton → gestionnaire Click créé et ouvert dans l'éditeur.
    run_cmd(
        &mut h,
        Command::OpenHandler(HandlerRequest {
            control: Some("button1".into()),
            event: EventKind::Click,
        }),
    );
    h.run();
    let user_file = {
        let app = h.state();
        let Some(Tab::Code(e)) = app.active_tab() else {
            panic!("l'éditeur de code devrait être actif");
        };
        assert!(e.path.ends_with("src/forms/form1.rs"));
        assert!(
            e.text
                .contains("pub fn button1_click(&mut self, app: &mut AppContext)")
        );
        e.path.clone()
    };

    // 10. Modifier le texte du champ lors du clic (code écrit par l'utilisateur).
    {
        let app = h.state_mut();
        let Some(Tab::Code(e)) = app.active_tab_mut() else {
            unreachable!()
        };
        e.text = e.text.replace(
            "// À FAIRE : écrivez votre code ici.",
            "self.text_box1.text = String::from(\"Bonjour !\");",
        );
    }
    run_cmd(&mut h, Command::SaveAll);
    let saved = std::fs::read_to_string(&user_file).unwrap();
    assert!(saved.contains("self.text_box1.text = String::from(\"Bonjour !\");"));

    // 11. Ajouter Form2.
    h.state_mut().add_form("Form2");
    h.run();
    assert!(matches!(h.state().active_tab(), Some(Tab::Designer(d)) if d.doc.name == "Form2"));
    // Le code utilisateur de Form1 est intact.
    assert_eq!(std::fs::read_to_string(&user_file).unwrap(), saved);

    // 7. Fermer et rouvrir le projet : tout est conservé.
    let root = h.state().project.as_ref().unwrap().root.clone();
    run_cmd(&mut h, Command::CloseProject);
    assert!(h.state().project.is_none());
    run_cmd(&mut h, Command::OpenPath(root.clone()));
    let app = h.state();
    let project = app.project.as_ref().unwrap();
    assert_eq!(project.file.forms, vec!["Form1", "Form2"]);
    let form1 = project.load_form("Form1").unwrap();
    assert_eq!(form1.controls.len(), 2);
    assert_eq!(form1.control("button1").unwrap().text, "Dire bonjour");
    assert_eq!(
        form1.control("button1").unwrap().events[&EventKind::Click],
        "button1_click"
    );
    // Projet récent mémorisé.
    assert!(
        app.settings
            .recent_projects
            .iter()
            .any(|p| p.starts_with(&root))
    );
}

#[test]
fn designer_undo_redo_and_format_commands() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness(app_in(dir.path(), "en-US"));
    h.state_mut()
        .create_project(ProjectKind::Gui, dir.path(), "App");
    h.run();
    {
        let d = h.state_mut().active_designer().unwrap();
        d.add_control(ComponentKind::Button, 10.0, 10.0);
        d.add_control(ComponentKind::Button, 50.0, 90.0);
        d.select_all();
    }
    run_cmd(
        &mut h,
        Command::Layout(vrs_components::ops::LayoutOp::AlignLeft),
    );
    let xs: Vec<f32> = h
        .state_mut()
        .active_designer()
        .unwrap()
        .doc
        .controls
        .iter()
        .map(|c| c.x)
        .collect();
    assert_eq!(xs, vec![10.0, 10.0]);
    run_cmd(&mut h, Command::Undo);
    assert_eq!(
        h.state_mut().active_designer().unwrap().doc.controls[1].x,
        50.0
    );
    run_cmd(&mut h, Command::Copy);
    run_cmd(&mut h, Command::Paste);
    assert_eq!(
        h.state_mut().active_designer().unwrap().doc.controls.len(),
        4
    );
    run_cmd(&mut h, Command::Delete);
    assert_eq!(
        h.state_mut().active_designer().unwrap().doc.controls.len(),
        2
    );
}

#[test]
fn untrusted_project_asks_before_running_cargo() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    let cfg = tempfile::tempdir().unwrap();
    let mut h = harness(app_in(cfg.path(), "fr-CA"));
    run_cmd(&mut h, Command::OpenPath(dir.path().to_path_buf()));
    run_cmd(&mut h, Command::Cargo(vrs_compiler::CargoCommand::Build));
    assert!(h.state().dialogs.trust.is_some());
    assert!(
        h.state().build.is_none(),
        "aucune commande ne doit être lancée sans accord"
    );
    assert!(h.query_by_label_contains("Projet non approuvé").is_some());
}

/// Chaîne complète avec Cargo : erreur de type → diagnostic E0308 + explication.
/// Lent (compile eframe) : `cargo test -p vrs-app --test gui -- --ignored`.
#[test]
#[ignore = "lent : lance cargo check sur un projet graphique"]
fn cargo_check_shows_diagnostic_with_explanation() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness(app_in(dir.path(), "fr-CA"));
    h.state_mut()
        .create_project(ProjectKind::Gui, dir.path(), "Diag");
    h.run();
    run_cmd(
        &mut h,
        Command::OpenHandler(HandlerRequest {
            control: None,
            event: EventKind::Load,
        }),
    );
    {
        let app = h.state_mut();
        let Some(Tab::Code(e)) = app.active_tab_mut() else {
            panic!()
        };
        e.text = e.text.replace(
            "// À FAIRE : écrivez votre code ici.",
            "let n: i32 = \"cinq\";",
        );
    }
    // Pendant la compilation, un indicateur animé redessine sans cesse : on
    // avance image par image.
    h.state_mut()
        .cmd(Command::Cargo(vrs_compiler::CargoCommand::Check));
    h.step();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1200);
    while h.state().is_building() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(200));
        h.step();
    }
    h.step();
    h.run();
    let app = h.state();
    let d = app
        .diagnostics
        .iter()
        .find(|d| d.code.as_deref() == Some("E0308"))
        .unwrap_or_else(|| {
            panic!(
                "E0308 attendu ; sortie : {:#?}",
                app.console.iter().map(|l| &l.text).collect::<Vec<_>>()
            )
        });
    assert!(
        d.span
            .as_ref()
            .unwrap()
            .file
            .ends_with("src/forms/form1.rs")
    );
    let explanation = app
        .i18n
        .try_tr(&d.explanation_key().unwrap(), None)
        .unwrap();
    assert!(explanation.contains("Types incompatibles"));
    // Le diagnostic est souligné dans l'éditeur ouvert.
    let Some(Tab::Code(e)) = app.active_tab() else {
        panic!()
    };
    assert!(e.diagnostics.iter().any(|x| x.message.contains("E0308")));
}

/// Aide au code : `clo` propose `app.exit()`, et `close();` reçoit une correction rapide.
#[test]
fn code_assistance_completion_and_quick_fix() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness(app_in(dir.path(), "fr-CA"));
    h.state_mut()
        .create_project(ProjectKind::Gui, dir.path(), "Aide");
    h.run();
    {
        let d = h.state_mut().active_designer().unwrap();
        d.add_control(ComponentKind::Button, 16.0, 16.0);
        d.add_control(ComponentKind::TextBox, 16.0, 56.0);
    }
    run_cmd(
        &mut h,
        Command::OpenHandler(HandlerRequest {
            control: Some("button1".into()),
            event: EventKind::Click,
        }),
    );

    // 1. Autocomplétion : « clo » → app.exit();
    let cursor = {
        let app = h.state_mut();
        let Some(Tab::Code(e)) = app.active_tab_mut() else {
            panic!()
        };
        assert!(e.completion.is_form);
        assert_eq!(e.completion.controls.len(), 2);
        e.text = e
            .text
            .replace("// À FAIRE : écrivez votre code ici.", "clo");
        let c = e.text.find("clo").unwrap() + 3;
        let c = e.text[..c].chars().count();
        e.set_cursor(c);
        c
    };
    h.run();
    {
        let app = h.state_mut();
        let Some(Tab::Code(e)) = app.active_tab_mut() else {
            panic!()
        };
        e.trigger_completion();
        let (_, items) = e.completion_items();
        assert_eq!(items[0].insert, "app.exit();", "{items:#?}");
    }
    h.run();
    // La liste est affichée, avec l'explication en français.
    assert!(h.query_by_label_contains("close() → app.exit()").is_some());
    assert!(h.query_by_label_contains("ferme l'application").is_some());
    h.key_press(egui::Key::Enter);
    h.run();
    {
        let app = h.state();
        let Some(Tab::Code(e)) = app.active_tab() else {
            panic!()
        };
        assert!(e.text.contains("        app.exit();\n"), "{}", e.text);
        assert!(!e.completion_open());
        let _ = cursor;
    }

    // 2. Correction rapide : « close(); » souligné, remplacé par app.exit().
    {
        let app = h.state_mut();
        let Some(Tab::Code(e)) = app.active_tab_mut() else {
            panic!()
        };
        e.text = e.text.replace(
            "app.exit();",
            "close();\n        TextBox1.text = String::new();",
        );
    }
    h.run();
    let fixes = {
        let app = h.state_mut();
        let i18n = vrs_i18n::I18n::with_embedded("fr-CA");
        let Some(Tab::Code(e)) = app.active_tab_mut() else {
            panic!()
        };
        e.hint_diagnostics(&i18n)
    };
    assert_eq!(fixes.len(), 2);
    assert!(fixes[0].message.contains("app.exit()"));
    {
        let app = h.state_mut();
        let Some(Tab::Code(e)) = app.active_tab_mut() else {
            panic!()
        };
        for f in fixes.iter().rev() {
            e.apply_fix(f);
        }
        assert!(e.text.contains("app.exit();"));
        assert!(e.text.contains("self.text_box1.text = String::new();"));
    }
    h.run();
    // Le code corrigé est du Rust valide.
    let app = h.state();
    let Some(Tab::Code(e)) = app.active_tab() else {
        panic!()
    };
    syn_check(&e.text);
}

fn syn_check(src: &str) {
    assert!(src.contains("impl Form1"), "{src}");
    assert_eq!(src.matches('{').count(), src.matches('}').count(), "{src}");
}

/// Phase 5 : fenêtre de démarrage, stratégie de fermeture, gestionnaire Closing.
#[test]
fn startup_form_close_strategy_and_closing_handler() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness(app_in(dir.path(), "fr-CA"));
    h.state_mut().create_project(ProjectKind::Gui, dir.path(), "Multi");
    h.run();
    h.state_mut().add_form("Form2");
    h.run();
    run_cmd(&mut h, Command::SetStartupForm("Form2".into()));
    run_cmd(&mut h, Command::SetCloseStrategy(true));
    let root = h.state().project.as_ref().unwrap().root.clone();
    let mod_rs = std::fs::read_to_string(root.join("src/forms/mod.rs")).unwrap();
    assert!(mod_rs.contains("pub fn startup_form() -> Form2"), "{mod_rs}");
    assert!(mod_rs.contains("CloseStrategy::LastFormCloses"), "{mod_rs}");
    let reopened = vrs_project::open_project(&root).unwrap();
    assert_eq!(reopened.file.startup_form.as_deref(), Some("Form2"));

    // Gestionnaire Closing (fenêtre) : il retourne bool.
    run_cmd(
        &mut h,
        Command::OpenHandler(HandlerRequest {
            control: None,
            event: EventKind::Closing,
        }),
    );
    let app = h.state();
    let Some(Tab::Code(e)) = app.active_tab() else { panic!() };
    assert!(e.text.contains("pub fn form2_closing(&mut self, app: &mut AppContext) -> bool"));
    assert!(e.completion.forms.contains(&"Form1".to_string()));
    let designer = std::fs::read_to_string(root.join("src/forms/form2_designer.rs")).unwrap();
    assert!(designer.contains("fn closing(&mut self, app: &mut AppContext) -> bool"));
}
