//! Surface de conception : dessin de la fenêtre, sélection, déplacement,
//! redimensionnement, placement de composants.

use egui::{CursorIcon, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2};
use vrs_components::ops::snap;
use vrs_components::{ComponentKind, Control, EventKind};
use vrs_i18n::I18n;
use vrs_runtime::{Bounds, ButtonState, LabelState, TextBoxState};
use vrs_settings::DesignerSettings;

use crate::state::{DesignerOutput, DesignerState, DragOp, HandlerRequest, now};

const MARGIN: f32 = 24.0;
const TITLE_BAR: f32 = 30.0;
const HANDLE: f32 = 7.0;
const MIN_SIZE: f32 = 8.0;

/// Les 8 poignées : (dx, dy) ∈ {-1, 0, 1}², sans (0, 0).
const HANDLES: [(i8, i8); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

fn bounds(c: &Control) -> Bounds {
    Bounds::new(c.x, c.y, c.width, c.height)
}

fn control_rect(origin: Pos2, c: &Control) -> Rect {
    Rect::from_min_size(
        origin + vec2(c.x, c.y),
        vec2(c.width.max(1.0), c.height.max(1.0)),
    )
}

fn handle_rect(r: Rect, (dx, dy): (i8, i8)) -> Rect {
    let x = match dx {
        -1 => r.left(),
        0 => r.center().x,
        _ => r.right(),
    };
    let y = match dy {
        -1 => r.top(),
        0 => r.center().y,
        _ => r.bottom(),
    };
    Rect::from_center_size(pos2(x, y), Vec2::splat(HANDLE))
}

fn handle_cursor((dx, dy): (i8, i8)) -> CursorIcon {
    match (dx, dy) {
        (0, _) => CursorIcon::ResizeVertical,
        (_, 0) => CursorIcon::ResizeHorizontal,
        (a, b) if a == b => CursorIcon::ResizeNwSe,
        _ => CursorIcon::ResizeNeSw,
    }
}

/// Dessine un contrôle avec les fonctions du runtime (rendu identique à l'application).
fn render_control(
    ui: &mut egui::Ui,
    c: &Control,
    preview_text: Option<&mut String>,
) -> egui::Response {
    let b = bounds(c);
    match c.kind {
        ComponentKind::Button => {
            let mut s = ButtonState::new(c.text.clone(), b);
            s.enabled = c.enabled;
            vrs_runtime::button(ui, &c.name, &s)
        }
        ComponentKind::Label => {
            let mut s = LabelState::new(c.text.clone(), b);
            s.enabled = c.enabled;
            vrs_runtime::label(ui, &c.name, &s)
        }
        ComponentKind::TextBox => {
            let mut s = TextBoxState::new(c.text.clone(), b);
            s.enabled = c.enabled;
            if let Some(t) = &preview_text {
                s.text = (*t).clone();
            }
            let r = vrs_runtime::text_box(ui, &c.name, &mut s);
            if let Some(t) = preview_text {
                *t = s.text;
            }
            r
        }
    }
}

/// Affiche la surface de conception.
pub fn canvas_ui(
    ui: &mut egui::Ui,
    state: &mut DesignerState,
    settings: &DesignerSettings,
    i18n: &I18n,
) -> DesignerOutput {
    let mut out = DesignerOutput::default();
    let before = state.doc.clone();
    let grid = if settings.snap_to_grid {
        settings.grid_size
    } else {
        0.0
    };

    ui.horizontal(|ui| {
        ui.toggle_value(&mut state.preview, i18n.tr("designer-preview"))
            .on_hover_text(i18n.tr("designer-preview-hint"));
        ui.weak(i18n.tr("designer-hint"));
    });
    ui.separator();

    egui::ScrollArea::both()
        .id_salt(("designer-scroll", &state.path))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total = vec2(
                state.doc.width + 2.0 * MARGIN + 16.0,
                state.doc.height + TITLE_BAR + 2.0 * MARGIN + 16.0,
            );
            let (area, bg) = ui.allocate_exact_size(total, Sense::click());
            let window = Rect::from_min_size(
                area.min + Vec2::splat(MARGIN),
                vec2(state.doc.width, state.doc.height + TITLE_BAR),
            );
            let client =
                Rect::from_min_max(pos2(window.left(), window.top() + TITLE_BAR), window.max);
            let origin = client.min;
            let visuals = ui.visuals().clone();
            let painter = ui.painter_at(area);

            // Cadre de fenêtre simulé.
            painter.rect_filled(
                window.expand(1.0),
                6.0,
                visuals.widgets.noninteractive.bg_stroke.color,
            );
            painter.rect_filled(
                Rect::from_min_size(window.min, vec2(window.width(), TITLE_BAR)),
                egui::CornerRadius {
                    nw: 6,
                    ne: 6,
                    sw: 0,
                    se: 0,
                },
                visuals.faint_bg_color,
            );
            painter.text(
                window.min + vec2(10.0, TITLE_BAR / 2.0),
                egui::Align2::LEFT_CENTER,
                &state.doc.title,
                egui::FontId::proportional(14.0),
                visuals.strong_text_color(),
            );
            for (i, sym) in ["✖", "⏹", "–"].iter().enumerate() {
                painter.text(
                    pos2(
                        window.right() - 16.0 - i as f32 * 24.0,
                        window.top() + TITLE_BAR / 2.0,
                    ),
                    egui::Align2::CENTER_CENTER,
                    sym,
                    egui::FontId::proportional(13.0),
                    visuals.weak_text_color(),
                );
            }
            painter.rect_filled(client, 0.0, visuals.panel_fill);

            // Grille.
            if settings.show_grid && settings.grid_size >= 4.0 {
                let dot = visuals.weak_text_color().gamma_multiply(0.35);
                let g = settings.grid_size;
                let mut y = client.top();
                while y <= client.bottom() {
                    let mut x = client.left();
                    while x <= client.right() {
                        painter.rect_filled(
                            Rect::from_min_size(pos2(x, y), Vec2::splat(1.0)),
                            0.0,
                            dot,
                        );
                        x += g;
                    }
                    y += g;
                }
            }

            // Contrôles : rendu réel, dans une zone limitée à la zone cliente.
            let mut previews = std::mem::take(&mut state.preview_states);
            let preview = state.preview;
            let controls = state.doc.controls.clone();
            ui.scope_builder(egui::UiBuilder::new().max_rect(client), |ui| {
                ui.set_clip_rect(client.intersect(ui.clip_rect()));
                for c in &controls {
                    let text = if preview && c.kind == ComponentKind::TextBox {
                        Some(
                            previews
                                .entry(c.name.clone())
                                .or_insert_with(|| c.text.clone()),
                        )
                    } else {
                        None
                    };
                    render_control(ui, c, text);
                    if !c.visible {
                        // Contrôle masqué à l'exécution : affiché en transparence.
                        let r = control_rect(origin, c);
                        ui.painter()
                            .rect_filled(r, 0.0, visuals.panel_fill.gamma_multiply(0.6));
                    }
                }
            });
            if !preview {
                previews.clear();
            }
            state.preview_states = previews;

            if preview {
                return;
            }

            // Interaction : un calque transparent au-dessus de chaque contrôle
            // (enregistré après les widgets, il reçoit les clics en priorité).
            let shift = ui.input(|i| i.modifiers.shift || i.modifiers.command);
            let mut clicked_control = false;
            for c in controls.iter().rev() {
                let r = control_rect(origin, c).intersect(client);
                if !r.is_positive() {
                    continue;
                }
                let id = ui.scope_id().with(("vrs-overlay", &c.name));
                let resp = ui.interact(r, id, Sense::click_and_drag());
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(CursorIcon::Move);
                }
                if resp.clicked() || resp.drag_started() || resp.secondary_clicked() {
                    clicked_control = true;
                    if shift && resp.clicked() {
                        if let Some(pos) = state.selection.iter().position(|n| *n == c.name) {
                            state.selection.remove(pos);
                        } else {
                            state.selection.push(c.name.clone());
                        }
                    } else if !state.selection.contains(&c.name) {
                        state.selection = vec![c.name.clone()];
                    }
                }
                if resp.double_clicked() {
                    state.selection = vec![c.name.clone()];
                    out.open_handler = Some(HandlerRequest {
                        control: Some(c.name.clone()),
                        event: c.kind.default_event(),
                    });
                }
                if resp.drag_started() {
                    state.history.break_coalescing();
                    let start = state
                        .selection
                        .iter()
                        .filter_map(|n| state.doc.control(n).map(|c| (n.clone(), c.x, c.y)))
                        .collect();
                    state.drag = Some(DragOp::Move {
                        start,
                        total: Vec2::ZERO,
                    });
                }
                resp.context_menu(|ui| {
                    if ui.button(i18n.tr("cmd-view-code")).clicked() {
                        out.open_handler = Some(HandlerRequest {
                            control: Some(c.name.clone()),
                            event: c.kind.default_event(),
                        });
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(i18n.tr("cmd-bring-front")).clicked() {
                        state.reorder(true);
                        ui.close();
                    }
                    if ui.button(i18n.tr("cmd-send-back")).clicked() {
                        state.reorder(false);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(i18n.tr("cmd-delete")).clicked() {
                        state.delete_selection();
                        ui.close();
                    }
                });
                if resp.dragged()
                    && let Some(DragOp::Move { total, .. }) = &mut state.drag
                {
                    *total += resp.drag_delta();
                }
            }

            // Poignées de redimensionnement (sélection unique).
            if state.selection.len() == 1
                && let Some(c) = state.doc.control(&state.selection[0]).cloned()
            {
                let r = control_rect(origin, &c);
                for h in HANDLES {
                    let hr = handle_rect(r, h);
                    let id = ui.scope_id().with(("vrs-handle", &c.name, h));
                    let resp = ui.interact(hr.expand(2.0), id, Sense::drag());
                    if resp.hovered() || resp.dragged() {
                        ui.ctx().set_cursor_icon(handle_cursor(h));
                    }
                    if resp.drag_started() {
                        state.history.break_coalescing();
                        state.drag = Some(DragOp::Resize {
                            name: c.name.clone(),
                            handle: h,
                            start: c.rect(),
                            total: Vec2::ZERO,
                        });
                    }
                    if resp.dragged()
                        && let Some(DragOp::Resize { total, .. }) = &mut state.drag
                    {
                        *total += resp.drag_delta();
                    }
                }
            }

            // Poignée de la fenêtre (coin inférieur droit).
            let form_handle = Rect::from_center_size(client.max, Vec2::splat(HANDLE + 2.0));
            let fr = ui.interact(
                form_handle,
                ui.scope_id().with("vrs-form-handle"),
                Sense::drag(),
            );
            if fr.hovered() || fr.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
            }
            if fr.drag_started() {
                state.history.break_coalescing();
                state.drag = Some(DragOp::ResizeForm {
                    start: (state.doc.width, state.doc.height),
                    total: Vec2::ZERO,
                });
            }
            if fr.dragged()
                && let Some(DragOp::ResizeForm { total, .. }) = &mut state.drag
            {
                *total += fr.drag_delta();
            }

            // Application du glisser en cours.
            apply_drag(state, grid);
            if !ui.input(|i| i.pointer.any_down()) && state.drag.take().is_some() {
                state.history.break_coalescing();
            }

            // Clic sur le fond : placer le composant choisi, ou désélectionner.
            if bg.clicked() && !clicked_control {
                match (state.pending_tool, bg.interact_pointer_pos()) {
                    (Some(kind), Some(p)) if client.contains(p) => {
                        let local = p - origin;
                        state.add_control(kind, snap(local.x, grid), snap(local.y, grid));
                    }
                    _ => state.selection.clear(),
                }
            }
            if bg.double_clicked() && !clicked_control {
                out.open_handler = Some(HandlerRequest {
                    control: None,
                    event: EventKind::Load,
                });
            }
            // Glisser-déposer depuis la boîte à outils.
            if let Some(kind) = bg.dnd_release_payload::<ComponentKind>()
                && let Some(p) = ui.input(|i| i.pointer.interact_pos())
            {
                let local = p - origin;
                state.add_control(
                    *kind,
                    snap(local.x.clamp(0.0, state.doc.width - 8.0), grid),
                    snap(local.y.clamp(0.0, state.doc.height - 8.0), grid),
                );
            }
            if bg.dnd_hover_payload::<ComponentKind>().is_some() {
                painter.rect_stroke(
                    client,
                    0.0,
                    Stroke::new(2.0, visuals.selection.bg_fill),
                    StrokeKind::Inside,
                );
            }
            if state.pending_tool.is_some() && bg.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            }

            // Dessin de la sélection.
            let sel_color = visuals.selection.stroke.color;
            let fg = ui.painter();
            for (i, name) in state.selection.iter().enumerate() {
                if let Some(c) = state.doc.control(name) {
                    let r = control_rect(origin, c).expand(2.0);
                    let width = if i == 0 { 2.0 } else { 1.0 };
                    fg.rect_stroke(r, 0.0, Stroke::new(width, sel_color), StrokeKind::Outside);
                    if state.selection.len() == 1 {
                        for h in HANDLES {
                            let hr = handle_rect(control_rect(origin, c), h);
                            fg.rect_filled(hr, 1.0, visuals.panel_fill);
                            fg.rect_stroke(
                                hr,
                                1.0,
                                Stroke::new(1.0, sel_color),
                                StrokeKind::Inside,
                            );
                        }
                    }
                }
            }
            fg.rect_filled(form_handle.shrink(1.0), 1.0, sel_color);

            // Clavier : flèches pour déplacer (si aucun champ texte n'a le focus).
            let typing = ui.ctx().memory(|m| m.focused().is_some());
            if !typing && !state.selection.is_empty() && ui.ui_contains_pointer() {
                let step = if ui.input(|i| i.modifiers.shift) {
                    grid.max(1.0)
                } else {
                    1.0
                };
                let (mut dx, mut dy) = (0.0, 0.0);
                ui.input(|i| {
                    if i.key_pressed(egui::Key::ArrowLeft) {
                        dx -= step;
                    }
                    if i.key_pressed(egui::Key::ArrowRight) {
                        dx += step;
                    }
                    if i.key_pressed(egui::Key::ArrowUp) {
                        dy -= step;
                    }
                    if i.key_pressed(egui::Key::ArrowDown) {
                        dy += step;
                    }
                });
                if dx != 0.0 || dy != 0.0 {
                    state.nudge(dx, dy);
                }
            }
        });

    if state.doc != before {
        out.changed = true;
        // Les glisser-déplacer forment une seule étape d'annulation.
        let key = state.drag.as_ref().map(|_| "drag".to_string());
        if key.is_some() {
            state.history.record(before, key, now());
        }
    }
    out
}

/// Recalcule positions/tailles à partir du déplacement cumulé.
fn apply_drag(state: &mut DesignerState, grid: f32) {
    let Some(drag) = state.drag.clone() else {
        return;
    };
    let (fw, fh) = (state.doc.width, state.doc.height);
    match drag {
        DragOp::Move { start, total } => {
            for (name, x, y) in start {
                if let Some(c) = state.doc.control_mut(&name) {
                    c.x = snap(x + total.x, grid).clamp(-c.width + 4.0, fw - 4.0);
                    c.y = snap(y + total.y, grid).clamp(-c.height + 4.0, fh - 4.0);
                }
            }
        }
        DragOp::Resize {
            name,
            handle: (hx, hy),
            start: [x, y, w, h],
            total,
        } => {
            if let Some(c) = state.doc.control_mut(&name) {
                if hx == 1 {
                    c.width = (snap(x + w + total.x, grid) - x).max(MIN_SIZE);
                } else if hx == -1 {
                    let nx = snap(x + total.x, grid).min(x + w - MIN_SIZE);
                    c.x = nx;
                    c.width = x + w - nx;
                }
                if hy == 1 {
                    c.height = (snap(y + h + total.y, grid) - y).max(MIN_SIZE);
                } else if hy == -1 {
                    let ny = snap(y + total.y, grid).min(y + h - MIN_SIZE);
                    c.y = ny;
                    c.height = y + h - ny;
                }
            }
        }
        DragOp::ResizeForm {
            start: (w, h),
            total,
        } => {
            state.doc.width = snap(w + total.x, grid).max(120.0);
            state.doc.height = snap(h + total.y, grid).max(80.0);
        }
    }
}
