//! The menu bar: the app's menus with the Drawing menu (design 2.1 §3) before View. Its first two
//! items run (J005): Change Weights at Once… opens its card, Revert to the Legend puts the
//! selected lines back to their row's look, or every line set apart when nothing is selected. The
//! others come with later jobs; until then they show dimmed, each with its job.

use std::cell::Cell;

use serde_json::json;

use super::legend::{Card, open_card, open_coord, open_tablet, open_underlay_align, revert};
use super::*;

/// A Drawing menu item that runs.
#[derive(Clone, Copy)]
enum Item {
    Weights,
    Revert,
    CutFace,
    Coord,
    LevelLines,
    Tablet,
    AlignUnderlay,
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(MENU).exact_size(MENU_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let mut bar =
            ui.new_child(egui::UiBuilder::new().max_rect(r.shrink2(vec2(6.0, 0.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
        let (open, chosen) = (app.session.active().is_some(), Cell::new(None));
        let body = |ui: &mut Ui| drawing_menu(ui, open, &chosen);
        let extra = crate::menus::ExtraMenu { before: "View", title: tw!("Drawing"), body: &body };
        let (_, title) = crate::menus::menu_bar_with(app, &mut bar, 12.0, Some(10.0), Some(&extra));
        // The new menu is underlined across its title, as in the mockup.
        if let Some(tr) = title {
            let under = Rect::from_min_max(pos2(tr.left(), r.center().y + 7.0), pos2(tr.right(), r.center().y + 9.0));
            ui.painter().rect_filled(under, 0.0, t.row_selected);
        }
        match chosen.get() {
            Some(Item::Weights) => open_card(app, ui.ctx(), Card::Weights, None),
            Some(Item::Revert) => {
                let selected = app.session.active().is_some_and(|st| !st.selection.objects.is_empty());
                if let Err(e) = revert(app, if selected { json!({}) } else { json!({ "all": true }) }) {
                    app.status(e);
                }
            }
            Some(Item::CutFace) => match app.run("drawing.cutFace", json!({})) {
                Ok(v) => {
                    let count = v["faces"].as_u64().unwrap_or(0);
                    let lines = v["lines"].as_u64().unwrap_or(0);
                    let msg = if let Some(w) = v["warning"].as_str() {
                        crate::i18n::fmt(
                            tw!("Cut Face: {count} shapes, {lines} hatch lines ({warning})"),
                            &[("count", &count.to_string()), ("lines", &lines.to_string()), ("warning", w)],
                        )
                    } else {
                        crate::i18n::fmt(
                            tw!("Cut Face: {count} shapes, {lines} hatch lines"),
                            &[("count", &count.to_string()), ("lines", &lines.to_string())],
                        )
                    };
                    app.status(msg);
                }
                Err(e) => app.status(e),
            },
            Some(Item::Coord) => open_coord(app, ui.ctx()),
            Some(Item::LevelLines) => match app.run("drawing.makeLevelLines", json!({})) {
                Ok(v) => {
                    let l50 = v["lines_50cm"].as_u64().unwrap_or(0);
                    let l10 = v["lines_10cm"].as_u64().unwrap_or(0);
                    let min_e = v["min_elevation"].as_f64().unwrap_or(0.0);
                    let max_e = v["max_elevation"].as_f64().unwrap_or(0.0);
                    let msg = crate::i18n::fmt(
                        tw!("Level Lines: {l50} major (50 cm), {l10} minor (10 cm) (EL. {min_e:.1} m ~ {max_e:.1} m)"),
                        &[("l50", &l50.to_string()), ("l10", &l10.to_string()), ("min_e", &format!("{min_e:.1}")), ("max_e", &format!("{max_e:.1}"))],
                    );
                    app.status(msg);
                }
                Err(e) => app.status(e),
            },
            Some(Item::Tablet) => open_tablet(app, ui.ctx()),
            Some(Item::AlignUnderlay) => open_underlay_align(app, ui.ctx()),
            None => {}
        }
    });
}

/// The Drawing menu's items, in groups: the items that run (with a drawing open), then the others,
/// each with the job that brings it.
fn drawing_menu(ui: &mut Ui, open: bool, chosen: &Cell<Option<Item>>) {
    let live = [
        (tw!("Change Weights at Once…"), tw!("Change every line's weight at once"), Item::Weights),
        (tw!("Revert to the Legend"), tw!("The selected lines, or every line set apart when nothing is selected"), Item::Revert),
        (tw!("Cut Face"), tw!("Apply 0.8 pt outer border and 0.1 pt 1.0 mm hatch lines to closed shapes"), Item::CutFace),
        (tw!("Scale and Coordinate System…"), tw!("Set drawing scale ratio and Korean national TM coordinate system"), Item::Coord),
        (tw!("Make Level Lines"), tw!("Generate 50 cm and 10 cm elevation level lines"), Item::LevelLines),
        (tw!("Tablet & Pen Settings…"), tw!("Configure stylus sensitivity, pressure curve and tolerances"), Item::Tablet),
        (tw!("Align Underlay…"), tw!("Align scanned drawing to real-world scale and TM coordinates using 2 or 4 GCPs"), Item::AlignUnderlay),
    ];
    for (label, tip, item) in live {
        if ui.add_enabled(open, egui::Button::new(label)).on_hover_text(tip).clicked() {
            chosen.set(Some(item));
            ui.close();
        }
    }
    let groups: [&[(&str, &str)]; 2] =
        [&[(tw!("Insert Legend Table"), "J012"), (tw!("Insert Title Block"), "J012")], &[(tw!("Send to Strata…"), "J019")]];
    for group in groups {
        ui.separator();
        for (label, job) in group.iter() {
            ui.add_enabled(false, egui::Button::new(*label)).on_disabled_hover_text(soon(label, job));
        }
    }
}
