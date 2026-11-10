//! Opérations de mise en forme (menu Format) sur une sélection de contrôles.
//!
//! Comme dans Visual Studio, le **premier** contrôle sélectionné sert de référence
//! pour l'alignement et l'uniformisation des tailles.

use crate::FormDocument;

/// Opération de mise en forme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutOp {
    AlignLeft,
    AlignRight,
    AlignTop,
    AlignBottom,
    AlignCenterH,
    AlignCenterV,
    SameWidth,
    SameHeight,
    SameSize,
    DistributeH,
    DistributeV,
}

impl LayoutOp {
    pub const ALL: [LayoutOp; 11] = [
        Self::AlignLeft,
        Self::AlignRight,
        Self::AlignTop,
        Self::AlignBottom,
        Self::AlignCenterH,
        Self::AlignCenterV,
        Self::SameWidth,
        Self::SameHeight,
        Self::SameSize,
        Self::DistributeH,
        Self::DistributeV,
    ];

    /// Clé de traduction de la commande.
    pub fn i18n_key(self) -> &'static str {
        match self {
            Self::AlignLeft => "cmd-align-left",
            Self::AlignRight => "cmd-align-right",
            Self::AlignTop => "cmd-align-top",
            Self::AlignBottom => "cmd-align-bottom",
            Self::AlignCenterH => "cmd-align-center-h",
            Self::AlignCenterV => "cmd-align-center-v",
            Self::SameWidth => "cmd-same-width",
            Self::SameHeight => "cmd-same-height",
            Self::SameSize => "cmd-same-size",
            Self::DistributeH => "cmd-distribute-h",
            Self::DistributeV => "cmd-distribute-v",
        }
    }

    /// Nombre minimal de contrôles sélectionnés.
    pub fn min_selection(self) -> usize {
        match self {
            Self::DistributeH | Self::DistributeV => 3,
            _ => 2,
        }
    }
}

/// Applique `op` aux contrôles nommés. Retourne `true` si quelque chose a changé.
pub fn apply(doc: &mut FormDocument, selection: &[String], op: LayoutOp) -> bool {
    let idx: Vec<usize> = selection.iter().filter_map(|n| doc.index_of(n)).collect();
    if idx.len() < op.min_selection() {
        return false;
    }
    let before = doc.controls.clone();
    let r = doc.controls[idx[0]].clone();
    match op {
        LayoutOp::AlignLeft => idx.iter().for_each(|&i| doc.controls[i].x = r.x),
        LayoutOp::AlignTop => idx.iter().for_each(|&i| doc.controls[i].y = r.y),
        LayoutOp::AlignRight => idx.iter().for_each(|&i| {
            let c = &mut doc.controls[i];
            c.x = r.x + r.width - c.width;
        }),
        LayoutOp::AlignBottom => idx.iter().for_each(|&i| {
            let c = &mut doc.controls[i];
            c.y = r.y + r.height - c.height;
        }),
        LayoutOp::AlignCenterH => idx.iter().for_each(|&i| {
            let c = &mut doc.controls[i];
            c.x = r.x + (r.width - c.width) / 2.0;
        }),
        LayoutOp::AlignCenterV => idx.iter().for_each(|&i| {
            let c = &mut doc.controls[i];
            c.y = r.y + (r.height - c.height) / 2.0;
        }),
        LayoutOp::SameWidth => idx.iter().for_each(|&i| doc.controls[i].width = r.width),
        LayoutOp::SameHeight => idx.iter().for_each(|&i| doc.controls[i].height = r.height),
        LayoutOp::SameSize => idx.iter().for_each(|&i| {
            doc.controls[i].width = r.width;
            doc.controls[i].height = r.height;
        }),
        LayoutOp::DistributeH => distribute(doc, &idx, true),
        LayoutOp::DistributeV => distribute(doc, &idx, false),
    }
    doc.controls != before
}

/// Espacement égal entre les contrôles, les extrémités restant fixes.
fn distribute(doc: &mut FormDocument, idx: &[usize], horizontal: bool) {
    let mut sorted = idx.to_vec();
    let pos = |doc: &FormDocument, i: usize| {
        let c = &doc.controls[i];
        if horizontal { c.x } else { c.y }
    };
    let size = |doc: &FormDocument, i: usize| {
        let c = &doc.controls[i];
        if horizontal { c.width } else { c.height }
    };
    sorted.sort_by(|&a, &b| pos(doc, a).total_cmp(&pos(doc, b)));
    let first = sorted[0];
    let last = *sorted.last().expect("au moins 3 éléments");
    let start = pos(doc, first);
    let end = pos(doc, last) + size(doc, last);
    let total: f32 = sorted.iter().map(|&i| size(doc, i)).sum();
    let gap = (end - start - total) / (sorted.len() - 1) as f32;
    let mut cursor = start;
    for &i in &sorted {
        let s = size(doc, i);
        let c = &mut doc.controls[i];
        if horizontal {
            c.x = cursor;
        } else {
            c.y = cursor;
        }
        cursor += s + gap;
    }
}

/// Déplace les contrôles nommés à la fin de la liste (premier plan) ou au début (arrière-plan).
pub fn reorder(doc: &mut FormDocument, selection: &[String], to_front: bool) -> bool {
    let before: Vec<String> = doc.controls.iter().map(|c| c.name.clone()).collect();
    let (mut sel, rest): (Vec<_>, Vec<_>) = doc
        .controls
        .drain(..)
        .partition(|c| selection.contains(&c.name));
    if to_front {
        doc.controls.extend(rest);
        doc.controls.append(&mut sel);
    } else {
        doc.controls.append(&mut sel);
        doc.controls.extend(rest);
    }
    let after: Vec<String> = doc.controls.iter().map(|c| c.name.clone()).collect();
    before != after
}

/// Arrondit une valeur à la grille.
pub fn snap(value: f32, grid: f32) -> f32 {
    if grid <= 0.0 {
        value
    } else {
        (value / grid).round() * grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComponentKind;

    fn doc() -> (FormDocument, Vec<String>) {
        let mut d = FormDocument::new("Form1");
        let a = d.add_control(ComponentKind::Button, 10.0, 10.0);
        let b = d.add_control(ComponentKind::Button, 50.0, 100.0);
        let c = d.add_control(ComponentKind::TextBox, 300.0, 40.0);
        (d, vec![a, b, c])
    }

    #[test]
    fn align_left_uses_first_selected() {
        let (mut d, sel) = doc();
        assert!(apply(&mut d, &sel, LayoutOp::AlignLeft));
        assert!(d.controls.iter().all(|c| c.x == 10.0));
        assert!(!apply(&mut d, &sel, LayoutOp::AlignLeft));
    }

    #[test]
    fn same_size_and_right() {
        let (mut d, sel) = doc();
        apply(&mut d, &sel, LayoutOp::SameSize);
        assert!(
            d.controls
                .iter()
                .all(|c| c.width == 96.0 && c.height == 28.0)
        );
        apply(&mut d, &sel, LayoutOp::AlignRight);
        assert!(d.controls.iter().all(|c| c.x + c.width == 106.0));
    }

    #[test]
    fn distribute_horizontally() {
        let (mut d, sel) = doc();
        // Tailles égales pour un calcul simple.
        apply(&mut d, &sel, LayoutOp::SameSize);
        assert!(apply(&mut d, &sel, LayoutOp::DistributeH));
        let xs: Vec<f32> = d.controls.iter().map(|c| c.x).collect();
        assert_eq!(xs, vec![10.0, 155.0, 300.0]);
    }

    #[test]
    fn single_selection_does_nothing() {
        let (mut d, sel) = doc();
        assert!(!apply(&mut d, &sel[..1], LayoutOp::AlignTop));
    }

    #[test]
    fn z_order() {
        let (mut d, sel) = doc();
        assert!(reorder(&mut d, &sel[..1], true));
        assert_eq!(d.controls.last().unwrap().name, "button1");
        assert!(reorder(&mut d, &sel[..1], false));
        assert_eq!(d.controls[0].name, "button1");
    }

    #[test]
    fn snapping() {
        assert_eq!(snap(13.0, 8.0), 16.0);
        assert_eq!(snap(11.0, 8.0), 8.0);
        assert_eq!(snap(11.0, 0.0), 11.0);
    }
}
