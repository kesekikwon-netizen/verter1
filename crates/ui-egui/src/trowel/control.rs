//! The Control bar. With a drawing tool: the tool, the legend row it draws with (its look, its
//! name and the next line's weight) and the options later jobs bring; with any other tool, the
//! classic Control bar's controls. Snap and its tolerance stay at the right.

use serde_json::json;

use super::*;

/// The tools the Control bar shows the drawing layout for: those that draw lines and shapes. The
/// anchor tools, Smooth and Arc keep the classic Control bar, where their options are.
const DRAWING_TOOLS: &[&str] = &[
    "pen",
    "curvature",
    "pencil",
    "lineSegment",
    "rectangle",
    "roundedRectangle",
    "ellipse",
    "polygon",
    "star",
    "paintbrush",
    "blobBrush",
    "stratum-line",
    "stratum-badge",
    "hearth-cut",
    "rim-arc",
    "centerline",
    "wall-cut",
];

const H: f32 = 28.0;
const GAP: f32 = 10.0;
/// How wide the legend field's list of layers is: a padlock or eye at a row's right stays by its
/// name.
const LIST_W: f32 = 260.0;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(CONTROL).exact_size(CONTROL_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let left = snap_controls(app, ui, r, &t);
        let room = Rect::from_min_max(pos2(r.left() + 12.0, r.top()), pos2(left - GAP, r.bottom()));
        // Either layout is cut at Snap when the window is too narrow for it.
        let mut row = ui.new_child(egui::UiBuilder::new().max_rect(room).layout(egui::Layout::left_to_right(egui::Align::Center)));
        row.set_clip_rect(room);
        if DRAWING_TOOLS.contains(&app.session.tool_id()) && app.session.active().is_some() {
            drawing(app, &mut row, room, &t);
        } else {
            crate::chrome::control_bar_contents(app, &mut row);
        }
    });
}

/// Snap to Point and its tolerance, at the right end of `r`. Returns where they begin.
fn snap_controls(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect, t: &Tokens) -> f32 {
    let cy = r.center().y;
    let value = app.session.prefs.snap_to_point_tolerance.round().clamp(1.0, 8.0);
    let digits = format!("{value:.0}");
    let field_w = (8.0 + width_of(ui, &digits, mono(12.0)) + 6.0 + width_of(ui, "px", font(12.0)) + 8.0).max(56.0);
    let field = Rect::from_min_size(pos2(r.right() - 12.0 - field_w, cy - H / 2.0), vec2(field_w, H));
    if let Some(v) = tolerance_field(ui, field, value, &digits, t) {
        app.run("prefs.set", json!({"key": "snapToPointTolerance", "value": v})).ok();
    }
    let label = text(ui, pos2(field.left() - GAP, cy), Align2::RIGHT_CENTER, tw!("Tolerance"), font(12.0), t.text_dim);
    let snap = toggle_rect(ui, pos2(label.left() - GAP, cy), H, tw!("Snap"));
    let snap = snap.translate(vec2(-snap.width(), 0.0));
    if toggle(ui, wid("ctl-snap", ()), snap, tw!("Snap"), app.ui.view.snap_to_point, t).on_hover_text(tl!("Snap to Point")).clicked() {
        crate::menus::invoke(app, "view.snapToPoint", json!({}));
    }
    snap.left()
}

/// The snapping tolerance in whole pixels (1 to 8) in `r`: its digits, then "px". A click lists the
/// values. Returns the one picked.
fn tolerance_field(ui: &Ui, r: Rect, value: f64, digits: &str, t: &Tokens) -> Option<f64> {
    let resp = ui.interact(r, wid("ctl-tolerance", ()), Sense::click());
    field_box(ui, r, if resp.hovered() { t.panel } else { t.input }, t.input_border);
    let x = text(ui, pos2(r.left() + 8.0, r.center().y), Align2::LEFT_CENTER, digits, mono(12.0), t.text).right() + 6.0;
    text(ui, pos2(x, r.center().y), Align2::LEFT_CENTER, "px", font(12.0), t.text_disabled);
    let mut set = None;
    let resp = resp.on_hover_text(tw!("How near the pointer snaps to a point"));
    egui::Popup::menu(&resp).show(|ui| {
        for v in 1..=8 {
            let v = f64::from(v);
            if ui.selectable_label(v == value, format!("{v:.0} px")).clicked() {
                set = Some(v);
            }
        }
    });
    set
}

/// The drawing tools' layout in `room`.
fn drawing(app: &mut VectorcraftApp, ui: &mut Ui, room: Rect, t: &Tokens) {
    let cy = room.center().y;
    let p = ui.painter().with_clip_rect(room);
    let mut x = room.left();
    let label = |s: &str, color: Color32, x: &mut f32| {
        let r = p.text(pos2(*x, cy), Align2::LEFT_CENTER, s, font(12.0), color);
        *x = r.right() + GAP;
    };
    label(&format!("{} ›", tool_name(app.session.tool_id())), t.text_dim, &mut x);
    label(tw!("Legend"), t.text_dim, &mut x);
    x = layer_field(app, ui, pos2(x, cy), t) + GAP;
    // Estimated lines (Dashed toggle).
    let stroke = crate::panels::current_stroke(app);
    let is_dashed = stroke.as_ref().and_then(|s| s.dash.as_ref()).is_some_and(|d| d.is_dashed());
    let estimate = toggle_rect(ui, pos2(x, cy), H, tw!("Estimated (Dashed)"));
    if toggle(ui, wid("ctl-estimate", ()), estimate, tw!("Estimated (Dashed)"), is_dashed, t)
        .on_hover_text(tw!("Toggle dashed stroke for estimated lines"))
        .clicked()
    {
        let new_dash: Option<Vec<f64>> = if is_dashed { None } else { Some(vec![4.0, 2.0]) };
        crate::panels::stroke::set(app, json!({ "dash": new_dash }));
    }
    x = estimate.right() + GAP;
    x = divider(ui, x, cy, t);
    let mut x2 = x;
    label(tw!("When Closed"), t.text_dim, &mut x2);
    x = when_closed_dropdown(ui, pos2(x2, cy), t) + GAP;
    x = divider(ui, x, cy, t);
    let mut x2 = x;
    label(tw!("Pen Pressure"), t.text_dim, &mut x2);
    x = pressure_segments(app, ui, pos2(x2, cy), t) + GAP;
    x = divider(ui, x, cy, t);
    let tool_id = app.session.tool_id();
    if tool_id == "stratum-badge" || tool_id == "stratum-line" || tool_id == "hearth-cut" {
        let btn_text = tw!("Strata Table");
        let btn_w = width_of(ui, btn_text, font(12.0)) + 16.0;
        let btn_rect = Rect::from_min_size(pos2(x, cy - H / 2.0), vec2(btn_w, H));
        let resp = ui.interact(btn_rect, wid("ctl-strata-table", ()), Sense::click());
        let fill = if resp.hovered() { t.hover } else { t.panel_darker };
        ui.painter().rect(btn_rect, CornerRadius::same(6), fill, Stroke::new(1.0, t.border), StrokeKind::Inside);
        text(ui, btn_rect.center(), Align2::CENTER_CENTER, btn_text, font(12.0), t.text);
        if resp.on_hover_text(tw!("Generate Three-Line Strata Explanation Table")).clicked() {
            app.run("drawing.generateStrataTable", json!({})).ok();
        }
        x = btn_rect.right() + GAP;
        x = divider(ui, x, cy, t);
    }
    if tool_id == "rim-arc" || tool_id == "centerline" || tool_id == "wall-cut" {
        let btn_text = tw!("Dimension Lines");
        let btn_w = width_of(ui, btn_text, font(12.0)) + 16.0;
        let btn_rect = Rect::from_min_size(pos2(x, cy - H / 2.0), vec2(btn_w, H));
        let resp = ui.interact(btn_rect, wid("ctl-artifact-dim", ()), Sense::click());
        let fill = if resp.hovered() { t.hover } else { t.panel_darker };
        ui.painter().rect(btn_rect, CornerRadius::same(6), fill, Stroke::new(1.0, t.border), StrokeKind::Inside);
        text(ui, btn_rect.center(), Align2::CENTER_CENTER, btn_text, font(12.0), t.text);
        if resp.on_hover_text(tw!("Generate Vessel Dimension Lines (Rim, Base, Height)")).clicked() {
            app.run(
                "artifact.generateDimensionLines",
                json!({
                    "centerline_x": 300.0,
                    "rim_y": 100.0,
                    "base_y": 350.0,
                    "rim_radius": 141.73228,
                    "base_radius": 70.86614
                }),
            )
            .ok();
        }
        x = btn_rect.right() + GAP;
        x = divider(ui, x, cy, t);
    }
    p.text(pos2(x, cy), Align2::LEFT_CENTER, tw!("Weight, hatch and line type: in the Drawing Palette at the right"), font(12.0), t.text_disabled);
}

/// A one-point upright line between groups; returns where the next one starts.
fn divider(ui: &Ui, x: f32, cy: f32, t: &Tokens) -> f32 {
    ui.painter().vline(x + 0.5, (cy - 10.0)..=(cy + 10.0), Stroke::new(1.0, t.border));
    x + 1.0 + GAP
}

/// What stands before a layer's name in the legend field and its list: its row's look, or the
/// layer's colour (`edge` thick) when it is not a legend row.
fn look_patch(ui: &Ui, r: Rect, doc: &vectorcraft_doc::Document, id: vectorcraft_doc::NodeId, edge: f32, t: &Tokens) {
    match doc.legend(id) {
        Some(legend) => super::legend::sample(ui, r, legend, t),
        None => {
            let c = crate::panels::layers::layer_colour(doc, id);
            ui.painter().rect(r, 0.0, c.gamma_multiply(0.25), Stroke::new(edge, c), StrokeKind::Inside);
        }
    }
}

/// The legend field: the look (or colour) and name of the layer the next line goes to and its
/// weight; a click lists the rows to draw with in the legend's order, and picking one gives the pen
/// its look (`legend.pick`, the selection staying where it is), or says why a hidden or locked one
/// isn't. When the current layer is locked or hidden it shows the layer that takes the line
/// instead, edged in clay, and says why on hover. Returns where it ends.
fn layer_field(app: &mut VectorcraftApp, ui: &mut Ui, at: Pos2, t: &Tokens) -> f32 {
    let Some(st) = app.session.active() else { return at.x };
    let current = st.current_layer();
    let target = draw_target(st);
    let name = match target.into.and_then(|id| st.doc.node(id)) {
        Some(n) => n.display_name(),
        None => tw!("No layer to draw on").to_string(),
    };
    // With no layer to draw on there is no look or weight to show: the field says so from its left.
    let weight = match target.into {
        Some(_) => {
            let stroke = crate::panels::current_stroke(app);
            let mixed = crate::panels::stroke_mixed(app, ui.ctx());
            crate::panels::stroke::shown_weight(app, stroke.as_ref(), &mixed).map(|w| app.session.stroke_unit().format(w)).unwrap_or_default()
        }
        None => String::new(),
    };
    let lead = if target.into.is_some() { 20.0 + 6.0 } else { 0.0 };
    let name_w = width_of(ui, &name, font(12.0)).min(220.0);
    let weight_w = width_of(ui, &weight, mono(12.0));
    let w = (8.0 + lead + name_w + 6.0 + weight_w + 8.0 + 8.0 + 8.0).max(196.0);
    let r = Rect::from_min_size(pos2(at.x, at.y - H / 2.0), vec2(w, H));
    let resp = ui.interact(r, wid("ctl-layer", ()), Sense::click());
    field_box(ui, r, if resp.hovered() { t.panel } else { t.input }, if target.note.is_some() { t.clay } else { t.input_border });
    let patch = Rect::from_min_size(pos2(r.left() + 8.0, at.y - 6.0), vec2(20.0, 12.0));
    if let (Some(into), Some(st)) = (target.into, app.session.active()) {
        look_patch(ui, patch, &st.doc, into, 2.0, t);
    }
    let p = ui.painter().with_clip_rect(r);
    let x = r.left() + 8.0 + lead;
    let n = p.text(pos2(x, at.y), Align2::LEFT_CENTER, &name, font(12.0), t.text);
    if !weight.is_empty() {
        p.text(pos2(n.right().min(x + name_w) + 6.0, at.y), Align2::LEFT_CENTER, &weight, mono(12.0), t.text_disabled);
    }
    triangle(ui, pos2(r.right() - 8.0 - 4.0, at.y), 8.0, true, t.text_dim);
    let resp = resp.on_hover_text(target.note.as_deref().unwrap_or(tw!("The layer and weight the next line is drawn with")));
    let mut pick = None;
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(LIST_W);
        if let Some(st) = app.session.active() {
            // In the legend's order: its sections, favourites first in each.
            for layer in super::legend::sections(&st.doc).into_iter().flat_map(|(_, layers)| layers) {
                let (row, _) = ui.allocate_exact_size(vec2(LIST_W, 24.0), Sense::hover());
                let rr = ui.interact(row, wid("ctl-layer-row", layer.id.0), Sense::click());
                if rr.hovered() {
                    ui.painter().rect_filled(row, CornerRadius::same(4), t.hover);
                }
                let patch = Rect::from_min_size(pos2(row.left() + 6.0, row.center().y - 6.0), vec2(20.0, 12.0));
                look_patch(ui, patch, &st.doc, layer.id, 1.5, t);
                let f = if Some(layer.id) == current { bold(12.0) } else { font(12.0) };
                // A locked or hidden layer takes no lines: dimmed, with why at the right.
                let editable = st.doc.is_editable(layer.id);
                let name = Rect::from_min_max(pos2(patch.right() + 8.0, row.top()), pos2(row.right() - 28.0, row.bottom()));
                ui.painter().with_clip_rect(name).text(
                    pos2(patch.right() + 8.0, row.center().y),
                    Align2::LEFT_CENTER,
                    layer.display_name(),
                    f,
                    if editable { t.text } else { t.text_dim },
                );
                let why = if !layer.visible {
                    Some("eye-off")
                } else if layer.locked {
                    Some("lock")
                } else {
                    None
                };
                if let Some(icon) = why {
                    crate::icons::paint(ui, icon, Rect::from_center_size(pos2(row.right() - 14.0, row.center().y), vec2(13.0, 13.0)), t.text_dim);
                }
                if rr.clicked() {
                    pick = Some(layer.id);
                    ui.close();
                }
            }
        }
    });
    if let Some(id) = pick {
        // A hidden or locked row isn't drawn with: say why.
        let done = match app.session.active().and_then(|st| super::legend::refused(st, id)) {
            Some(why) => Err(why),
            None => app.run("legend.pick", json!({ "id": id.0 })).map(drop),
        };
        if let Err(e) = done {
            app.status(e);
        }
    }
    r.right()
}

/// J009: Drop-down for polygon closure action: Area and Perimeter, Area Only, Perimeter Only, Off.
fn when_closed_dropdown(ui: &mut Ui, at: Pos2, t: &Tokens) -> f32 {
    let mode_id = wid("ctl-closed-mode", ());
    let mode = ui.data(|d| d.get_temp::<usize>(mode_id)).unwrap_or(0);
    let options = [tw!("Area and Perimeter"), tw!("Area Only"), tw!("Perimeter Only"), tw!("Off")];
    let cur_label = options.get(mode).copied().unwrap_or(options[0]);
    let w = width_of(ui, cur_label, font(12.0)) + 26.0;
    let r = Rect::from_min_size(pos2(at.x, at.y - H / 2.0), vec2(w, H));
    let resp = ui.interact(r, wid("ctl-closed", ()), Sense::click());
    field_box(ui, r, if resp.hovered() { t.panel } else { t.input }, t.input_border);
    text(ui, pos2(r.left() + 8.0, at.y), Align2::LEFT_CENTER, cur_label, font(12.0), t.text);
    triangle(ui, pos2(r.right() - 8.0 - 4.0, at.y), 8.0, true, t.text_dim);
    let resp = resp.on_hover_text(tw!("Calculate plan metrics when closing polygons"));

    let mut set_mode = None;
    egui::Popup::menu(&resp).show(|ui| {
        for (i, opt) in options.iter().enumerate() {
            let row = ui.selectable_label(i == mode, *opt);
            if row.clicked() {
                set_mode = Some(i);
            }
        }
    });
    if let Some(m) = set_mode {
        ui.data_mut(|d| d.insert_temp(mode_id, m));
    }
    r.right()
}

/// J016: Interactive segmented choice for Pen Pressure: Same Weight vs Follow Pressure.
fn pressure_segments(app: &mut VectorcraftApp, ui: &mut Ui, at: Pos2, t: &Tokens) -> f32 {
    let follow = app.session.prefs.pen_pressure_follow;
    let labels = [tw!("Same Weight"), tw!("Follow Pressure")];
    let widths: Vec<f32> = labels.iter().map(|l| width_of(ui, l, font(12.0)) + 18.0).collect();
    let r = Rect::from_min_size(pos2(at.x, at.y - H / 2.0), vec2(widths.iter().sum(), H));
    field_box(ui, r, t.input, t.input_border);
    let mut x = r.left();
    let mut set_follow = None;
    for (i, (label, w)) in labels.iter().zip(&widths).enumerate() {
        let seg = Rect::from_min_size(pos2(x, r.top()), vec2(*w, H));
        let is_selected = if i == 0 { !follow } else { follow };
        let resp = ui.interact(seg, wid("ctl-pressure-seg", i), Sense::click());
        if resp.hovered() && !is_selected {
            ui.painter().rect_filled(seg.shrink(1.0), CornerRadius::same(7), t.hover);
        }
        if is_selected {
            ui.painter().rect(seg.shrink(1.0), CornerRadius::same(7), t.row_selected, Stroke::new(1.0, t.text), StrokeKind::Inside);
        } else if i > 0 {
            ui.painter().vline(seg.left() + 0.5, seg.y_range(), Stroke::new(1.0, t.border));
        }
        let txt_color = if is_selected || resp.hovered() { t.text } else { t.text_dim };
        text(ui, seg.center(), Align2::CENTER_CENTER, label, font(12.0), txt_color);
        let tip = if i == 0 {
            tw!("Uniform line weight for archaeological drafting standards")
        } else {
            tw!("Variable line weight scaling with stylus pen pressure")
        };
        if resp.on_hover_text(tip).clicked() {
            set_follow = Some(i == 1);
        }
        x += w;
    }
    if let Some(f) = set_follow {
        app.session.prefs.pen_pressure_follow = f;
        app.run("prefs.set", json!({ "key": "penPressureFollow", "value": f })).ok();
    }
    r.right()
}
