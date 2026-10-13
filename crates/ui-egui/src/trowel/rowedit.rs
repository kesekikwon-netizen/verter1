//! A legend row's editor (design 2.1 §7, figure ②): the card a double click on a row opens beside
//! the right column. The row's name, weight (a chip, or points typed, with its paper millimetres
//! beside), line type and ★ are set with `legend.set`, which restyles the row's lines that
//! followed it; lines set apart keep their look until put back. Its lines can be selected
//! together, and Layer Options… opens the layer's own options. Choosing the Strata layer (J019),
//! estimated lines (J009), hatches (J014), labels (J012) and where a row is kept (J004, J018) come
//! with later jobs.

use serde_json::{Value, json};
use vectorcraft_doc::legend::{MM, SAME, lines, stroke_of};
use vectorcraft_doc::{Hatch, Legend, LineType, Node, NodeId};
use vectorcraft_engine::cmd::legend::LADDER;

use super::card::{self, GAP_X};
use super::legend::{badge, pt};
use super::*;

/// Strata's survey layers: key and name (hgis `data/schemas/ka_hgis_layers.yaml`, `name_ko`).
const STRATA: [(&str, &str); 7] = [
    ("survey_area", "조사구역"),
    ("feature_poly", "유구_면"),
    ("feature_line", "유구_선"),
    ("section_line", "단면선"),
    ("control_points", "기준점"),
    ("artifact_point", "유물"),
    ("trial_trench", "시굴격자"),
];

/// The rows' height.
const ROW_H: f32 = 24.0;
const SEG_H: f32 = 24.0;

/// What the editor asks for.
enum Ask {
    /// `legend.set` with these params, the row's id aside.
    Set(Value),
    Rename(String),
    Select,
    Revert,
    Options,
    /// Points typed that aren't a weight.
    NotAWeight,
}

/// The row as the editor shows it.
struct Row {
    name: String,
    legend: Legend,
    locked: bool,
    /// Its lines, top first, and those set apart from its look.
    lines: Vec<NodeId>,
    apart: Vec<NodeId>,
}

impl Row {
    fn of(layer: &Node) -> Row {
        let all = lines(layer);
        let apart = match layer.legend() {
            Some(l) => all.iter().filter(|line| stroke_of(line).is_some_and(|s| l.differs(s))).map(|line| line.id).collect(),
            None => vec![],
        };
        Row {
            name: layer.display_name(),
            legend: layer.legend().cloned().unwrap_or_default(),
            locked: layer.locked,
            lines: all.iter().map(|line| line.id).collect(),
            apart,
        }
    }
}

/// Show the editor of row `id` at `place` ([`card::show`]). Returns whether it stays open: not when
/// the row is gone.
pub(super) fn show(app: &mut VectorcraftApp, ctx: &egui::Context, place: card::Place, id: NodeId) -> bool {
    let row = match app.session.active() {
        Some(st) => match st.doc.node(id) {
            Some(n) if n.is_layer() && st.doc.parent_of(id).is_none() => Row::of(n),
            _ => return false,
        },
        None => return false,
    };
    let (mut open, asks) = card::show(ctx, wid("row-card", ()), place, (tw!("Legend Row"), &row.name), |ui, t| body(ui, &row, t));
    let ids = |v: &[NodeId]| v.iter().map(|n| n.0).collect::<Vec<_>>();
    for ask in asks {
        let done = match ask {
            Ask::Set(mut p) => {
                p["id"] = json!(id.0);
                app.run("legend.set", p).map(|done| {
                    let n = done["lines"].as_u64().unwrap_or(0);
                    if n > 0 {
                        app.status(crate::i18n::fmt(tw!("Lines changed: {n}"), &[("n", &n.to_string())]));
                    }
                })
            }
            Ask::Rename(name) => app.run("layer.setProps", json!({ "ids": [id.0], "name": name })).map(drop),
            Ask::Select => app.run("select.set", json!({ "ids": ids(&row.lines) })).map(drop),
            Ask::Revert => super::legend::revert(app, json!({ "ids": ids(&row.apart) })),
            Ask::Options => {
                // The layer's options take over from the card.
                crate::menus::invoke(app, "ui.layerOptions", json!({ "ids": [id.0] }));
                open = false;
                Ok(())
            }
            Ask::NotAWeight => Err(tw!("Type a weight in points, such as 0.5").to_string()),
        };
        if let Err(e) = done {
            app.status(e);
        }
    }
    open
}

/// The editor's rows.
fn body(ui: &mut Ui, row: &Row, t: &Tokens) -> Vec<Ask> {
    let mut asks = vec![];
    if row.locked {
        card::note(ui, tw!("This row's layer is locked: unlock it to change the row"), t.caution);
    }
    ui.scope(|ui| {
        if row.locked {
            ui.disable();
        }
        look(ui, row, t, &mut asks);
        later(ui, row, t, &mut asks);
        // ★.
        let r = card::row(ui, "★", ROW_H, t);
        let fav = toggle_rect(ui, r.left_center(), ROW_H, tw!("Often Used"));
        let fav = toggle(ui, wid("card-favourite", ()), fav, tw!("Often Used"), row.legend.favourite, t);
        if fav.on_hover_text(tw!("Shown first in its section, its name in clay")).clicked() {
            asks.push(Ask::Set(json!({ "favourite": !row.legend.favourite })));
        }
        lines_of(ui, row, t, &mut asks);
        saved_in(ui, t);
    });
    let places = card::flow(ui, 28.0, &[card::button_w(ui, tw!("Layer Options…"))]);
    if let Some(r) = places.first()
        && card::button(ui, wid("card-options", ()), *r, tw!("Layer Options…"), true, t).clicked()
    {
        asks.push(Ask::Options);
    }
    asks
}

/// Name, Strata, weight and line type.
fn look(ui: &mut Ui, row: &Row, t: &Tokens, asks: &mut Vec<Ask>) {
    let legend = &row.legend;
    let r = card::row(ui, tw!("Name"), ROW_H, t);
    if let Some(name) = card::text_field(ui, wid("card-name", ()), r, &row.name, font(11.0), ("", ""), t).filter(|n| !n.is_empty()) {
        asks.push(Ask::Rename(name));
    }
    // The Strata layer it matches: shown here, chosen with J019.
    let r = card::row(ui, tw!("Strata"), ROW_H, t);
    let strata = match legend.strata.as_deref() {
        Some(key) => match STRATA.iter().find(|(k, _)| *k == key) {
            Some((k, name)) => format!("{name} · {k}"),
            None => key.to_string(),
        },
        None => tw!("None").to_string(),
    };
    card::field_soon(ui, wid("card-strata", ()), r, &strata, true, (tw!("Choose a Strata Layer"), "J019"), t);
    let r = card::row(ui, "", 18.0, t);
    text(ui, r.left_center(), Align2::LEFT_CENTER, tw!("This row is the layer"), font(12.0), t.text_dim);
    // Weight: a palette weight, points typed, or none. A chip pressed as the typing ends wins, then
    // the none toggle: what was typed only counts on its own.
    let chips: Vec<String> = LADDER.iter().map(|w| pt(*w)).collect();
    let parts: Vec<Seg> = LADDER
        .iter()
        .zip(&chips)
        .map(|(w, label)| Seg { key: label, label, on: legend.weight.is_some_and(|x| (x - w).abs() <= SAME), soon: None })
        .collect();
    let r = card::row(ui, tw!("Weight"), SEG_H, t);
    let seg = SegLook { h: SEG_H, pad: 6.0, font: mono(11.0) };
    let chip = segments(ui, "card-weight", r.left_center(), &seg, &parts, t).0.and_then(card::points);
    let r = card::row(ui, "", ROW_H, t);
    let field = Rect::from_min_size(r.min, vec2(76.0, r.height()));
    let shown = legend.weight.map(pt).unwrap_or_default();
    let typed = card::text_field(ui, wid("card-pt", ()), field, &shown, mono(11.0), ("", "pt"), t);
    if let Some(w) = legend.weight {
        // Paper millimetres: shown only, the row keeps points.
        let mm = crate::i18n::fmt(tw!("= {mm} mm"), &[("mm", &format!("{:.2}", w / MM))]);
        text(ui, pos2(field.right() + GAP_X, r.center().y), Align2::LEFT_CENTER, mm, font(12.0), t.text_dim);
    }
    let none = toggle_rect(ui, r.right_center(), ROW_H, tw!("No weight"));
    let none = toggle(ui, wid("card-weight", "none"), none.translate(vec2(-none.width(), 0.0)), tw!("No weight"), legend.weight.is_none(), t);
    let none = none.on_hover_text(tw!("The row leaves its lines' weights alone")).clicked() && legend.weight.is_some();
    let set = match (chip, none, typed) {
        (Some(w), _, _) => legend.weight.is_none_or(|x| (x - w).abs() > SAME).then(|| Ask::Set(json!({ "weight": w }))),
        (None, true, _) => Some(Ask::Set(json!({ "weight": null }))),
        (None, false, Some(typed)) => Some(match card::points(&typed) {
            Some(w) => Ask::Set(json!({ "weight": w })),
            None => Ask::NotAWeight,
        }),
        (None, false, None) => None,
    };
    asks.extend(set);
    // Line type; a pattern of its own shows none of them, and says so.
    let kinds = line_types();
    let parts: Vec<Seg> = kinds.iter().map(|(key, label, kind)| Seg { key, label, on: legend.line.same(kind), soon: None }).collect();
    let r = card::row(ui, tw!("Line"), SEG_H, t);
    let seg = SegLook { h: SEG_H, pad: 6.0, font: font(11.0) };
    // A row without a weight has no look: its line types wait for one.
    let mut types = ui.new_child(egui::UiBuilder::new().max_rect(r));
    if legend.weight.is_none() {
        types.disable();
    }
    let (pressed, end) = segments(&types, "card-line", r.left_center(), &seg, &parts, t);
    if legend.weight.is_none() {
        // Dimmed: why, on hover.
        let all = Rect::from_min_max(r.min, pos2(end, r.bottom()));
        types
            .interact(all, wid("card-line-why", ()), Sense::hover())
            .on_disabled_hover_text(tw!("A row without a weight has no line type: give it a weight first"));
    }
    if let Some(key) = pressed
        && parts.iter().any(|p| p.key == key && !p.on)
    {
        asks.push(Ask::Set(json!({ "line": key })));
    }
    if matches!(legend.line, LineType::Custom(_)) {
        text(ui, pos2(end + GAP_X, r.center().y), Align2::LEFT_CENTER, tw!("Custom"), font(11.0), t.text_dim);
    }
}

/// What later jobs bring to a row: estimated lines (J009), hatches (J014) and its label (J012).
fn later(ui: &mut Ui, row: &Row, t: &Tokens, asks: &mut Vec<Ask>) {
    let r = card::row(ui, tw!("Estimated"), ROW_H, t);
    let estimated = Rect::from_min_size(r.min, vec2(132.0, r.height()));
    card::field_soon(ui, wid("card-estimated", ()), estimated, tw!("Same weight, dashed"), true, (tw!("Estimated Lines"), "J009"), t);
    let h = row.legend.hatch;
    let parts = vec![
        Seg { key: "none", label: tw!("None"), on: h == Hatch::None, soon: None },
        Seg { key: "45", label: "45°", on: h == Hatch::Deg45, soon: None },
        Seg { key: "135", label: "135°", on: h == Hatch::Deg135, soon: None },
        Seg { key: "alt", label: tw!("Alternate"), on: h == Hatch::Alternate, soon: None },
    ];
    let r = card::row(ui, tw!("Hatch"), SEG_H, t);
    let (pressed, _) = segments(ui, "card-hatch", r.left_center(), &SegLook { h: SEG_H, pad: 6.0, font: font(11.0) }, &parts, t);
    if let Some(key) = pressed
        && parts.iter().any(|p| p.key == key && !p.on)
    {
        asks.push(Ask::Set(json!({ "hatch": key })));
    }
    let r = card::row(ui, tw!("Text"), ROW_H, t);
    let size = Rect::from_min_size(r.min, vec2(70.0, r.height()));
    card::field_soon(ui, wid("card-text", ()), size, "9 pt", false, (tw!("Text Size"), "J012"), t);
    text(ui, pos2(size.right() + GAP_X, r.center().y), Align2::LEFT_CENTER, tw!("Name label"), font(12.0), t.text_dim);
}

/// The row's lines: how many, how many are set apart, Select All and Back to the Legend.
fn lines_of(ui: &mut Ui, row: &Row, t: &Tokens, asks: &mut Vec<Ask>) {
    card::caption(ui, tw!("This row's lines"), t);
    let count = crate::i18n::fmt(tw!("Lines: {n}"), &[("n", &row.lines.len().to_string())]);
    let apart = crate::i18n::fmt(tw!("Apart {n}"), &[("n", &row.apart.len().to_string())]);
    let mut widths = vec![width_of(ui, &count, mono(12.0))];
    if !row.apart.is_empty() {
        widths.push(width_of(ui, &apart, font(10.0)) + 8.0);
    }
    widths.extend([card::button_w(ui, tw!("Select All")), card::button_w(ui, tw!("Back to the Legend"))]);
    let mut places = card::flow(ui, ROW_H, &widths).into_iter();
    if let Some(r) = places.next() {
        text(ui, r.left_center(), Align2::LEFT_CENTER, &count, mono(12.0), t.text);
    }
    if !row.apart.is_empty()
        && let Some(r) = places.next()
    {
        badge(ui, r.right_center(), &apart, t);
    }
    if let Some(r) = places.next()
        && card::button(ui, wid("card-select", ()), r, tw!("Select All"), !row.lines.is_empty(), t)
            .on_hover_text(tw!("Select this row's lines"))
            .clicked()
    {
        asks.push(Ask::Select);
    }
    let tip = if row.apart.is_empty() { tw!("No line is set apart") } else { tw!("Put this row's lines set apart back to its look") };
    if let Some(r) = places.next()
        && card::button(ui, wid("card-revert", ()), r, tw!("Back to the Legend"), !row.apart.is_empty(), t).on_hover_text(tip).clicked()
    {
        asks.push(Ask::Revert);
    }
}

/// Where the row is kept: in the drawing for now; a survey's rows (J004), the user's defaults and
/// template files (J018) come later.
fn saved_in(ui: &mut Ui, t: &Tokens) {
    card::caption(ui, tw!("Saved in"), t);
    let places = [("survey", tw!("This Survey Only"), "J004"), ("defaults", tw!("My Defaults"), "J018"), ("template", tw!("Template File"), "J018")];
    let widths: Vec<f32> = places.iter().map(|(_, label, _)| toggle_rect(ui, Pos2::ZERO, ROW_H, label).width()).collect();
    for ((key, label, job), r) in places.iter().zip(card::flow(ui, ROW_H, &widths)) {
        toggle_soon(ui, wid("card-saved", *key), r, label, job, t);
    }
    card::note(ui, tw!("Saved in this drawing file for now"), t.text_dim);
}
