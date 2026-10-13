//! Scale & TM Coordinate Reference System Card (Trowel, design 2.1 §7, blueprint Section 4):
//! Configures archaeological scale (1:10, 1:20, 1:40, 1:50, 1:100, 1:1), Korean national TM
//! coordinate reference systems (EPSG:5186 Central Origin, EPSG:5187 Eastern Origin),
//! stroke weight to paper mm conversion, and real-world grid origin coordinates.

use serde::{Deserialize, Serialize};
use serde_json::json;
use vectorcraft_doc::DocCoord;

use super::card::{self, GAP_X, button, primary, text_field};
use super::*;

const ROW_H: f32 = 24.0;
const SEG_H: f32 = 24.0;

/// Currently edited settings in the card.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CoordRule {
    pub scale: f64,
    pub epsg: u32,
    pub origin_x: f64,
    pub origin_y: f64,
    pub elevation_datum: String,
}

impl Default for CoordRule {
    fn default() -> Self {
        Self { scale: 20.0, epsg: 5186, origin_x: 148_000.0, origin_y: 98_150.0, elevation_datum: "해발".to_string() }
    }
}

fn rule_id() -> egui::Id {
    wid("coord-rule", ())
}

/// Start the card over: opened again, it shows the document's current values.
pub(super) fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<CoordRule>(rule_id()));
}

/// Show the Scale & Coordinate System card at `place`. Returns whether it stays open.
pub(super) fn show(app: &mut VectorcraftApp, ctx: &egui::Context, place: card::Place) -> bool {
    let Some(st) = app.session.active() else { return false };
    let mut rule = ctx.data(|d| d.get_temp::<CoordRule>(rule_id())).unwrap_or_else(|| {
        if let Some(coord) = &st.doc.coord {
            CoordRule {
                scale: coord.scale,
                epsg: coord.epsg,
                origin_x: coord.origin_x,
                origin_y: coord.origin_y,
                elevation_datum: coord.elevation_datum.clone(),
            }
        } else {
            CoordRule::default()
        }
    });

    let title = tw!("Scale & TM Coordinate System");
    let sub = DocCoord::new(rule.scale, rule.epsg, rule.origin_x, rule.origin_y).summary_label();
    let (mut open, asks) = card::show(ctx, wid("coord-card", ()), place, (title, &sub), |ui, t| body(ui, &mut rule, t));

    for ask in asks {
        match ask {
            Ask::Apply => {
                let params = json!({
                    "scale": rule.scale,
                    "epsg": rule.epsg,
                    "origin_x": rule.origin_x,
                    "origin_y": rule.origin_y,
                    "datum": rule.elevation_datum,
                });
                match app.run("drawing.setCoord", params) {
                    Ok(_) => {
                        let coord = DocCoord::new(rule.scale, rule.epsg, rule.origin_x, rule.origin_y);
                        app.status(coord.summary_label());
                        open = false;
                    }
                    Err(e) => app.status(e),
                }
            }
            Ask::Close => open = false,
        }
    }

    if open {
        ctx.data_mut(|d| d.insert_temp(rule_id(), rule));
    } else {
        forget(ctx);
    }

    open
}

enum Ask {
    Apply,
    Close,
}

fn body(ui: &mut Ui, rule: &mut CoordRule, t: &Tokens) -> Vec<Ask> {
    let mut asks = vec![];

    // 1. Longitude Ruler
    longitude_ruler(ui, rule, t);
    ui.add_space(6.0);

    // 2. CRS selection
    crs_choice(ui, rule, t);
    ui.add_space(6.0);

    // 3. Scale selection
    scale_choice(ui, rule, t);
    ui.add_space(6.0);

    // 4. Stroke weight conversion preview
    stroke_preview(ui, t);
    ui.add_space(6.0);

    // 5. Origin Coordinates
    origin_fields(ui, rule, t);
    ui.add_space(8.0);

    // 6. Action buttons
    let b_w = card::button_w(ui, tw!("Apply"));
    let c_w = card::button_w(ui, tw!("Cancel"));
    let places = card::flow(ui, 28.0, &[b_w, c_w]);
    if let Some(r) = places.first()
        && primary(ui, wid("coord-apply", ()), *r, tw!("Apply"), true, t).clicked()
    {
        asks.push(Ask::Apply);
    }
    if let Some(r) = places.get(1)
        && button(ui, wid("coord-close", ()), *r, tw!("Cancel"), true, t).clicked()
    {
        asks.push(Ask::Close);
    }

    asks
}

/// Longitude ruler spanning 125° to 131° with 127° (Central) and 129° (Eastern) highlighted.
fn longitude_ruler(ui: &mut Ui, rule: &mut CoordRule, t: &Tokens) {
    let r = card::row(ui, tw!("CRS Ruler"), 36.0, t);
    field_box(ui, r, t.input, t.input_border);

    let marks = [
        ("125°", 125, false, 0),
        ("126°", 126, false, 0),
        ("127° (Central)", 127, true, 5186),
        ("128°", 128, false, 0),
        ("129° (Eastern)", 129, true, 5187),
        ("130°", 130, false, 0),
        ("131°", 131, false, 0),
    ];

    let count = marks.len();
    let step = r.width() / count as f32;

    for (i, (label, _deg, major, target_epsg)) in marks.into_iter().enumerate() {
        let mx = r.left() + (i as f32 + 0.5) * step;
        let mark_rect = Rect::from_center_size(pos2(mx, r.center().y), vec2(step, r.height()));
        let resp = ui.interact(mark_rect, wid("coord-ruler-mark", i), Sense::click());

        if target_epsg != 0 && resp.clicked() {
            rule.epsg = target_epsg;
        }

        let is_cur = rule.epsg == target_epsg && target_epsg != 0;
        let color = if is_cur {
            t.clay
        } else if major {
            t.clay_text
        } else {
            t.text_dim
        };

        let font_id = if major || is_cur { bold(9.5) } else { font(9.5) };
        text(ui, pos2(mx, r.top() + 8.0), Align2::CENTER_TOP, label, font_id, color);

        // Tick mark
        let tick_h = if major || is_cur { 8.0 } else { 4.0 };
        ui.painter().vline(mx, (r.bottom() - tick_h)..=r.bottom(), Stroke::new(1.0, color));
    }
}

/// EPSG segmented button choice.
fn crs_choice(ui: &mut Ui, rule: &mut CoordRule, t: &Tokens) {
    let r = card::row(ui, tw!("Origin"), SEG_H, t);
    let parts = [
        Seg { key: "5186", label: tw!("EPSG:5186 Central (Rec.)"), on: rule.epsg == 5186, soon: None },
        Seg { key: "5187", label: tw!("EPSG:5187 Eastern"), on: rule.epsg == 5187, soon: None },
    ];
    let seg = SegLook { h: SEG_H, pad: 6.0, font: font(10.5) };
    if let (Some(k), _) = segments(ui, "coord-epsg", r.left_center(), &seg, &parts, t) {
        if k == "5186" {
            rule.epsg = 5186;
        } else if k == "5187" {
            rule.epsg = 5187;
        }
    }
}

/// Scale segmented button choice.
fn scale_choice(ui: &mut Ui, rule: &mut CoordRule, t: &Tokens) {
    let r = card::row(ui, tw!("Scale"), SEG_H, t);
    let scales = [("10", "1:10", 10.0), ("20", "1:20", 20.0), ("40", "1:40", 40.0), ("50", "1:50", 50.0), ("100", "1:100", 100.0), ("1", "1:1", 1.0)];
    let parts: Vec<Seg> = scales.iter().map(|(k, l, s)| Seg { key: k, label: l, on: (rule.scale - s).abs() < 1e-4, soon: None }).collect();
    let seg = SegLook { h: SEG_H, pad: 5.0, font: mono(11.0) };
    if let (Some(k), _) = segments(ui, "coord-scale", r.left_center(), &seg, &parts, t)
        && let Some((_, _, s)) = scales.iter().find(|(key, _, _)| *key == k)
    {
        rule.scale = *s;
    }
}

/// Stroke weight to paper mm conversion preview box.
fn stroke_preview(ui: &mut Ui, t: &Tokens) {
    let r = card::row(ui, tw!("Paper Stroke"), 46.0, t);
    field_box(ui, r, t.panel_darker, t.input_border);

    let lines = ["0.8 pt 굴광선 = 종이 실측 0.28 mm", "0.5 pt 바닥선 = 종이 실측 0.18 mm", "0.2 pt 토층선 = 종이 실측 0.07 mm"];

    let p = ui.painter().with_clip_rect(r.shrink(2.0));
    for (i, line) in lines.iter().enumerate() {
        let y = r.top() + 7.0 + i as f32 * 13.0;
        p.text(pos2(r.left() + 8.0, y), Align2::LEFT_TOP, *line, font(10.0), t.text_dim);
    }
}

/// Real-world TM origin coordinate fields.
fn origin_fields(ui: &mut Ui, rule: &mut CoordRule, t: &Tokens) {
    let r = card::row(ui, tw!("Origin (m)"), ROW_H, t);
    let half_w = (r.width() - GAP_X) / 2.0;

    // Easting X
    let r_x = Rect::from_min_size(r.min, vec2(half_w, r.height()));
    let x_str = format!("{:.3}", rule.origin_x);
    if let Some(typed) = text_field(ui, wid("coord-origin-x", ()), r_x, &x_str, mono(11.0), ("X", "m"), t)
        && let Ok(v) = typed.replace(',', "").parse::<f64>()
        && v.is_finite()
    {
        rule.origin_x = v;
    }

    // Northing Y
    let r_y = Rect::from_min_size(pos2(r.left() + half_w + GAP_X, r.top()), vec2(half_w, r.height()));
    let y_str = format!("{:.3}", rule.origin_y);
    if let Some(typed) = text_field(ui, wid("coord-origin-y", ()), r_y, &y_str, mono(11.0), ("Y", "m"), t)
        && let Ok(v) = typed.replace(',', "").parse::<f64>()
        && v.is_finite()
    {
        rule.origin_y = v;
    }
}
