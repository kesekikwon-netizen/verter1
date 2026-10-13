//! Elevation level line generation engine commands (Trowel, J015).
//!
//! Generates standard 50 cm (0.5 pt) and 10 cm (0.2 pt) level lines clipped outside
//! section ground/obstacle areas with left-aligned "152.5m" elevation labels.

use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::{Legend, LegendGroup, LegendSample, LineType};
use vectorcraft_doc::{Appearance, CharStyle, Document, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::{BezPath, ParamCurve, PathData, Point, Rect, Shape, SubPath, subtract_horizontal_obstacles};

use super::layer::make_layer;
use super::*;

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        "drawing.makeLevelLines",
        "Make Level Lines",
        [],
        None,
        "{base_elevation?: number, interval_cm?: number, min_elevation?: number, max_elevation?: number, left_x?: number, right_x?: number} generate 50 cm (0.5 pt) and 10 cm (0.2 pt) elevation level lines outside obstacles (Trowel, J015) → {lines_50cm, lines_10cm, labels, min_elevation, max_elevation}",
        has_doc,
        make_level_lines
    )]
}

fn get_or_create_layer(d: &mut Document, name: &str, weight: Option<f64>, sample: LegendSample) -> Result<NodeId> {
    if let Some(l) = d.layers.iter().find(|l| l.display_name().trim() == name) {
        return Ok(l.id);
    }
    let mut n = make_layer(d, Some(name));
    if let NodeKind::Layer { ref mut legend, .. } = n.kind {
        *legend = Some(Box::new(Legend {
            weight,
            line: LineType::Solid,
            sample,
            group: Some(LegendGroup::Section),
            favourite: weight.is_some_and(|w| (w - 0.5).abs() < 1e-6),
            ..Legend::default()
        }));
    }
    let at = d.layers.len();
    let id = d.insert(None, at, n)?;
    Ok(id)
}

fn clear_layer_children(d: &mut Document, layer_id: NodeId) {
    if let Some(layer) = d.node_mut(layer_id)
        && let Some(children) = layer.children_mut()
    {
        children.clear();
    }
}

fn collect_closed_paths_in_node(node: &Node, d: &Document, out: &mut Vec<BezPath>) {
    if !d.is_editable(node.id) {
        return;
    }
    match &node.kind {
        NodeKind::Path { path, .. } if path.is_closed() => {
            out.push(path.to_bezpath());
        }
        _ => {
            if let Some(children) = node.children() {
                for c in children {
                    collect_closed_paths_in_node(c, d, out);
                }
            }
        }
    }
}

fn collect_ground_paths_in_node(node: &Node, d: &Document, bottom_y: f64, out: &mut Vec<BezPath>) {
    if !d.is_editable(node.id) {
        return;
    }
    match &node.kind {
        NodeKind::Path { path, .. } => {
            let bp = path.to_bezpath();
            if path.is_closed() {
                out.push(bp);
            } else {
                let b = bp.bounding_box();
                if b.width() > 1.0 {
                    let mut ground = bp;
                    let segs: Vec<_> = ground.segments().collect();
                    if let (Some(first_seg), Some(last_seg)) = (segs.first(), segs.last()) {
                        let p0 = first_seg.eval(0.0);
                        let p1 = last_seg.eval(1.0);
                        ground.line_to(Point::new(p1.x, bottom_y));
                        ground.line_to(Point::new(p0.x, bottom_y));
                        ground.close_path();
                        out.push(ground);
                    }
                }
            }
        }
        _ => {
            if let Some(children) = node.children() {
                for c in children {
                    collect_ground_paths_in_node(c, d, bottom_y, out);
                }
            }
        }
    }
}

fn section_content_bounds(d: &Document) -> Option<Rect> {
    let mut bounds: Option<Rect> = None;
    for l in &d.layers {
        let name_str = l.display_name();
        let name = name_str.trim();
        // Ignore level line layers and template/desca layers when calculating bounds of section art
        if name.starts_with("해발") || name.contains("desca") || name.contains("밑그림") {
            continue;
        }
        if let Some(b) = d.bounds_of(&[l.id], true) {
            bounds = match bounds {
                Some(prev) => Some(prev.union(b)),
                None => Some(b),
            };
        }
    }
    bounds
}

fn make_level_lines(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.makeLevelLines";

    let st = s.doc()?;
    let coord = st.doc.coord.clone().unwrap_or_default();
    let ab_rect = st.doc.artboards.first().map(|a| a.rect).unwrap_or(Rect::new(0.0, 0.0, 1190.551, 841.890));

    // 1. Determine bounding box for level lines
    let art_bounds = section_content_bounds(&st.doc);
    let (left_x, right_x, y_top, y_bottom) = match art_bounds {
        Some(b) if b.width() > 10.0 && b.height() > 10.0 => {
            let lx = p.get("left_x").and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or(b.x0 - 40.0).max(ab_rect.x0 + 20.0);
            let rx = p.get("right_x").and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or(b.x1 + 40.0).min(ab_rect.x1 - 20.0);
            (lx, rx, b.y0, b.y1)
        }
        _ => {
            let lx = p.get("left_x").and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or(ab_rect.x0 + 50.0);
            let rx = p.get("right_x").and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or(ab_rect.x1 - 50.0);
            (lx, rx, ab_rect.y0 + 100.0, ab_rect.y1 - 100.0)
        }
    };

    if !left_x.is_finite() || !right_x.is_finite() || right_x <= left_x {
        return Err(bad(C, "right_x must be greater than left_x and both must be finite"));
    }

    // 2. Determine elevation range (meters)
    // Note: in paper coordinates, smaller Y is higher elevation, larger Y is lower elevation.
    let elev_at_bottom = coord.pt_to_tm(0.0, y_bottom).1;
    let elev_at_top = coord.pt_to_tm(0.0, y_top).1;
    let lowest_detected = elev_at_bottom.min(elev_at_top);
    let highest_detected = elev_at_bottom.max(elev_at_top);

    let min_elev = p.get("min_elevation").and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or_else(|| {
        // Round lowest down to nearest 10 cm (0.1 m)
        (lowest_detected * 10.0).floor() / 10.0
    });

    let max_elev = p.get("max_elevation").and_then(Value::as_f64).filter(|x| x.is_finite()).unwrap_or_else(|| {
        // First 50 cm boundary strictly above or at highest detected elevation
        ((highest_detected + 1e-4) / 0.5).ceil() * 0.5
    });

    if !min_elev.is_finite() || !max_elev.is_finite() || max_elev < min_elev {
        return Err(bad(C, format!("max_elevation ({max_elev}) cannot be less than min_elevation ({min_elev}) and both must be finite")));
    }

    let span = max_elev - min_elev;
    if span > 100.0 {
        return Err(bad(C, format!("elevation span {span:.1} m is too large (maximum 100 m)")));
    }

    // 3. Collect obstacles
    let bottom_ground_y = y_bottom + 50.0;
    let mut obstacles = Vec::new();
    for l in &st.doc.layers {
        let name_str = l.display_name();
        let name = name_str.trim();
        if name.contains("단면선") {
            collect_ground_paths_in_node(l, &st.doc, bottom_ground_y, &mut obstacles);
        } else if name.contains("잘린 돌")
            || name.contains("보이는 돌")
            || name.contains("교란")
            || name.contains("생토")
            || name.contains("토층선")
            || name.contains("유물")
            || name.contains("기와")
        {
            collect_closed_paths_in_node(l, &st.doc, &mut obstacles);
        }
    }

    // 4. Edit session to generate level lines
    let mut count_50 = 0usize;
    let mut count_10 = 0usize;
    let mut count_labels = 0usize;

    s.edit("Make Level Lines", |d, _| {
        // Prepare target layers
        let l_50_id = get_or_create_layer(d, "해발선 50 cm", Some(0.5), LegendSample::Line)?;
        let l_10_id = get_or_create_layer(d, "해발선 10 cm", Some(0.2), LegendSample::Line)?;
        let l_num_id = get_or_create_layer(d, "해발 숫자", None, LegendSample::Mark("152.5m".into()))?;

        // Clear existing generated lines in these layers (idempotency)
        clear_layer_children(d, l_50_id);
        clear_layer_children(d, l_10_id);
        clear_layer_children(d, l_num_id);

        let step = 0.1; // 10 cm step in meters
        let total_steps = (((max_elev - min_elev) / step).round() as usize).min(1000);

        for i in 0..=total_steps {
            let elev = min_elev + (i as f64) * step;
            let y_paper = coord.tm_to_pt(0.0, elev).1;

            // Determine if 50 cm line or 10 cm line
            let is_50cm = ((elev * 10.0).round() as i64).rem_euclid(5) == 0;
            let (target_layer, weight) = if is_50cm { (l_50_id, 0.5) } else { (l_10_id, 0.2) };

            // Subtract obstacles
            let intervals = subtract_horizontal_obstacles(y_paper, (left_x, right_x), &obstacles, 1.0);

            let mut has_drawn_segment = false;
            for (x0, x1) in intervals {
                let stroke = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), weight);
                let pd = PathData::new(vec![SubPath::polyline(&[Point::new(x0, y_paper), Point::new(x1, y_paper)], false)]);
                let n = Node::path(d.alloc_id(), pd, stroke);
                d.insert(Some(target_layer), usize::MAX, n)?;
                has_drawn_segment = true;
            }

            if has_drawn_segment {
                if is_50cm {
                    count_50 += 1;
                } else {
                    count_10 += 1;
                }
            }

            // Elevation number label at left for 50 cm lines
            if is_50cm {
                let label_text = if (elev - elev.round()).abs() < 1e-4 { format!("{:.1}m", elev) } else { format!("{:.2}m", elev) };
                let style = CharStyle { size: 8.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
                let text_pos = Point::new(left_x - 34.0, y_paper + 3.0);
                let text_obj = TextObject::point(text_pos, &label_text, style);
                let text_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text_obj)));
                d.insert(Some(l_num_id), usize::MAX, text_node)?;
                count_labels += 1;
            }
        }

        Ok(())
    })?;

    Ok(json!({
        "lines_50cm": count_50,
        "lines_10cm": count_10,
        "labels": count_labels,
        "min_elevation": min_elev,
        "max_elevation": max_elev,
    }))
}
