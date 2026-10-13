//! Scanned Underlay 2-Point and 4-Point Alignment Card (Trowel, Dialog 2, J008).
//!
//! Provides 2-point similarity (rotation and scale) and 4-point affine (distortion correction)
//! ground control point (GCP) registration for excavation drawing underlays, computing
//! residual errors and RMSE paper tolerances (<= 2.0 mm).

use serde::{Deserialize, Serialize};
use serde_json::json;
use vectorcraft_doc::DocCoord;
use vectorcraft_doc::coord::format_tm_coord;
use vectorcraft_engine::cmd::underlay::{GcpPoint, find_underlay_image, solve_affine, solve_similarity};
use vectorcraft_geom::Point;

use super::card::{self, button, primary};
use super::*;

const ROW_H: f32 = 24.0;
const SEG_H: f32 = 24.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct GcpRow {
    pub scan_x: f64,
    pub scan_y: f64,
    pub tm_x: f64,
    pub tm_y: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct UnderlayRule {
    pub is_affine: bool,
    pub gcps: Vec<GcpRow>,
}

impl Default for UnderlayRule {
    fn default() -> Self {
        // Blueprint Dialog 2 default GCP coordinates
        Self {
            is_affine: true,
            gcps: vec![
                GcpRow { scan_x: 420.0, scan_y: 380.0, tm_x: 147_962.418, tm_y: 98_131.207 },
                GcpRow { scan_x: 2840.0, scan_y: 392.0, tm_x: 148_021.774, tm_y: 98_128.560 },
                GcpRow { scan_x: 2855.0, scan_y: 2680.0, tm_x: 148_026.093, tm_y: 98_181.945 },
                GcpRow { scan_x: 415.0, scan_y: 2672.0, tm_x: 147_958.650, tm_y: 98_186.312 },
            ],
        }
    }
}

fn rule_id() -> egui::Id {
    wid("underlay-rule", ())
}

/// Start the card over: opened again, it shows the document's current values.
pub(super) fn forget(ctx: &egui::Context) {
    ctx.data_mut(|d| d.remove::<UnderlayRule>(rule_id()));
}

/// Show the Underlay Alignment card at `place`. Returns whether it stays open.
pub(super) fn show(app: &mut VectorcraftApp, ctx: &egui::Context, place: card::Place) -> bool {
    let mut rule = ctx.data(|d| d.get_temp::<UnderlayRule>(rule_id())).unwrap_or_default();

    let title = tw!("Underlay Alignment");
    let sub = if rule.is_affine { tw!("4-Point Affine (Distortion Correction)") } else { tw!("2-Point Similarity (Rotation · Scale)") };

    let (mut open, asks) = card::show(ctx, wid("underlay-card", ()), place, (title, sub), |ui, t| body(app, ui, &mut rule, t));

    for ask in asks {
        match ask {
            Ask::Apply => {
                let gcp_count = if rule.is_affine { 4 } else { 2 };
                let gcps_json: Vec<_> = rule
                    .gcps
                    .iter()
                    .take(gcp_count)
                    .map(|g| {
                        json!({
                            "scan": [g.scan_x, g.scan_y],
                            "target_tm": [g.tm_x, g.tm_y],
                        })
                    })
                    .collect();

                let params = json!({
                    "method": if rule.is_affine { "affine" } else { "twoPoint" },
                    "gcps": gcps_json,
                });

                match app.run("underlay.align", params) {
                    Ok(v) => {
                        let rmse = v["rmse_mm"].as_f64().unwrap_or(0.0);
                        let pass = v["pass"].as_bool().unwrap_or(false);
                        let status_msg = if pass {
                            crate::i18n::fmt(tw!("Underlay aligned: RMSE {rmse:.2} mm (within tolerance)"), &[("rmse", &format!("{rmse:.2}"))])
                        } else {
                            crate::i18n::fmt(tw!("Underlay aligned: RMSE {rmse:.2} mm (exceeds tolerance)"), &[("rmse", &format!("{rmse:.2}"))])
                        };
                        app.status(status_msg);
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

fn body(app: &VectorcraftApp, ui: &mut Ui, rule: &mut UnderlayRule, t: &Tokens) -> Vec<Ask> {
    let mut asks = vec![];

    // 1. Scanned Underlay Image Information
    underlay_info(app, ui, t);
    ui.add_space(6.0);

    // 2. Registration Method Segment (2-Point vs 4-Point Affine)
    method_choice(ui, rule, t);
    ui.add_space(6.0);

    // 3. Three-line GCP Table (t3)
    let solved = gcp_table(app, ui, rule, t);
    ui.add_space(6.0);

    // 4. Comprehensive Error Analysis (RMSE)
    error_summary(ui, solved.as_ref(), t);
    ui.add_space(8.0);

    // 5. Actions (Apply Alignment, Cancel)
    let b_w = card::button_w(ui, tw!("Apply Alignment"));
    let c_w = card::button_w(ui, tw!("Cancel"));
    let places = card::flow(ui, 28.0, &[b_w, c_w]);
    if let Some(r) = places.first()
        && primary(ui, wid("underlay-apply", ()), *r, tw!("Apply Alignment"), true, t).clicked()
    {
        asks.push(Ask::Apply);
    }
    if let Some(r) = places.get(1)
        && button(ui, wid("underlay-close", ()), *r, tw!("Cancel"), true, t).clicked()
    {
        asks.push(Ask::Close);
    }

    asks
}

/// 1. Displays filename and resolution of active underlay scan.
fn underlay_info(app: &VectorcraftApp, ui: &mut Ui, t: &Tokens) {
    let r = card::row(ui, tw!("Scanned Underlay"), ROW_H, t);
    field_box(ui, r, t.input, t.input_border);

    let info_text = if let Some(st) = app.session.active()
        && let Some((_id, im, name)) = find_underlay_image(&st.doc, None)
    {
        format!("{name} ({} × {} px)", im.width, im.height)
    } else {
        "광령리1_현장실측방안_0928.jpg (300 DPI)".to_string()
    };

    text(ui, pos2(r.left() + 8.0, r.center().y), Align2::LEFT_CENTER, &info_text, font(11.0), t.text);
}

/// 2. Registration Method Selection Segment.
fn method_choice(ui: &mut Ui, rule: &mut UnderlayRule, t: &Tokens) {
    let r = card::row(ui, tw!("Method"), SEG_H, t);
    let half_w = (r.width() - 4.0) / 2.0;

    let r_2pt = Rect::from_min_size(r.min, vec2(half_w, SEG_H));
    let r_affine = Rect::from_min_size(pos2(r.min.x + half_w + 4.0, r.min.y), vec2(half_w, SEG_H));

    if seg_btn(ui, wid("underlay-2pt", ()), r_2pt, tw!("2-Point Similarity"), !rule.is_affine, t).clicked() {
        rule.is_affine = false;
    }
    if seg_btn(ui, wid("underlay-affine", ()), r_affine, tw!("4-Point Affine"), rule.is_affine, t).clicked() {
        rule.is_affine = true;
    }
}

fn seg_btn(ui: &mut Ui, id: egui::Id, r: Rect, label: &str, on: bool, t: &Tokens) -> egui::Response {
    let resp = ui.interact(r, id, Sense::click());
    let fill = if on {
        t.action
    } else if resp.hovered() {
        t.hover
    } else {
        t.input
    };
    let border = if on { t.action } else { t.input_border };
    field_box(ui, r, fill, border);
    let color = if on { t.on_action } else { t.text };
    let font_weight = if on { bold(11.0) } else { font(11.0) };
    text(ui, r.center(), Align2::CENTER_CENTER, label, font_weight, color);
    resp
}

/// Solved results for the current card UI.
struct SolvedGcp {
    residuals_mm: Vec<f64>,
    rmse_mm: f64,
    pass: bool,
}

/// 3. Three-line Table (t3) for GCPs.
fn gcp_table(app: &VectorcraftApp, ui: &mut Ui, rule: &UnderlayRule, t: &Tokens) -> Option<SolvedGcp> {
    let gcp_count = if rule.is_affine { 4 } else { 2 };
    let coord = app.session.active().and_then(|st| st.doc.coord.clone()).unwrap_or_else(|| DocCoord::new(20.0, 5186, 148_000.0, 98_150.0));

    // Solve for current parameters
    let mut gcp_pts = Vec::with_capacity(gcp_count);
    for g in rule.gcps.iter().take(gcp_count) {
        let (tx, ty) = coord.tm_to_pt(g.tm_x, g.tm_y);
        gcp_pts.push(GcpPoint { scan: Point::new(g.scan_x, g.scan_y), target: Point::new(tx, ty), target_tm: Some(Point::new(g.tm_x, g.tm_y)) });
    }

    let solved = if rule.is_affine { solve_affine(&gcp_pts).ok() } else { solve_similarity(&gcp_pts).ok() }.map(|res| SolvedGcp {
        residuals_mm: res.residuals_mm,
        rmse_mm: res.rmse_mm,
        pass: res.pass,
    });

    let total_h = 20.0 + (gcp_count as f32) * 20.0 + 4.0;
    let (table_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), total_h), Sense::hover());

    // Three-line table: top border
    ui.painter().hline(table_rect.x_range(), table_rect.top(), Stroke::new(1.0, t.border));

    // Column widths: Total available ~ 276
    let col_w = [36.0, 72.0, 114.0, 54.0];
    let col_x = [
        table_rect.left(),
        table_rect.left() + col_w[0],
        table_rect.left() + col_w[0] + col_w[1],
        table_rect.left() + col_w[0] + col_w[1] + col_w[2],
    ];

    // Table Header
    let head_y = table_rect.top() + 10.0;
    text(ui, pos2(col_x[0], head_y), Align2::LEFT_CENTER, tw!("GCP"), bold(10.0), t.text_dim);
    text(ui, pos2(col_x[1], head_y), Align2::LEFT_CENTER, tw!("Scan (px)"), bold(10.0), t.text_dim);
    text(ui, pos2(col_x[2], head_y), Align2::LEFT_CENTER, tw!("Target TM (m)"), bold(10.0), t.text_dim);
    text(ui, pos2(table_rect.right() - 2.0, head_y), Align2::RIGHT_CENTER, tw!("Residual"), bold(10.0), t.text_dim);

    // Three-line table: mid separator
    ui.painter().hline(table_rect.x_range(), table_rect.top() + 19.0, Stroke::new(0.5, t.border));

    // Table Rows
    for i in 0..gcp_count {
        if let Some(g) = rule.gcps.get(i) {
            let row_y = table_rect.top() + 20.0 + (i as f32) * 20.0 + 10.0;
            let gcp_label = format!("GCP {}", i + 1);
            let scan_label = format!("{:.0}, {:.0}", g.scan_x, g.scan_y);
            let tm_label = format!("{}, {}", format_tm_coord(g.tm_x), format_tm_coord(g.tm_y));

            text(ui, pos2(col_x[0], row_y), Align2::LEFT_CENTER, &gcp_label, font(10.5), t.text);
            text(ui, pos2(col_x[1], row_y), Align2::LEFT_CENTER, &scan_label, font(10.0), t.text_dim);
            text(ui, pos2(col_x[2], row_y), Align2::LEFT_CENTER, &tm_label, font(9.5), t.text);

            let res_text = if let Some(s) = &solved
                && let Some(r) = s.residuals_mm.get(i)
            {
                format!("{r:.2} mm")
            } else {
                "-".to_string()
            };
            text(ui, pos2(table_rect.right() - 2.0, row_y), Align2::RIGHT_CENTER, &res_text, bold(10.0), t.text);
        }
    }

    // Three-line table: bottom border
    ui.painter().hline(table_rect.x_range(), table_rect.bottom(), Stroke::new(1.0, t.border));

    solved
}

/// 4. Error Summary (RMSE and compliance note).
fn error_summary(ui: &mut Ui, solved: Option<&SolvedGcp>, t: &Tokens) {
    let r = card::row(ui, tw!("Error Analysis"), ROW_H, t);
    field_box(ui, r, t.panel_darker, t.input_border);

    let (summary_text, color) = if let Some(s) = solved {
        if s.pass {
            (format!("RMSE: {:.2} mm (허용치 2.0 mm 이내 만족)", s.rmse_mm), t.axis_y)
        } else {
            (format!("RMSE: {:.2} mm (허용치 2.0 mm 초과 경고)", s.rmse_mm), t.clay)
        }
    } else {
        ("오차 계산 불가 (특이점)".to_string(), t.clay)
    };

    text(ui, pos2(r.left() + 6.0, r.center().y), Align2::LEFT_CENTER, &summary_text, bold(11.0), color);
}
