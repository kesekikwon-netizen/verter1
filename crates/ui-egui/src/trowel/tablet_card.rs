//! Tablet & Pen Sensitivity Settings Card (J016 / J017, blueprint Section 6):
//! Wacom / XP-Pen / Apple Pencil stylus calibration, pressure curve selection
//! (Soft / Normal / Firm), 3 pt tap tolerance, 8 px close radius, Windows Ink gesture
//! suppression, palm rejection, and interactive test drawing canvas.

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::card::{self, GAP_X, button, primary};
use super::*;
use crate::canvas::apply_pressure_curve;

const ROW_H: f32 = 24.0;
const SEG_H: f32 = 24.0;
const TEST_BOX_H: f32 = 100.0;

/// Currently edited settings in the tablet card.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct TabletRule {
    pub curve: String,
    pub tap_tolerance: f64,
    pub close_radius: f64,
    pub palm_rejection: bool,
    pub disable_gestures: bool,
    pub canvas_button_row: bool,
}

impl Default for TabletRule {
    fn default() -> Self {
        Self {
            curve: "normal".to_string(),
            tap_tolerance: 3.0,
            close_radius: 8.0,
            palm_rejection: true,
            disable_gestures: true,
            canvas_button_row: true,
        }
    }
}

/// Scratch points for interactive test drawing in the card.
#[derive(Clone, Debug, Default)]
struct TestStrokes {
    strokes: Vec<Vec<(Pos2, f32)>>,
    current: Vec<(Pos2, f32)>,
    last_pressure: f32,
}

fn rule_id() -> egui::Id {
    wid("tablet-rule", ())
}

fn test_id() -> egui::Id {
    wid("tablet-test-strokes", ())
}

/// Reset temporary card state.
pub(super) fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| {
        d.remove::<TabletRule>(rule_id());
        d.remove::<TestStrokes>(test_id());
    });
}

/// Show the Tablet & Pen Settings card at `place`. Returns whether it stays open.
pub(super) fn show(app: &mut VectorcraftApp, ctx: &egui::Context, place: card::Place) -> bool {
    let mut rule = ctx.data(|d| d.get_temp::<TabletRule>(rule_id())).unwrap_or_else(|| TabletRule {
        curve: app.session.prefs.pen_pressure_curve.clone(),
        tap_tolerance: app.session.prefs.pen_tap_tolerance,
        close_radius: app.session.prefs.pen_close_radius,
        palm_rejection: app.session.prefs.pen_palm_rejection,
        disable_gestures: app.session.prefs.pen_disable_gestures,
        canvas_button_row: app.session.prefs.canvas_button_row,
    });

    let title = tw!("Tablet & Pen Settings");
    let sub = match rule.curve.as_str() {
        "soft" => tw!("Soft Curve · 3 pt Tap · 8 px Close"),
        "firm" => tw!("Firm Curve · 3 pt Tap · 8 px Close"),
        _ => tw!("Normal Curve (Archaeological CAD) · 3 pt / 8 px"),
    };

    let (mut open, asks) = card::show(ctx, wid("tablet-card", ()), place, (title, sub), |ui, t| body(ui, &mut rule, t));

    for ask in asks {
        match ask {
            Ask::Apply => {
                let values = json!({
                    "penPressureCurve": rule.curve,
                    "penTapTolerance": rule.tap_tolerance,
                    "penCloseRadius": rule.close_radius,
                    "penPalmRejection": rule.palm_rejection,
                    "penDisableGestures": rule.disable_gestures,
                    "canvasButtonRow": rule.canvas_button_row,
                });
                if let Err(e) = app.run("prefs.set", json!({ "values": values })) {
                    app.status(e);
                } else {
                    app.status(tw!("Tablet & Pen settings saved"));
                    open = false;
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

fn body(ui: &mut Ui, rule: &mut TabletRule, t: &Tokens) -> Vec<Ask> {
    let mut asks = vec![];

    // 1. Recognized Device Information
    device_info_row(ui, t);
    ui.add_space(6.0);

    // 2. Pressure Sensitivity Curve
    pressure_curve_row(ui, rule, t);
    ui.add_space(6.0);

    // 3. Tolerances (3 pt Tap, 8 px Close)
    tolerances_row(ui, rule, t);
    ui.add_space(6.0);

    // 4. Options (Palm Rejection, Windows Gestures, Canvas Buttons)
    options_rows(ui, rule, t);
    ui.add_space(6.0);

    // 5. Interactive Pen Test Drawing Field
    pen_test_area(ui, &rule.curve, t);
    ui.add_space(8.0);

    // 6. Action Buttons
    let b_w = card::button_w(ui, tw!("Save Settings"));
    let c_w = card::button_w(ui, tw!("Cancel"));
    let places = card::flow(ui, 28.0, &[b_w, c_w]);
    if let Some(r) = places.first()
        && primary(ui, wid("tablet-apply", ()), *r, tw!("Save Settings"), true, t).clicked()
    {
        asks.push(Ask::Apply);
    }
    if let Some(r) = places.get(1)
        && button(ui, wid("tablet-close", ()), *r, tw!("Cancel"), true, t).clicked()
    {
        asks.push(Ask::Close);
    }

    asks
}

/// Device info banner.
fn device_info_row(ui: &mut Ui, t: &Tokens) {
    let r = card::row(ui, tw!("Device"), ROW_H, t);
    field_box(ui, r, t.input, t.input_border);
    let dev_label = if cfg!(target_os = "windows") {
        "Wacom / Windows Ink Digitizer"
    } else if cfg!(target_os = "macos") {
        "Wacom / Apple Pencil (Cocoa)"
    } else {
        "Wacom / XP-Pen Tablet"
    };
    text(ui, pos2(r.left() + 8.0, r.center().y), Align2::LEFT_CENTER, dev_label, mono(11.0), t.text);
}

/// 3-way segmented curve selection: Soft | Normal | Firm.
fn pressure_curve_row(ui: &mut Ui, rule: &mut TabletRule, t: &Tokens) {
    let r = card::row(ui, tw!("Pressure"), SEG_H, t);
    field_box(ui, r, t.input, t.input_border);

    let curves = [("soft", tw!("Soft")), ("normal", tw!("Normal (CAD)")), ("firm", tw!("Firm"))];
    let seg_w = r.width() / curves.len() as f32;

    for (i, (key, label)) in curves.into_iter().enumerate() {
        let seg = Rect::from_min_size(pos2(r.left() + i as f32 * seg_w, r.top()), vec2(seg_w, SEG_H));
        let selected = rule.curve == key;
        let resp = ui.interact(seg, wid("tablet-curve", key), Sense::click());

        if selected {
            ui.painter().rect(seg.shrink(1.0), CornerRadius::same(6), t.row_selected, Stroke::new(1.0, t.text), StrokeKind::Inside);
        } else if resp.hovered() {
            ui.painter().rect_filled(seg.shrink(1.0), CornerRadius::same(6), t.hover);
        }
        if i > 0 {
            ui.painter().vline(seg.left() + 0.5, seg.y_range(), Stroke::new(1.0, t.border));
        }

        let col = if selected || resp.hovered() { t.text } else { t.text_dim };
        text(ui, seg.center(), Align2::CENTER_CENTER, label, font(11.0), col);

        if resp.clicked() {
            rule.curve = key.to_string();
        }
    }
}

/// Tolerances display (3 pt tap, 8 px close).
fn tolerances_row(ui: &mut Ui, rule: &mut TabletRule, t: &Tokens) {
    let r = card::row(ui, tw!("Tolerances"), ROW_H, t);
    let half_w = (r.width() - GAP_X) / 2.0;

    // Tap tolerance
    let r1 = Rect::from_min_size(r.min, vec2(half_w, ROW_H));
    field_box(ui, r1, t.input, t.input_border);
    let tap_str = format!("{:.1} pt (Tap)", rule.tap_tolerance);
    text(ui, r1.center(), Align2::CENTER_CENTER, &tap_str, mono(11.0), t.text);

    // Close radius
    let r2 = Rect::from_min_size(pos2(r.left() + half_w + GAP_X, r.top()), vec2(half_w, ROW_H));
    field_box(ui, r2, t.input, t.input_border);
    let close_str = format!("{:.0} px (Close)", rule.close_radius);
    text(ui, r2.center(), Align2::CENTER_CENTER, &close_str, mono(11.0), t.text);
}

/// Palm rejection, Windows pen gestures, and Canvas button row checkboxes.
fn options_rows(ui: &mut Ui, rule: &mut TabletRule, t: &Tokens) {
    let check_box = |ui: &mut Ui, id_str: &str, val: &mut bool, label: &str| {
        let r = card::row(ui, "", 20.0, t);
        let resp = ui.interact(r, wid("tablet-opt", id_str), Sense::click());
        let box_r = Rect::from_center_size(pos2(r.left() + 8.0, r.center().y), vec2(14.0, 14.0));
        ui.painter().rect(
            box_r,
            CornerRadius::same(3),
            if *val { t.row_selected } else { t.input },
            Stroke::new(1.0, if *val { t.accent } else { t.input_border }),
            StrokeKind::Inside,
        );
        if *val {
            crate::icons::paint(ui, "check", box_r.shrink(2.0), t.accent);
        }
        text(ui, pos2(r.left() + 22.0, r.center().y), Align2::LEFT_CENTER, label, font(11.0), t.text);
        if resp.clicked() {
            *val = !*val;
        }
    };

    check_box(ui, "palm", &mut rule.palm_rejection, tw!("Palm Rejection (Ignore touch while drawing)"));
    check_box(ui, "gestures", &mut rule.disable_gestures, tw!("Disable Windows Pen Gestures / Flicks"));
    check_box(ui, "canvas_bar", &mut rule.canvas_button_row, tw!("Show Canvas Quick Action Buttons"));
}

/// Interactive pen calibration & test drawing canvas area.
fn pen_test_area(ui: &mut Ui, curve: &str, t: &Tokens) {
    let r = card::row(ui, tw!("Pen Test"), TEST_BOX_H, t);
    field_box(ui, r, t.panel_darker, t.input_border);

    let ctx = ui.ctx();
    let mut data = ctx.data(|d| d.get_temp::<TestStrokes>(test_id())).unwrap_or_default();

    // Check interaction inside test box
    let resp = ui.interact(r, wid("pen-test-area", ()), Sense::click_and_drag());
    let pointer = ui.input(|i| i.pointer.clone());

    if resp.hovered() || resp.dragged() {
        if pointer.any_pressed()
            && let Some(p) = pointer.interact_pos()
        {
            if r.contains(p) {
                let raw_force = ui
                    .input(|i| {
                        i.events.iter().rev().find_map(|e| match e {
                            egui::Event::Touch { force: Some(f), .. } if f.is_finite() => Some(f.clamp(0.0, 1.0)),
                            _ => None,
                        })
                    })
                    .unwrap_or(0.7);
                let eff_p = apply_pressure_curve(raw_force, curve);
                data.last_pressure = eff_p;
                data.current = vec![(p, eff_p)];
            }
        } else if resp.dragged()
            && let Some(p) = pointer.interact_pos()
        {
            if r.contains(p) {
                let raw_force = ui
                    .input(|i| {
                        i.events.iter().rev().find_map(|e| match e {
                            egui::Event::Touch { force: Some(f), .. } if f.is_finite() => Some(f.clamp(0.0, 1.0)),
                            _ => None,
                        })
                    })
                    .unwrap_or(data.last_pressure);
                let eff_p = apply_pressure_curve(raw_force, curve);
                data.last_pressure = eff_p;
                data.current.push((p, eff_p));
            }
        } else if pointer.any_released() && !data.current.is_empty() {
            let finished = std::mem::take(&mut data.current);
            data.strokes.push(finished);
        }
    }

    // Paint past test strokes
    let painter = ui.painter().with_clip_rect(r);
    for stroke in &data.strokes {
        for w in stroke.windows(2) {
            let width = (w[1].1 * 4.5).clamp(1.0, 6.0);
            painter.line_segment([w[0].0, w[1].0], Stroke::new(width, t.text));
        }
    }
    // Paint current stroke
    for w in data.current.windows(2) {
        let width = (w[1].1 * 4.5).clamp(1.0, 6.0);
        painter.line_segment([w[0].0, w[1].0], Stroke::new(width, t.clay));
    }

    // Prompt text when empty
    if data.strokes.is_empty() && data.current.is_empty() {
        text(ui, r.center(), Align2::CENTER_CENTER, tw!("Draw with pen to test pressure & curve"), font(11.0), t.text_disabled);
    }

    // Clear test button
    let clear_r = Rect::from_min_size(pos2(r.right() - 48.0, r.top() + 4.0), vec2(44.0, 18.0));
    let clear_resp = ui.interact(clear_r, wid("pen-test-clear", ()), Sense::click());
    if clear_resp.hovered() {
        ui.painter().rect_filled(clear_r, CornerRadius::same(3), t.hover);
    }
    text(ui, clear_r.center(), Align2::CENTER_CENTER, tw!("Clear"), font(10.0), t.text_dim);
    if clear_resp.clicked() {
        data.strokes.clear();
        data.current.clear();
        data.last_pressure = 0.0;
    }

    // Pressure gauge bar at the bottom
    let gauge_r = Rect::from_min_max(pos2(r.left() + 4.0, r.bottom() - 6.0), pos2(r.right() - 4.0, r.bottom() - 2.0));
    ui.painter().rect_filled(gauge_r, CornerRadius::same(2), t.input_border);
    let fill_w = gauge_r.width() * data.last_pressure.clamp(0.0, 1.0);
    let fill_r = Rect::from_min_size(gauge_r.min, vec2(fill_w, gauge_r.height()));
    ui.painter().rect_filled(fill_r, CornerRadius::same(2), t.accent);

    ctx.data_mut(|d| d.insert_temp(test_id(), data));
}
