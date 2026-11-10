//! Boîte à outils : composants à placer sur la fenêtre.

use vrs_components::ComponentKind;
use vrs_i18n::I18n;

use crate::DesignerState;

fn icon(kind: ComponentKind) -> &'static str {
    match kind {
        ComponentKind::Button => "🆗",
        ComponentKind::Label => "🏷",
        ComponentKind::TextBox => "✏",
    }
}

/// Affiche la boîte à outils. Un clic choisit le composant (puis un clic sur la
/// fenêtre le place) ; un glisser-déposer le place directement.
pub fn toolbox_ui(ui: &mut egui::Ui, state: Option<&mut DesignerState>, i18n: &I18n) {
    let Some(state) = state else {
        ui.add_enabled_ui(false, |ui| {
            for kind in ComponentKind::ALL {
                ui.label(format!("{}  {}", icon(kind), i18n.tr(kind.i18n_key())));
            }
        });
        return;
    };
    if ui
        .selectable_label(
            state.pending_tool.is_none(),
            format!("↖  {}", i18n.tr("component-pointer")),
        )
        .clicked()
    {
        state.pending_tool = None;
    }
    for kind in ComponentKind::ALL {
        let selected = state.pending_tool == Some(kind);
        let text = format!("{}  {}", icon(kind), i18n.tr(kind.i18n_key()));
        let r = ui
            .selectable_label(selected, text.as_str())
            .interact(egui::Sense::drag())
            .on_hover_text(i18n.tr(kind.i18n_desc_key()));
        // Glisser-déposer vers la surface de conception.
        r.dnd_set_drag_payload(kind);
        if r.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            if let Some(p) = ui.ctx().pointer_interact_pos() {
                egui::Area::new(egui::Id::unique("vrs-toolbox-drag"))
                    .order(egui::Order::Tooltip)
                    .fixed_pos(p + egui::vec2(12.0, 8.0))
                    .interactable(false)
                    .show(ui.ctx(), |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| ui.label(text.as_str()));
                    });
            }
        }
        if r.clicked() {
            state.pending_tool = if selected { None } else { Some(kind) };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use vrs_components::FormDocument;

    use super::*;
    use crate::{canvas_ui, properties_ui};

    /// Rendu complet du concepteur sans écran : boîte à outils, surface, propriétés.
    #[test]
    fn designer_renders_and_places_component() {
        use egui_kittest::kittest::Queryable;
        let i18n = I18n::with_embedded("fr-CA");
        let mut state = DesignerState::new(FormDocument::new("Form1"), Path::new("/tmp/F.vrsform"));
        state.add_control(ComponentKind::Button, 16.0, 16.0);
        state.doc.controls[0].text = "Cliquez-moi".into();
        let settings = vrs_settings::DesignerSettings::default();
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(1000.0, 700.0))
            .with_accessibility_check(false)
            .build_ui_state(
                |ui, state: &mut DesignerState| {
                    egui::Panel::right("p").show(ui, |ui| {
                        toolbox_ui(ui, Some(state), &i18n);
                        ui.separator();
                        properties_ui(ui, state, &i18n);
                    });
                    egui::CentralPanel::default().show(ui, |ui| {
                        canvas_ui(ui, state, &settings, &i18n);
                    });
                },
                state,
            );
        harness.run();
        // Le bouton est dessiné avec son texte et ses propriétés sont visibles.
        assert!(harness.query_by_label("Cliquez-moi").is_some());
        // Choisir « Zone de texte » dans la boîte à outils puis cliquer sur la fenêtre.
        harness.get_by_label_contains("Zone de texte").click();
        harness.run();
        assert_eq!(harness.state().pending_tool, Some(ComponentKind::TextBox));
    }
}
