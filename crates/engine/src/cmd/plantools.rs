//! Archaeological Plan Tools engine commands (Trowel, J009).
//!
//! Provides the five foundational field survey drafting commands:
//! - Section Line (`drawing.addSectionLine`)
//! - Spot Height (`drawing.addSpotHeight`)
//! - Find Number Badge (`drawing.addFindNumber`)
//! - Posthole Depth (`drawing.addPosthole`)
//! - Slope Hachure Ticks (`drawing.addSlopeTicks`)
//! - Polygon Closure Metrics (`drawing.calculateMetrics`)
//! - Feature Properties (`feature.setProps`)

use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::{Legend, LegendGroup, LegendSample, LineType};
use vectorcraft_doc::{Appearance, ArrowAlign, Arrowhead, CharStyle, Dash, Document, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::{PathData, Point, Rect, SubPath, kurbo, shapes};

use super::edit::selected_roots;
use super::layer::make_layer;
use super::*;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "drawing.addSectionLine",
            "Add Section Line",
            [],
            None,
            "{x1: number, y1: number, x2: number, y2: number, label?: string, flip?: bool, tick_length?: number} add archaeological cross-section line with cut ticks and sight arrows (Trowel, J009) → {id, label, x1, y1, x2, y2, flip}",
            has_doc,
            add_section_line
        ),
        cmd!(
            "drawing.addSpotHeight",
            "Add Spot Height",
            [],
            None,
            "{x: number, y: number, elevation?: number, label?: string} add ground survey elevation dot and height text (Trowel, J009) → {id, x, y, elevation, label}",
            has_doc,
            add_spot_height
        ),
        cmd!(
            "drawing.addFindNumber",
            "Add Find Number",
            [],
            None,
            "{x: number, y: number, number?: number} add artifact find badge circle with auto-incrementing number (Trowel, J009) → {id, x, y, number}",
            has_doc,
            add_find_number
        ),
        cmd!(
            "drawing.addPosthole",
            "Add Posthole",
            [],
            None,
            "{x: number, y: number, radius?: number, number?: number, depth_cm?: number} add posthole circle and depth label (Trowel, J009) → {id, x, y, radius, number, depth_cm}",
            has_doc,
            add_posthole
        ),
        cmd!(
            "drawing.addSlopeTicks",
            "Add Slope Hachure Ticks",
            [],
            None,
            "{points: [[x, y], ...], tick_length?: number, spacing?: number, flip?: bool} generate alternating long and short slope hachure ticks along ridge (Trowel, J009) → {id, ticks, length, flip}",
            has_doc,
            add_slope_ticks
        ),
        cmd!(
            "drawing.calculateMetrics",
            "Calculate Polygon Metrics",
            [],
            None,
            "{ids?: [id...]} calculate real-world area (m²), perimeter (m), and major/minor axes for closed polygon features (Trowel, J009) → {ids, area_m2, perimeter_m, major_axis_m, minor_axis_m, area_pt, perimeter_pt}",
            has_doc,
            calculate_metrics
        ),
        cmd!(
            "feature.setProps",
            "Set Feature Properties",
            [],
            None,
            "{ids?: [id...], kind?: string, number?: string|number, period?: string} set archaeological feature metadata attributes and name (Trowel, J009) → {ids, kind, number, period}",
            has_doc,
            set_feature_props
        ),
    ]
}

fn get_or_create_plan_layer(
    d: &mut Document,
    name: &str,
    weight: Option<f64>,
    line: LineType,
    sample: LegendSample,
    strata: Option<&str>,
) -> Result<NodeId> {
    if let Some(l) = d.layers.iter().find(|l| l.display_name().trim() == name) {
        return Ok(l.id);
    }
    let mut n = make_layer(d, Some(name));
    if let NodeKind::Layer { ref mut legend, .. } = n.kind {
        *legend = Some(Box::new(Legend {
            weight,
            line,
            sample,
            group: Some(LegendGroup::Plan),
            strata: strata.map(str::to_string),
            favourite: weight.is_some_and(|w| (w - 0.2).abs() < 1e-6 || (w - 0.5).abs() < 1e-6 || (w - 0.8).abs() < 1e-6),
            ..Legend::default()
        }));
    }
    let at = d.layers.len();
    let id = d.insert(None, at, n)?;
    Ok(id)
}

fn next_section_label(d: &Document) -> String {
    let mut count = 0;
    d.walk(|n| {
        if !n.is_layer()
            && let Some(name) = &n.name
            && name.starts_with("단면 위치선 ")
        {
            count += 1;
        }
    });
    if count < 26 {
        let byte = b'A' + (count as u8);
        (byte as char).to_string()
    } else {
        format!("A{}", count - 25)
    }
}

fn add_section_line(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.addSectionLine";

    let x1 = f64_req(p, "x1", C)?;
    let y1 = f64_req(p, "y1", C)?;
    let x2 = f64_req(p, "x2", C)?;
    let y2 = f64_req(p, "y2", C)?;

    if !x1.is_finite() || !y1.is_finite() || !x2.is_finite() || !y2.is_finite() {
        return Err(bad(C, "coordinates must be finite numbers"));
    }

    let dx = x2 - x1;
    let dy = y2 - y1;
    let length = dx.hypot(dy);
    if length < 1e-4 {
        return Err(bad(C, "start and end points must not be coincident"));
    }

    let flip = bool_or(p, "flip", false);
    let tick_length = f64_or(p, "tick_length", 12.0).clamp(1.0, 100.0);

    let doc_ref = &s.doc()?.doc;
    let label = match str_param(p, "label") {
        Some(l) if !l.trim().is_empty() => l.trim().to_string(),
        _ => next_section_label(doc_ref),
    };
    let label_prime = format!("{label}'");

    // Unit tangent vector along line P1 -> P2
    let ux = dx / length;
    let uy = dy / length;

    // Normal vector pointing in sight direction
    let (nx, ny) = if flip { (uy, -ux) } else { (-uy, ux) };

    let p1 = Point::new(x1, y1);
    let p2 = Point::new(x2, y2);
    let t1 = Point::new(x1 + nx * tick_length, y1 + ny * tick_length);
    let t2 = Point::new(x2 + nx * tick_length, y2 + ny * tick_length);

    let group_id = s.edit("Add Section Line", |d, _| {
        let layer_id = get_or_create_plan_layer(d, "단면 위치선", Some(0.2), LineType::DashDot, LegendSample::Line, Some("section_line"))?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some(format!("단면 위치선 {label}-{label_prime}"));
        d.insert(Some(layer_id), usize::MAX, group)?;

        // 1. Central dash-dot base line
        let mut base_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.2);
        if let Some(st) = base_app.stroke_mut() {
            st.dash = Some(Dash { pattern: vec![6.0, 2.5, 0.9, 2.5], offset: 0.0, align_corners: true });
        }
        let base_pd = PathData::new(vec![SubPath::polyline(&[p1, p2], false)]);
        let base_node = Node::path(d.alloc_id(), base_pd, base_app);
        d.insert(Some(gid), usize::MAX, base_node)?;

        // 2. Cut tick at P1 with sight arrowhead
        let mut tick1_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 1.0);
        if let Some(st) = tick1_app.stroke_mut() {
            st.end_arrow = Some(Arrowhead::Triangle);
            st.arrow_align = ArrowAlign::Tip;
        }
        let tick1_pd = PathData::new(vec![SubPath::polyline(&[p1, t1], false)]);
        let tick1_node = Node::path(d.alloc_id(), tick1_pd, tick1_app);
        d.insert(Some(gid), usize::MAX, tick1_node)?;

        // 3. Cut tick at P2 with sight arrowhead
        let mut tick2_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 1.0);
        if let Some(st) = tick2_app.stroke_mut() {
            st.end_arrow = Some(Arrowhead::Triangle);
            st.arrow_align = ArrowAlign::Tip;
        }
        let tick2_pd = PathData::new(vec![SubPath::polyline(&[p2, t2], false)]);
        let tick2_node = Node::path(d.alloc_id(), tick2_pd, tick2_app);
        d.insert(Some(gid), usize::MAX, tick2_node)?;

        // 4. Start label (A) near tick 1
        let text_style = CharStyle { size: 10.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let label1_pos = Point::new(t1.x + nx * 7.0 - ux * 3.0, t1.y + ny * 7.0 - uy * 3.0);
        let text1_obj = TextObject::point(label1_pos, &label, text_style.clone());
        let text1_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text1_obj)));
        d.insert(Some(gid), usize::MAX, text1_node)?;

        // 5. End label (A') near tick 2
        let label2_pos = Point::new(t2.x + nx * 7.0 - ux * 3.0, t2.y + ny * 7.0 - uy * 3.0);
        let text2_obj = TextObject::point(label2_pos, &label_prime, text_style);
        let text2_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text2_obj)));
        d.insert(Some(gid), usize::MAX, text2_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "label": label,
        "x1": x1,
        "y1": y1,
        "x2": x2,
        "y2": y2,
        "flip": flip,
    }))
}

fn add_spot_height(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.addSpotHeight";

    let x = f64_req(p, "x", C)?;
    let y = f64_req(p, "y", C)?;

    if !x.is_finite() || !y.is_finite() {
        return Err(bad(C, "coordinates must be finite numbers"));
    }

    let st = s.doc()?;
    let coord = st.doc.coord.clone().unwrap_or_default();

    let elevation = match p.get("elevation").and_then(Value::as_f64).filter(|e| e.is_finite()) {
        Some(e) => e,
        None => {
            let tm_y = coord.pt_to_tm(x, y).1;
            (tm_y * 100.0).round() / 100.0
        }
    };

    let label_text = match str_param(p, "label") {
        Some(l) if !l.trim().is_empty() => l.trim().to_string(),
        _ => format!("{elevation:.2}"),
    };

    let group_id = s.edit("Add Spot Height", |d, _| {
        let layer_id = get_or_create_plan_layer(d, "표고점", None, LineType::Solid, LegendSample::Mark("·152".into()), None)?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some(format!("표고점 {label_text}"));
        d.insert(Some(layer_id), usize::MAX, group)?;

        // 1. Solid black dot (radius 1.6 pt)
        let dot_pd = shapes::ellipse(Rect::new(x - 1.6, y - 1.6, x + 1.6, y + 1.6));
        let dot_app = Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 0.0);
        let dot_node = Node::path(d.alloc_id(), dot_pd, dot_app);
        d.insert(Some(gid), usize::MAX, dot_node)?;

        // 2. Height text (9 pt, offset +3 pt right, -3 pt up in paper coords)
        let style = CharStyle { size: 9.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let text_pos = Point::new(x + 3.0, y - 3.0);
        let text_obj = TextObject::point(text_pos, &label_text, style);
        let text_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text_obj)));
        d.insert(Some(gid), usize::MAX, text_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "elevation": elevation,
        "label": label_text,
    }))
}

fn next_find_number(d: &Document) -> u32 {
    let mut count = 0;
    d.walk(|n| {
        if !n.is_layer()
            && let Some(name) = &n.name
            && name.starts_with("유물 번호 ")
        {
            count += 1;
        }
    });
    count + 1
}

fn add_find_number(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.addFindNumber";

    let x = f64_req(p, "x", C)?;
    let y = f64_req(p, "y", C)?;

    if !x.is_finite() || !y.is_finite() {
        return Err(bad(C, "coordinates must be finite numbers"));
    }

    let doc_ref = &s.doc()?.doc;
    let number = p.get("number").and_then(Value::as_u64).map(|n| n as u32).unwrap_or_else(|| next_find_number(doc_ref));

    let group_id = s.edit("Add Find Number", |d, _| {
        let layer_id =
            get_or_create_plan_layer(d, "유물 출토 위치", Some(0.2), LineType::Solid, LegendSample::Mark("③".into()), Some("artifact_point"))?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some(format!("유물 번호 {number}"));
        d.insert(Some(layer_id), usize::MAX, group)?;

        // 1. White badge circle (radius 6.0 pt, 0.6 pt black stroke)
        let circle_pd = shapes::ellipse(Rect::new(x - 6.0, y - 6.0, x + 6.0, y + 6.0));
        let circle_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::BLACK), 0.6);
        let circle_node = Node::path(d.alloc_id(), circle_pd, circle_app);
        d.insert(Some(gid), usize::MAX, circle_node)?;

        // 2. Centered number text (8 pt)
        let num_str = number.to_string();
        let style = CharStyle { size: 8.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let text_offset_x = if number < 10 { 2.5 } else { 4.5 };
        let text_pos = Point::new(x - text_offset_x, y + 3.0);
        let text_obj = TextObject::point(text_pos, &num_str, style);
        let text_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text_obj)));
        d.insert(Some(gid), usize::MAX, text_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "number": number,
    }))
}

fn next_posthole_number(d: &Document) -> u32 {
    let mut count = 0;
    d.walk(|n| {
        if !n.is_layer()
            && let Some(name) = &n.name
            && name.starts_with("주혈 P")
        {
            count += 1;
        }
    });
    count + 1
}

fn add_posthole(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.addPosthole";

    let x = f64_req(p, "x", C)?;
    let y = f64_req(p, "y", C)?;

    if !x.is_finite() || !y.is_finite() {
        return Err(bad(C, "coordinates must be finite numbers"));
    }

    let radius = f64_or(p, "radius", 13.0).clamp(2.0, 500.0);
    let depth_cm = p.get("depth_cm").and_then(Value::as_i64).map(|d| d as i32);

    let doc_ref = &s.doc()?.doc;
    let number = p.get("number").and_then(Value::as_u64).map(|n| n as u32).unwrap_or_else(|| next_posthole_number(doc_ref));

    let label_text = match depth_cm {
        Some(d) => format!("P{number} (-{d})"),
        None => format!("P{number}"),
    };

    let group_id = s.edit("Add Posthole", |d, _| {
        let layer_id = get_or_create_plan_layer(d, "내부 시설", Some(0.5), LineType::Solid, LegendSample::Area, None)?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some(format!("주혈 P{number}"));
        d.insert(Some(layer_id), usize::MAX, group)?;

        // 1. Posthole outline circle (1.25 pt black stroke, no fill)
        let circle_pd = shapes::ellipse(Rect::new(x - radius, y - radius, x + radius, y + radius));
        let circle_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 1.25);
        let circle_node = Node::path(d.alloc_id(), circle_pd, circle_app);
        d.insert(Some(gid), usize::MAX, circle_node)?;

        // 2. Posthole label text (7.5 pt)
        let style = CharStyle { size: 7.5, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let text_offset_x = (label_text.len() as f64) * 2.2;
        let text_pos = Point::new(x - text_offset_x, y + 2.5);
        let text_obj = TextObject::point(text_pos, &label_text, style);
        let text_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text_obj)));
        d.insert(Some(gid), usize::MAX, text_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "radius": radius,
        "number": number,
        "depth_cm": depth_cm,
    }))
}

fn add_slope_ticks(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.addSlopeTicks";

    let raw_pts = p.get("points").and_then(Value::as_array).ok_or_else(|| bad(C, "missing or invalid `points` array"))?;
    if raw_pts.len() < 2 {
        return Err(bad(C, "`points` must contain at least 2 coordinate pairs"));
    }

    let mut points = Vec::with_capacity(raw_pts.len());
    for item in raw_pts {
        let pair = item.as_array().ok_or_else(|| bad(C, "each point must be [x, y]"))?;
        if pair.len() != 2 {
            return Err(bad(C, "each point must be [x, y]"));
        }
        let x = pair[0].as_f64().filter(|f| f.is_finite()).ok_or_else(|| bad(C, "non-finite point x"))?;
        let y = pair[1].as_f64().filter(|f| f.is_finite()).ok_or_else(|| bad(C, "non-finite point y"))?;
        points.push(Point::new(x, y));
    }

    let tick_length = f64_or(p, "tick_length", 12.0).clamp(1.0, 100.0);
    let spacing = f64_or(p, "spacing", 6.0).clamp(0.5, 50.0);
    let flip = bool_or(p, "flip", false);

    // Compute cumulative segment lengths
    let mut seg_lengths = Vec::with_capacity(points.len() - 1);
    let mut total_length = 0.0;
    for i in 0..points.len() - 1 {
        let d = points[i].distance(points[i + 1]);
        seg_lengths.push(d);
        total_length += d;
    }

    if total_length < 1e-4 {
        return Err(bad(C, "total polyline length is zero"));
    }

    let mut tick_paths = Vec::new();
    let count = (total_length / spacing).floor() as usize;

    for i in 0..=count {
        let target_dist = (i as f64) * spacing;
        let mut accumulated = 0.0;
        let mut cur_pt = points[0];
        let mut normal = (0.0, 1.0);

        for (seg_idx, &len) in seg_lengths.iter().enumerate() {
            if accumulated + len >= target_dist || seg_idx == seg_lengths.len() - 1 {
                let remaining = target_dist - accumulated;
                let t = if len > 1e-6 { (remaining / len).clamp(0.0, 1.0) } else { 0.0 };
                let p0 = points[seg_idx];
                let p1 = points[seg_idx + 1];
                cur_pt = Point::new(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y));

                let dx = (p1.x - p0.x) / len.max(1e-6);
                let dy = (p1.y - p0.y) / len.max(1e-6);
                normal = if flip { (dy, -dx) } else { (-dy, dx) };
                break;
            }
            accumulated += len;
        }

        // Alternating long (1.0x) and short (0.5x) ticks
        let is_even = (i % 2) == 0;
        let current_tick_len = if is_even { tick_length } else { tick_length * 0.5 };
        let end_pt = Point::new(cur_pt.x + normal.0 * current_tick_len, cur_pt.y + normal.1 * current_tick_len);

        tick_paths.push((cur_pt, end_pt));
    }

    let group_id = s.edit("Add Slope Hachure Ticks", |d, _| {
        let layer_id = get_or_create_plan_layer(d, "경사 표시", Some(0.2), LineType::Solid, LegendSample::Area, None)?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some("경사 기호".to_string());
        d.insert(Some(layer_id), usize::MAX, group)?;

        let tick_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.2);
        for (p_start, p_end) in tick_paths {
            let pd = PathData::new(vec![SubPath::polyline(&[p_start, p_end], false)]);
            let tick_node = Node::path(d.alloc_id(), pd, tick_app.clone());
            d.insert(Some(gid), usize::MAX, tick_node)?;
        }

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "ticks": count + 1,
        "length": total_length,
        "flip": flip,
    }))
}

/// 2D Monotone Chain convex hull algorithm.
fn convex_hull(mut pts: Vec<Point>) -> Vec<Point> {
    if pts.len() <= 3 {
        return pts;
    }
    pts.sort_by(|a, b| {
        if (a.x - b.x).abs() > 1e-9 {
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal)
        }
    });
    pts.dedup_by(|a, b| (a.x - b.x).hypot(a.y - b.y) < 1e-9);

    if pts.len() <= 3 {
        return pts;
    }

    let cross = |o: Point, a: Point, b: Point| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);

    let mut lower = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p);
    }

    let mut upper = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Computes Minimum Area Bounding Box (Oriented Bounding Box) for points.
/// Returns (major_dimension, minor_dimension) where major >= minor.
fn min_bounding_box(pts: &[Point]) -> (f64, f64) {
    if pts.len() < 2 {
        return (0.0, 0.0);
    }
    if pts.len() == 2 {
        let d = pts[0].distance(pts[1]);
        return (d, 0.0);
    }

    let hull = convex_hull(pts.to_vec());
    if hull.len() < 2 {
        return (0.0, 0.0);
    }

    let mut min_area = f64::MAX;
    let mut best_dim = (0.0, 0.0);

    for i in 0..hull.len() {
        let j = (i + 1) % hull.len();
        let p1 = hull[i];
        let p2 = hull[j];
        let d = p1.distance(p2);
        if d < 1e-9 {
            continue;
        }

        let ux = (p2.x - p1.x) / d;
        let uy = (p2.y - p1.y) / d;
        let nx = -uy;
        let ny = ux;

        let mut min_u = f64::MAX;
        let mut max_u = f64::MIN;
        let mut min_n = f64::MAX;
        let mut max_n = f64::MIN;

        for p in &hull {
            let proj_u = p.x * ux + p.y * uy;
            let proj_n = p.x * nx + p.y * ny;
            min_u = min_u.min(proj_u);
            max_u = max_u.max(proj_u);
            min_n = min_n.min(proj_n);
            max_n = max_n.max(proj_n);
        }

        let len_u = (max_u - min_u).max(0.0);
        let len_n = (max_n - min_n).max(0.0);
        let area = len_u * len_n;

        if area < min_area {
            min_area = area;
            let major = len_u.max(len_n);
            let minor = len_u.min(len_n);
            best_dim = (major, minor);
        }
    }

    best_dim
}

fn calculate_metrics(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.calculateMetrics";

    let target_ids = match ids_param(p, "ids") {
        Some(v) if !v.is_empty() => v,
        _ => selected_roots(s)?,
    };

    if target_ids.is_empty() {
        return Err(bad(C, "select at least one closed polygon or path"));
    }

    let st = s.doc()?;
    let coord = st.doc.coord.clone().unwrap_or_default();
    let ppm = coord.points_per_meter().max(1e-6);

    let mut all_points = Vec::new();

    for id in &target_ids {
        if let Some(n) = st.doc.node(*id) {
            match &n.kind {
                NodeKind::Path { path, .. } => {
                    let bp = path.to_bezpath();
                    kurbo::flatten(&bp, 0.2, |el| match el {
                        kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => {
                            all_points.push(p);
                        }
                        _ => {}
                    });
                }
                NodeKind::Compound { children, .. } => {
                    for c in children {
                        if let NodeKind::Path { path, .. } = &c.kind {
                            let bp = path.to_bezpath();
                            kurbo::flatten(&bp, 0.2, |el| match el {
                                kurbo::PathEl::MoveTo(p) | kurbo::PathEl::LineTo(p) => {
                                    all_points.push(p);
                                }
                                _ => {}
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if all_points.len() < 3 {
        return Err(bad(C, "target geometry must contain at least 3 points"));
    }

    // 1. Green's Shoelace Area
    let n = all_points.len();
    let mut shoelace_sum = 0.0;
    let mut perimeter_pt = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        shoelace_sum += all_points[i].x * all_points[j].y - all_points[j].x * all_points[i].y;
        perimeter_pt += all_points[i].distance(all_points[j]);
    }
    let area_pt = shoelace_sum.abs() * 0.5;

    // 2. Minimum Bounding Box (Oriented Bounding Box) for Major and Minor axes
    let (major_pt, minor_pt) = min_bounding_box(&all_points);

    // 3. Real-world metric conversion
    let area_m2 = area_pt / (ppm * ppm);
    let perimeter_m = perimeter_pt / ppm;
    let major_axis_m = major_pt / ppm;
    let minor_axis_m = minor_pt / ppm;

    // 4. Update feature node attributes with calculated metrics
    s.edit("Calculate Polygon Metrics", |d, _| {
        for id in &target_ids {
            if let Some(node) = d.node_mut(*id) {
                node.edit_attrs(|a| {
                    let upsert = |data: &mut Vec<(String, String)>, key: &str, val: String| {
                        if let Some(entry) = data.iter_mut().find(|(k, _)| k == key) {
                            entry.1 = val;
                        } else {
                            data.push((key.to_string(), val));
                        }
                    };
                    upsert(&mut a.data, "feature_area_m2", format!("{area_m2:.2}"));
                    upsert(&mut a.data, "feature_perimeter_m", format!("{perimeter_m:.2}"));
                    upsert(&mut a.data, "feature_major_m", format!("{major_axis_m:.2}"));
                    upsert(&mut a.data, "feature_minor_m", format!("{minor_axis_m:.2}"));
                });
            }
        }
        Ok(())
    })?;

    Ok(json!({
        "ids": target_ids.iter().map(|i| i.0).collect::<Vec<_>>(),
        "area_m2": (area_m2 * 100.0).round() / 100.0,
        "perimeter_m": (perimeter_m * 100.0).round() / 100.0,
        "major_axis_m": (major_axis_m * 100.0).round() / 100.0,
        "minor_axis_m": (minor_axis_m * 100.0).round() / 100.0,
        "area_pt": area_pt,
        "perimeter_pt": perimeter_pt,
    }))
}

fn set_feature_props(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "feature.setProps";

    let target_ids = match ids_param(p, "ids") {
        Some(v) if !v.is_empty() => v,
        _ => selected_roots(s)?,
    };

    if target_ids.is_empty() {
        return Err(bad(C, "select at least one object"));
    }

    let kind_opt = str_param(p, "kind").map(str::to_string);
    let number_opt = match p.get("number") {
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        _ => None,
    };
    let period_opt = str_param(p, "period").map(str::to_string);

    s.edit("Set Feature Properties", |d, _| {
        for id in &target_ids {
            let Some(node) = d.node_mut(*id) else { continue };

            // Update attributes data
            node.edit_attrs(|a| {
                let upsert = |data: &mut Vec<(String, String)>, key: &str, val_opt: &Option<String>| {
                    if let Some(val) = val_opt {
                        if let Some(entry) = data.iter_mut().find(|(k, _)| k == key) {
                            entry.1 = val.clone();
                        } else {
                            data.push((key.to_string(), val.clone()));
                        }
                    }
                };
                upsert(&mut a.data, "feature_kind", &kind_opt);
                upsert(&mut a.data, "feature_number", &number_opt);
                upsert(&mut a.data, "feature_period", &period_opt);
            });

            // Update node display name if kind or number is supplied
            let cur_kind = kind_opt
                .as_deref()
                .or_else(|| node.attrs.as_ref().and_then(|a| a.data.iter().find(|(k, _)| k == "feature_kind").map(|(_, v)| v.as_str())));
            let cur_num = number_opt
                .as_deref()
                .or_else(|| node.attrs.as_ref().and_then(|a| a.data.iter().find(|(k, _)| k == "feature_number").map(|(_, v)| v.as_str())));

            match (cur_kind, cur_num) {
                (Some(k), Some(n)) if !k.is_empty() && !n.is_empty() => {
                    let formatted_num = if n.ends_with('호') { n.to_string() } else { format!("{n}호") };
                    node.name = Some(format!("{k} {formatted_num}"));
                }
                (Some(k), _) if !k.is_empty() => {
                    node.name = Some(k.to_string());
                }
                (_, Some(n)) if !n.is_empty() => {
                    let formatted_num = if n.ends_with('호') { n.to_string() } else { format!("{n}호") };
                    node.name = Some(formatted_num);
                }
                _ => {}
            }
        }
        Ok(())
    })?;

    Ok(json!({
        "ids": target_ids.iter().map(|i| i.0).collect::<Vec<_>>(),
        "kind": kind_opt,
        "number": number_opt,
        "period": period_opt,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_with_doc() -> Session {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 800, "height": 600})).unwrap();
        s
    }

    #[test]
    fn test_add_section_line_and_undo_redo() {
        let mut s = session_with_doc();
        let res = s
            .execute(
                "drawing.addSectionLine",
                &json!({
                    "x1": 100.0,
                    "y1": 100.0,
                    "x2": 300.0,
                    "y2": 100.0,
                    "label": "A",
                    "flip": false,
                    "tick_length": 12.0
                }),
            )
            .unwrap();

        let id = res["id"].as_u64().unwrap();
        assert_eq!(res["label"], "A");

        {
            let doc = &s.doc().unwrap().doc;
            let node = doc.node(NodeId(id)).unwrap();
            assert_eq!(node.name.as_deref(), Some("단면 위치선 A-A'"));
            let children = node.children().unwrap();
            assert_eq!(children.len(), 5); // 1 base line, 2 ticks, 2 text labels
        }

        // Test undo and redo
        s.execute("edit.undo", &json!({})).unwrap();
        {
            let doc = &s.doc().unwrap().doc;
            assert!(doc.node(NodeId(id)).is_none());
        }

        s.execute("edit.redo", &json!({})).unwrap();
        {
            let doc = &s.doc().unwrap().doc;
            assert!(doc.node(NodeId(id)).is_some());
        }
    }

    #[test]
    fn test_add_spot_height() {
        let mut s = session_with_doc();
        let res = s
            .execute(
                "drawing.addSpotHeight",
                &json!({
                    "x": 250.0,
                    "y": 150.0,
                    "elevation": 152.36
                }),
            )
            .unwrap();

        let id = res["id"].as_u64().unwrap();
        assert_eq!(res["elevation"], 152.36);
        assert_eq!(res["label"], "152.36");

        let doc = &s.doc().unwrap().doc;
        let node = doc.node(NodeId(id)).unwrap();
        assert_eq!(node.name.as_deref(), Some("표고점 152.36"));
    }

    #[test]
    fn test_add_find_number_auto_increment() {
        let mut s = session_with_doc();
        let res1 = s.execute("drawing.addFindNumber", &json!({ "x": 100.0, "y": 100.0 })).unwrap();
        assert_eq!(res1["number"], 1);

        let res2 = s.execute("drawing.addFindNumber", &json!({ "x": 120.0, "y": 100.0 })).unwrap();
        assert_eq!(res2["number"], 2);

        let res3 = s.execute("drawing.addFindNumber", &json!({ "x": 140.0, "y": 100.0, "number": 10 })).unwrap();
        assert_eq!(res3["number"], 10);
    }

    #[test]
    fn test_add_posthole() {
        let mut s = session_with_doc();
        let res = s
            .execute(
                "drawing.addPosthole",
                &json!({
                    "x": 200.0,
                    "y": 300.0,
                    "radius": 13.0,
                    "depth_cm": 25
                }),
            )
            .unwrap();

        let id = res["id"].as_u64().unwrap();
        assert_eq!(res["number"], 1);
        assert_eq!(res["depth_cm"], 25);

        let doc = &s.doc().unwrap().doc;
        let node = doc.node(NodeId(id)).unwrap();
        assert_eq!(node.name.as_deref(), Some("주혈 P1"));
    }

    #[test]
    fn test_add_slope_ticks() {
        let mut s = session_with_doc();
        let res = s
            .execute(
                "drawing.addSlopeTicks",
                &json!({
                    "points": [[100.0, 100.0], [200.0, 100.0]],
                    "tick_length": 12.0,
                    "spacing": 10.0,
                    "flip": false
                }),
            )
            .unwrap();

        let id = res["id"].as_u64().unwrap();
        assert_eq!(res["ticks"], 11); // 100 pt / 10 = 10 intervals -> 11 ticks

        let doc = &s.doc().unwrap().doc;
        let node = doc.node(NodeId(id)).unwrap();
        assert_eq!(node.name.as_deref(), Some("경사 기호"));
    }

    #[test]
    fn test_calculate_metrics_rectangle() {
        let mut s = session_with_doc();

        // Set document coordinate scale to 1:20
        s.execute(
            "drawing.setCoord",
            &json!({
                "scale": 20.0,
                "epsg": 5186,
                "origin_x": 148_000.0,
                "origin_y": 98_150.0
            }),
        )
        .unwrap();

        // 1 meter at 1:20: (1000 / 20) * (72 / 25.4) = 50 * 2.83464567 = 141.73228346 pt
        let ppm = (1000.0 / 20.0) * (72.0 / 25.4);
        let w_m = 4.0;
        let h_m = 3.0;
        let w_pt = w_m * ppm;
        let h_pt = h_m * ppm;

        let res = s
            .execute(
                "path.create",
                &json!({
                    "anchors": [
                        { "x": 100.0, "y": 100.0 },
                        { "x": 100.0 + w_pt, "y": 100.0 },
                        { "x": 100.0 + w_pt, "y": 100.0 + h_pt },
                        { "x": 100.0, "y": 100.0 + h_pt }
                    ],
                    "closed": true
                }),
            )
            .unwrap();
        let path_id = res["id"].as_u64().unwrap();

        let m_res = s
            .execute(
                "drawing.calculateMetrics",
                &json!({
                    "ids": [path_id]
                }),
            )
            .unwrap();

        // Area: 4m * 3m = 12 m²
        assert!((m_res["area_m2"].as_f64().unwrap() - 12.0).abs() < 0.05);
        // Perimeter: 2 * (4m + 3m) = 14 m
        assert!((m_res["perimeter_m"].as_f64().unwrap() - 14.0).abs() < 0.05);
        // Major: 4.0 m
        assert!((m_res["major_axis_m"].as_f64().unwrap() - 4.0).abs() < 0.05);
        // Minor: 3.0 m
        assert!((m_res["minor_axis_m"].as_f64().unwrap() - 3.0).abs() < 0.05);

        // Verify attributes were saved on the node
        let doc = &s.doc().unwrap().doc;
        let node = doc.node(NodeId(path_id)).unwrap();
        let attrs = node.attrs.as_ref().unwrap();
        assert!(attrs.data.iter().any(|(k, v)| k == "feature_area_m2" && v.starts_with("12.")));
    }

    #[test]
    fn test_set_feature_props() {
        let mut s = session_with_doc();
        let res = s
            .execute(
                "path.create",
                &json!({
                    "anchors": [{ "x": 50.0, "y": 50.0 }, { "x": 150.0, "y": 50.0 }],
                    "closed": false
                }),
            )
            .unwrap();
        let id = res["id"].as_u64().unwrap();

        s.execute(
            "feature.setProps",
            &json!({
                "ids": [id],
                "kind": "주거지",
                "number": "5",
                "period": "청동기"
            }),
        )
        .unwrap();

        let doc = &s.doc().unwrap().doc;
        let node = doc.node(NodeId(id)).unwrap();
        assert_eq!(node.name.as_deref(), Some("주거지 5호"));

        let attrs = node.attrs.as_ref().unwrap();
        assert!(attrs.data.iter().any(|(k, v)| k == "feature_kind" && v == "주거지"));
        assert!(attrs.data.iter().any(|(k, v)| k == "feature_number" && v == "5"));
        assert!(attrs.data.iter().any(|(k, v)| k == "feature_period" && v == "청동기"));
    }
}
