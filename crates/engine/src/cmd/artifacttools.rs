//! Commands for archaeological artifact drafting (J011, design 2.1 §2 & §3):
//! 1:1 scale drafting, centerline (℄), symmetric left-exterior / right-section drafting,
//! 3-point rim/base arc diameter fitting, wall cut cross-section, and 4 core dimension lines.

use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::{LegendSample, LineType};
use vectorcraft_doc::{Appearance, CharStyle, Dash, Document, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::{PathData, Point, Rect, SubPath, shapes};

use super::stratatools::get_or_create_section_layer;
use super::*;

/// Points per centimeter at 1:1 scale (1 inch = 72 pt = 2.54 cm => 1 cm = 72 / 2.54 pt ≈ 28.3464567 pt).
pub const POINTS_PER_CM: f64 = 72.0 / 2.54;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "artifact.setCenterline",
            "Set Artifact Centerline",
            [],
            None,
            "{x: number, y1?: number, y2?: number} add 0.6 pt centerline (1-dot chain dash: 14 3 2 3) on '중심선' layer topped with ℄ symbol (Trowel, J011) → {id, x, top_y, bot_y}",
            has_doc,
            set_centerline
        ),
        cmd!(
            "artifact.fitRimArc",
            "Fit Rim or Base Arc",
            [],
            None,
            "{points: [[x, y], [x, y], [x, y]], kind?: 'rim' | 'base'} fit 3-point circle arc to calculate rim/base diameter and add guide arc (Trowel, J011) → {id, diameter_cm, diameter_pt, radius_pt, center_x, center_y, label}",
            has_doc,
            fit_rim_arc
        ),
        cmd!(
            "artifact.mirrorToExterior",
            "Mirror Section to Exterior Outline",
            [],
            None,
            "{points: [[x, y], ...], centerline_x: number} mirror right section profile across centerline to form left exterior outline (0.5 pt) (Trowel, J011) → {id, points}",
            has_doc,
            mirror_to_exterior
        ),
        cmd!(
            "artifact.addWallCut",
            "Add Vessel Wall Cut Cross-Section",
            [],
            None,
            "{points: [[x, y], ...]} add closed vessel wall cut cross-section with 0.8 pt outline and soft clay fill (Trowel, J011) → {id, points}",
            has_doc,
            add_wall_cut
        ),
        cmd!(
            "artifact.generateDimensionLines",
            "Generate Vessel Dimension Lines",
            [],
            None,
            "{centerline_x: number, rim_y: number, base_y: number, rim_radius: number, base_radius: number, max_body_y?: number, max_body_radius?: number} generate rim, base, max body, and height dimension lines with 45° ticks (Trowel, J011) → {id, rim_cm, base_cm, height_cm}",
            has_doc,
            generate_dimension_lines
        ),
        cmd!(
            "artifact.setProps",
            "Set Artifact Properties",
            [],
            None,
            "{kind?: string, period?: string, rim_cm?: number, base_cm?: number, height_cm?: number} set artifact metadata attributes on document (Trowel, J011) → {kind, period, rim_cm, base_cm, height_cm}",
            has_doc,
            set_artifact_props
        ),
    ]
}

/// Adds a 0.6 pt centerline (1-dot chain dash: 14 3 2 3) on `"중심선"` layer, topped with the `℄` symbol.
fn set_centerline(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "artifact.setCenterline";

    let x = f64_req(p, "x", C)?;
    let y1 = p.get("y1").and_then(Value::as_f64).unwrap_or(80.0);
    let y2 = p.get("y2").and_then(Value::as_f64).unwrap_or(500.0);

    if !x.is_finite() || !y1.is_finite() || !y2.is_finite() || (y2 - y1).abs() < 10.0 {
        return Err(bad(C, "invalid vertical span for centerline"));
    }

    let (top_y, bot_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };

    let group_id = s.edit("Set Artifact Centerline", |d, _| {
        let layer_id = get_or_create_section_layer(d, "중심선", Some(0.6), LineType::DashDot, LegendSample::Line, Some("centerline"))?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some("중심선 (Centerline ℄)".into());

        let mut attrs = group.attrs.take().unwrap_or_default();
        attrs.data.push(("artifact:centerline_x".into(), x.to_string()));
        group.attrs = Some(attrs);

        d.insert(Some(layer_id), usize::MAX, group)?;

        // Centerline geometry: 0.6 pt stroke, dashdot (14 3 2 3)
        let line_pd = shapes::line(Point::new(x, top_y), Point::new(x, bot_y));
        let mut app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.6);
        if let Some(st) = app.stroke_mut() {
            st.dash = Some(Dash { pattern: vec![14.0, 3.0, 2.0, 3.0], offset: 0.0, align_corners: true });
        }
        let lid = d.alloc_id();
        let line_node = Node::path(lid, line_pd, app);
        d.insert(Some(gid), usize::MAX, line_node)?;

        // Centerline symbol `℄` at top
        let text_style = CharStyle { size: 12.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let text_pos = Point::new(x - 5.0, top_y - 8.0);
        let text_obj = TextObject::point(text_pos, "℄", text_style);
        let tid = d.alloc_id();
        let text_node = Node::new(tid, NodeKind::Text(Box::new(text_obj)));
        d.insert(Some(gid), usize::MAX, text_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "top_y": top_y,
        "bot_y": bot_y,
    }))
}

/// Calculates the circumcircle through 3 points to determine rim or base diameter.
/// Returns diameter in cm and pt, and adds guide arc and dimension text.
fn fit_rim_arc(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "artifact.fitRimArc";

    let arr = p.get("points").or_else(|| p.get("anchors")).and_then(Value::as_array).ok_or_else(|| bad(C, "points array is required"))?;
    if arr.len() < 3 {
        return Err(bad(C, "exactly 3 points are required to fit a rim arc"));
    }

    let mut pts = Vec::with_capacity(3);
    for item in arr.iter().take(3) {
        let (x, y) = if let Some(xy) = item.as_array() {
            if xy.len() < 2 {
                return Err(bad(C, "invalid coordinates"));
            }
            (xy.first().and_then(Value::as_f64).unwrap_or(0.0), xy.get(1).and_then(Value::as_f64).unwrap_or(0.0))
        } else if let Some(obj) = item.as_object() {
            (obj.get("x").and_then(Value::as_f64).unwrap_or(0.0), obj.get("y").and_then(Value::as_f64).unwrap_or(0.0))
        } else {
            return Err(bad(C, "invalid point format"));
        };
        if !x.is_finite() || !y.is_finite() {
            return Err(bad(C, "coordinates must be finite"));
        }
        pts.push(Point::new(x, y));
    }

    let (p1, p2, p3) = (pts[0], pts[1], pts[2]);
    let d = 2.0 * (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y));
    if d.abs() < 1e-7 {
        return Err(bad(C, "points are collinear and cannot form a circle arc"));
    }

    let p1_sq = p1.x * p1.x + p1.y * p1.y;
    let p2_sq = p2.x * p2.x + p2.y * p2.y;
    let p3_sq = p3.x * p3.x + p3.y * p3.y;

    let center_x = (p1_sq * (p2.y - p3.y) + p2_sq * (p3.y - p1.y) + p3_sq * (p1.y - p2.y)) / d;
    let center_y = (p1_sq * (p3.x - p2.x) + p2_sq * (p1.x - p3.x) + p3_sq * (p2.x - p1.x)) / d;

    let radius_pt = ((p1.x - center_x).powi(2) + (p1.y - center_y).powi(2)).sqrt();
    let diameter_pt = radius_pt * 2.0;
    let diameter_cm = diameter_pt / POINTS_PER_CM;

    let kind = p.get("kind").and_then(Value::as_str).unwrap_or("rim");
    let kind_korean = match kind {
        "base" => "저경(底徑)",
        _ => "구경(口徑)",
    };

    let group_id = s.edit("Fit Rim Arc", |d, _| {
        let layer_id = get_or_create_section_layer(d, "실측 보조선", Some(0.3), LineType::Dotted, LegendSample::Line, Some("drafting_guide"))?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some(format!("{kind_korean}: {diameter_cm:.1} cm"));

        let mut attrs = group.attrs.take().unwrap_or_default();
        attrs.data.push(("artifact:fit_kind".into(), kind.into()));
        attrs.data.push(("artifact:diameter_cm".into(), format!("{diameter_cm:.2}")));
        attrs.data.push(("artifact:radius_pt".into(), format!("{radius_pt:.2}")));
        attrs.data.push(("artifact:center_x".into(), format!("{center_x:.2}")));
        attrs.data.push(("artifact:center_y".into(), format!("{center_y:.2}")));
        group.attrs = Some(attrs);

        d.insert(Some(layer_id), usize::MAX, group)?;

        // Guide Arc: 0.3 pt dotted guide ellipse/arc
        let arc_pd = shapes::ellipse(Rect::new(center_x - radius_pt, center_y - radius_pt, center_x + radius_pt, center_y + radius_pt));
        let mut arc_app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0x3B, 0x82, 0xF6)), 0.3);
        if let Some(st) = arc_app.stroke_mut() {
            st.dash = Some(Dash { pattern: vec![3.0, 3.0], offset: 0.0, align_corners: true });
        }
        let aid = d.alloc_id();
        let arc_node = Node::path(aid, arc_pd, arc_app);
        d.insert(Some(gid), usize::MAX, arc_node)?;

        // Measurement label at apex or p2
        let text_style =
            CharStyle { size: 9.0, fill: Paint::solid(Color::rgb8(0x1E, 0x3A, 0x8A)), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let label_text = format!("{kind_korean}: {diameter_cm:.1} cm");
        let text_pos = Point::new(p2.x - 20.0, p2.y - 6.0);
        let text_obj = TextObject::point(text_pos, &label_text, text_style);
        let tid = d.alloc_id();
        let text_node = Node::new(tid, NodeKind::Text(Box::new(text_obj)));
        d.insert(Some(gid), usize::MAX, text_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "diameter_cm": diameter_cm,
        "diameter_pt": diameter_pt,
        "radius_pt": radius_pt,
        "center_x": center_x,
        "center_y": center_y,
        "label": format!("{kind_korean}: {diameter_cm:.1} cm"),
    }))
}

/// Mirrors section points on the right of the centerline to produce the left exterior outline (0.5 pt solid line).
fn mirror_to_exterior(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "artifact.mirrorToExterior";

    let arr = p.get("points").or_else(|| p.get("anchors")).and_then(Value::as_array).ok_or_else(|| bad(C, "points array is required"))?;
    if arr.len() < 2 {
        return Err(bad(C, "at least 2 points are required"));
    }

    let centerline_x = f64_req(p, "centerline_x", C)?;
    if !centerline_x.is_finite() {
        return Err(bad(C, "centerline_x must be finite"));
    }

    let mut mirrored_pts = Vec::with_capacity(arr.len());
    for item in arr {
        let (x, y) = if let Some(xy) = item.as_array() {
            if xy.len() < 2 {
                return Err(bad(C, "invalid coordinates"));
            }
            (xy.first().and_then(Value::as_f64).unwrap_or(0.0), xy.get(1).and_then(Value::as_f64).unwrap_or(0.0))
        } else if let Some(obj) = item.as_object() {
            (obj.get("x").and_then(Value::as_f64).unwrap_or(0.0), obj.get("y").and_then(Value::as_f64).unwrap_or(0.0))
        } else {
            return Err(bad(C, "invalid point format"));
        };
        if !x.is_finite() || !y.is_finite() {
            return Err(bad(C, "coordinates must be finite"));
        }
        // Reflection across vertical line x = centerline_x:
        let mirrored_x = 2.0 * centerline_x - x;
        mirrored_pts.push(Point::new(mirrored_x, y));
    }

    let path_id = s.edit("Mirror to Exterior", |d, _| {
        let layer_id = get_or_create_section_layer(d, "외형선", Some(0.5), LineType::Solid, LegendSample::Line, Some("exterior_outline"))?;

        let pd = PathData::new(vec![SubPath::polyline(&mirrored_pts, false)]);
        let app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.5);
        let pid = d.alloc_id();
        let mut node = Node::path(pid, pd, app);
        node.name = Some("외형 윤곽선 (0.5 pt)".into());
        d.insert(Some(layer_id), usize::MAX, node)?;
        Ok(pid)
    })?;

    Ok(json!({
        "id": path_id.0,
        "points": mirrored_pts.len(),
    }))
}

/// Forms a closed polygon for the cut profile of the vessel wall on the right half.
/// Outline: 0.8 pt, fill: solid soft clay (#F3DEDA) or 45° 0.1 pt hatch.
fn add_wall_cut(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "artifact.addWallCut";

    let arr = p.get("points").or_else(|| p.get("anchors")).and_then(Value::as_array).ok_or_else(|| bad(C, "points array is required"))?;
    if arr.len() < 3 {
        return Err(bad(C, "at least 3 points are required for wall cut cross-section"));
    }

    let mut points = Vec::with_capacity(arr.len());
    for item in arr {
        let (x, y) = if let Some(xy) = item.as_array() {
            if xy.len() < 2 {
                return Err(bad(C, "invalid coordinates"));
            }
            (xy.first().and_then(Value::as_f64).unwrap_or(0.0), xy.get(1).and_then(Value::as_f64).unwrap_or(0.0))
        } else if let Some(obj) = item.as_object() {
            (obj.get("x").and_then(Value::as_f64).unwrap_or(0.0), obj.get("y").and_then(Value::as_f64).unwrap_or(0.0))
        } else {
            return Err(bad(C, "invalid point format"));
        };
        if !x.is_finite() || !y.is_finite() {
            return Err(bad(C, "coordinates must be finite"));
        }
        points.push(Point::new(x, y));
    }

    let path_id = s.edit("Add Wall Cut Cross-Section", |d, _| {
        let layer_id = get_or_create_section_layer(d, "단면 기벽", Some(0.8), LineType::Solid, LegendSample::Line, Some("vessel_wall_cut"))?;

        let pd = PathData::new(vec![SubPath::polyline(&points, true)]);
        // Standard academic wall cut: 0.8 pt outer border + subtle fill (#F3DEDA)
        let fill = Paint::solid(Color::rgb8(0xF3, 0xDE, 0xDA));
        let stroke = Paint::solid(Color::BLACK);
        let app = Appearance::basic(fill, stroke, 0.8);

        let pid = d.alloc_id();
        let mut node = Node::path(pid, pd, app);
        node.name = Some("기벽 단면 (0.8 pt)".into());
        d.insert(Some(layer_id), usize::MAX, node)?;
        Ok(pid)
    })?;

    Ok(json!({
        "id": path_id.0,
        "points": points.len(),
    }))
}

/// Creates the 4 standard archaeological vessel dimension lines:
/// Rim Diameter (구경), Max Body Diameter (최대동경), Base Diameter (저경), Vessel Height (기고).
fn generate_dimension_lines(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "artifact.generateDimensionLines";

    let cx = f64_req(p, "centerline_x", C)?;
    let rim_y = f64_req(p, "rim_y", C)?;
    let base_y = f64_req(p, "base_y", C)?;
    let rim_radius = f64_req(p, "rim_radius", C)?;
    let base_radius = f64_req(p, "base_radius", C)?;

    let max_body_y = p.get("max_body_y").and_then(Value::as_f64);
    let max_body_radius = p.get("max_body_radius").and_then(Value::as_f64);

    let height_pt = (base_y - rim_y).abs();
    let height_cm = height_pt / POINTS_PER_CM;
    let rim_cm = (rim_radius * 2.0) / POINTS_PER_CM;
    let base_cm = (base_radius * 2.0) / POINTS_PER_CM;

    let group_id = s.edit("Generate Dimension Lines", |d, _| {
        let layer_id = get_or_create_section_layer(d, "치수선", Some(0.3), LineType::Solid, LegendSample::Line, Some("dimension_lines"))?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some(format!("유물 치수선 (구경 {rim_cm:.1}cm · 기고 {height_cm:.1}cm)"));

        let mut attrs = group.attrs.take().unwrap_or_default();
        attrs.data.push(("artifact:rim_cm".into(), format!("{rim_cm:.1}")));
        attrs.data.push(("artifact:base_cm".into(), format!("{base_cm:.1}")));
        attrs.data.push(("artifact:height_cm".into(), format!("{height_cm:.1}")));
        group.attrs = Some(attrs);

        d.insert(Some(layer_id), usize::MAX, group)?;

        let line_app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0x64, 0x74, 0x8B)), 0.3);
        let text_style =
            CharStyle { size: 8.5, fill: Paint::solid(Color::rgb8(0x33, 0x41, 0x55)), font_family: "Noto Sans KR".into(), ..CharStyle::default() };

        // Helper to add a horizontal dimension line with ticks and text:
        let add_h_dim = |d: &mut Document, y: f64, radius: f64, label: &str| -> Result<()> {
            let left_x = cx - radius;
            let right_x = cx + radius;

            // Main horizontal line
            let pd = shapes::line(Point::new(left_x, y), Point::new(right_x, y));
            let lid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::path(lid, pd, line_app.clone()))?;

            // 45° Tick marks at ends
            let tick_len = 4.0;
            let left_tick = shapes::line(Point::new(left_x - tick_len, y + tick_len), Point::new(left_x + tick_len, y - tick_len));
            let ltid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::path(ltid, left_tick, line_app.clone()))?;

            let right_tick = shapes::line(Point::new(right_x - tick_len, y + tick_len), Point::new(right_x + tick_len, y - tick_len));
            let rtid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::path(rtid, right_tick, line_app.clone()))?;

            // Centered dimension label
            let text_pos = Point::new(cx - 18.0, y - 4.0);
            let text_obj = TextObject::point(text_pos, label, text_style.clone());
            let tid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(tid, NodeKind::Text(Box::new(text_obj))))?;

            Ok(())
        };

        // 1. Rim Dimension (placed 16 pt above rim)
        add_h_dim(d, rim_y - 16.0, rim_radius, &format!("구경: {rim_cm:.1} cm"))?;

        // 2. Base Dimension (placed 16 pt below base)
        add_h_dim(d, base_y + 16.0, base_radius, &format!("저경: {base_cm:.1} cm"))?;

        // 3. Max Body Dimension (if specified)
        if let (Some(mby), Some(mbr)) = (max_body_y, max_body_radius) {
            let mb_cm = (mbr * 2.0) / POINTS_PER_CM;
            add_h_dim(d, mby, mbr, &format!("최대동경: {mb_cm:.1} cm"))?;
        }

        // 4. Height Dimension (placed 24 pt to the right of max width)
        let max_r = rim_radius.max(base_radius).max(max_body_radius.unwrap_or(0.0));
        let h_dim_x = cx + max_r + 24.0;

        let vert_line = shapes::line(Point::new(h_dim_x, rim_y), Point::new(h_dim_x, base_y));
        let vlid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(vlid, vert_line, line_app.clone()))?;

        // Top/bottom ticks for vertical line
        let top_tick = shapes::line(Point::new(h_dim_x - 4.0, rim_y + 4.0), Point::new(h_dim_x + 4.0, rim_y - 4.0));
        let ttid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(ttid, top_tick, line_app.clone()))?;

        let bot_tick = shapes::line(Point::new(h_dim_x - 4.0, base_y + 4.0), Point::new(h_dim_x + 4.0, base_y - 4.0));
        let btid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(btid, bot_tick, line_app))?;

        // Height label
        let h_text_pos = Point::new(h_dim_x + 6.0, (rim_y + base_y) / 2.0 + 3.0);
        let h_text_obj = TextObject::point(h_text_pos, &format!("기고: {height_cm:.1} cm"), text_style);
        let htid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(htid, NodeKind::Text(Box::new(h_text_obj))))?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "rim_cm": rim_cm,
        "base_cm": base_cm,
        "height_cm": height_cm,
    }))
}

/// 6. Set Artifact Properties:
fn set_artifact_props(s: &mut Session, p: &Value) -> Result<Value> {
    let kind = p.get("kind").and_then(Value::as_str);
    let period = p.get("period").and_then(Value::as_str);
    let rim_cm = p.get("rim_cm").and_then(Value::as_f64);
    let base_cm = p.get("base_cm").and_then(Value::as_f64);
    let height_cm = p.get("height_cm").and_then(Value::as_f64);

    s.edit("Set Artifact Properties", |d, _| {
        if let Some(first_layer) = d.layers.first() {
            let layer_id = first_layer.id;
            if let Some(node) = d.node_mut(layer_id) {
                let mut attrs = node.attrs.take().unwrap_or_default();
                if let Some(k) = kind {
                    attrs.data.retain(|(key, _)| key != "artifact:kind");
                    attrs.data.push(("artifact:kind".into(), k.into()));
                }
                if let Some(pr) = period {
                    attrs.data.retain(|(key, _)| key != "artifact:period");
                    attrs.data.push(("artifact:period".into(), pr.into()));
                }
                if let Some(r) = rim_cm {
                    attrs.data.retain(|(key, _)| key != "artifact:rim_cm");
                    attrs.data.push(("artifact:rim_cm".into(), format!("{r:.1}")));
                }
                if let Some(b) = base_cm {
                    attrs.data.retain(|(key, _)| key != "artifact:base_cm");
                    attrs.data.push(("artifact:base_cm".into(), format!("{b:.1}")));
                }
                if let Some(h) = height_cm {
                    attrs.data.retain(|(key, _)| key != "artifact:height_cm");
                    attrs.data.push(("artifact:height_cm".into(), format!("{h:.1}")));
                }
                node.attrs = Some(attrs);
            }
        }
        Ok(NodeId(0))
    })?;

    Ok(json!({
        "kind": kind,
        "period": period,
        "rim_cm": rim_cm,
        "base_cm": base_cm,
        "height_cm": height_cm,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_with_doc() -> Session {
        let mut s = Session::new();
        s.execute(
            "file.new",
            &json!({
                "width": 800,
                "height": 600
            }),
        )
        .unwrap();
        s
    }

    #[test]
    fn test_set_centerline() {
        let mut s = session_with_doc();
        let res = s.execute("artifact.setCenterline", &json!({ "x": 300.0, "y1": 50.0, "y2": 450.0 })).unwrap();
        assert_eq!(res["x"], 300.0);
        assert_eq!(res["top_y"], 50.0);
        assert_eq!(res["bot_y"], 450.0);
    }

    #[test]
    fn test_fit_rim_arc_3_points() {
        let mut s = session_with_doc();
        // A circle with center (200, 200) and radius 100 pt:
        // p1 = (100, 200), p2 = (200, 100), p3 = (300, 200)
        let res = s
            .execute(
                "artifact.fitRimArc",
                &json!({
                    "points": [[100.0, 200.0], [200.0, 100.0], [300.0, 200.0]],
                    "kind": "rim"
                }),
            )
            .unwrap();

        let diam_cm = res["diameter_cm"].as_f64().unwrap();
        let radius_pt = res["radius_pt"].as_f64().unwrap();
        assert!((radius_pt - 100.0).abs() < 1e-4);
        assert!((diam_cm - (200.0 / POINTS_PER_CM)).abs() < 1e-4);
    }

    #[test]
    fn test_mirror_to_exterior_and_wall_cut() {
        let mut s = session_with_doc();
        let cx = 300.0;
        let pts = [[350.0, 100.0], [380.0, 200.0], [340.0, 300.0]];
        let mirror_res = s.execute("artifact.mirrorToExterior", &json!({ "points": pts, "centerline_x": cx })).unwrap();
        assert_eq!(mirror_res["points"], 3);

        let wall_pts = [[350.0, 100.0], [380.0, 200.0], [370.0, 200.0], [345.0, 100.0]];
        let wall_res = s.execute("artifact.addWallCut", &json!({ "points": wall_pts })).unwrap();
        assert_eq!(wall_res["points"], 4);
    }

    #[test]
    fn test_generate_dimension_lines_and_props() {
        let mut s = session_with_doc();
        let dim_res = s
            .execute(
                "artifact.generateDimensionLines",
                &json!({
                    "centerline_x": 300.0,
                    "rim_y": 100.0,
                    "base_y": 350.0,
                    "rim_radius": 141.73228, // 5 cm radius => 10 cm rim
                    "base_radius": 70.86614,  // 2.5 cm radius => 5 cm base
                    "max_body_y": 200.0,
                    "max_body_radius": 170.0
                }),
            )
            .unwrap();

        assert!((dim_res["rim_cm"].as_f64().unwrap() - 10.0).abs() < 0.1);
        assert!((dim_res["base_cm"].as_f64().unwrap() - 5.0).abs() < 0.1);
        assert!(dim_res["height_cm"].as_f64().unwrap() > 0.0);

        let props_res = s
            .execute(
                "artifact.setProps",
                &json!({
                    "kind": "토기",
                    "period": "신석기",
                    "rim_cm": 10.0,
                    "base_cm": 5.0,
                    "height_cm": 8.8
                }),
            )
            .unwrap();
        assert_eq!(props_res["kind"], "토기");
        assert_eq!(props_res["period"], "신석기");
    }
}
