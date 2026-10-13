//! Commands for archaeological sheet layout and PDF export elements (J012):
//! North arrow (도북 방위), composite graphic scale bar (복합 축척 막대),
//! standard archaeological title block (국가유산 표준 표제란),
//! extracted legend table (자동 추출 범례표), and sheet properties.

use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::{Legend, LegendSample, LineType};
use vectorcraft_doc::{Appearance, CharStyle, Document, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::{PathData, Point, Rect, SubPath, shapes};

use super::layer::make_layer;
use super::*;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "sheet.addNorthArrow",
            "Add North Arrow",
            [],
            None,
            "{x?: number, y?: number, rotation?: number, size_pt?: number} add standard 28mm needle north arrow with 'N' mark to '방위표' layer (Trowel, J012) → {id, x, y, size_pt}",
            has_doc,
            add_north_arrow
        ),
        cmd!(
            "sheet.addScaleBar",
            "Add Graphic Scale Bar",
            [],
            None,
            "{x?: number, y?: number, scale_ratio?: number, total_m?: number, unit?: string} add composite graphic scale bar (SCALE 1:S) with alternating blocks (Trowel, J012) → {id, x, y, scale_ratio, length_pt}",
            has_doc,
            add_scale_bar
        ),
        cmd!(
            "sheet.addTitleBlock",
            "Add Standard Title Block",
            [],
            None,
            "{x?: number, y?: number, site?: string, title?: string, scale?: string, institute?: string, date?: string, plate_no?: string} add standard 4-row archaeological title block to '표제란' layer (Trowel, J012) → {id, x, y, width, height}",
            has_doc,
            add_title_block
        ),
        cmd!(
            "sheet.generateLegendTable",
            "Generate Sheet Legend Table",
            [],
            None,
            "{x?: number, y?: number, width?: number} collect active drawing layers and generate sheet legend table on '범례표' layer (Trowel, J012) → {id, x, y, rows}",
            has_doc,
            generate_legend_table
        ),
        cmd!(
            "sheet.setProps",
            "Set Sheet Properties",
            [],
            None,
            "{title?: string, plate_no?: string, site?: string, institute?: string, scale?: string, date?: string} set sheet publication metadata attributes on document (Trowel, J012) → {title, plate_no, site, institute, scale, date}",
            has_doc,
            set_sheet_props
        ),
    ]
}

pub(crate) fn get_or_create_sheet_layer(d: &mut Document, name: &str, weight: Option<f64>, line: LineType, sample: LegendSample) -> Result<NodeId> {
    if let Some(l) = d.layers.iter().find(|l| l.display_name().trim() == name) {
        return Ok(l.id);
    }
    let mut n = make_layer(d, Some(name));
    if let NodeKind::Layer { ref mut legend, .. } = n.kind {
        *legend = Some(Box::new(Legend { weight, line, sample, group: None, strata: None, favourite: true, ..Legend::default() }));
    }
    let at = d.layers.len();
    let id = d.insert(None, at, n)?;
    Ok(id)
}

/// Adds standard 28 mm needle north arrow with 'N' mark to '방위표' layer.
fn add_north_arrow(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "sheet.addNorthArrow";

    let (default_x, default_y) = if let Some(doc) = s.active() {
        if let Some(ab) = doc.doc.artboards.first() { (ab.rect.x1 - 80.0, ab.rect.y0 + 80.0) } else { (700.0, 80.0) }
    } else {
        (700.0, 80.0)
    };

    let x = p.get("x").and_then(Value::as_f64).unwrap_or(default_x);
    let y = p.get("y").and_then(Value::as_f64).unwrap_or(default_y);
    let size_pt = p.get("size_pt").and_then(Value::as_f64).unwrap_or(79.37); // 28 mm

    if !x.is_finite() || !y.is_finite() || !size_pt.is_finite() || !(10.0..=50_000.0).contains(&size_pt) || x.abs() > 100_000.0 || y.abs() > 100_000.0
    {
        return Err(bad(C, "invalid coordinates or size for north arrow"));
    }

    let group_id = s.edit("Add North Arrow", |d, _| {
        let layer_id = get_or_create_sheet_layer(d, "방위표", Some(0.5), LineType::Solid, LegendSample::Line)?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some("도북 방위 (North Arrow)".into());

        d.insert(Some(layer_id), usize::MAX, group)?;

        let half_w = size_pt * 0.16;
        let apex_y = y - size_pt * 0.5;
        let base_y = y + size_pt * 0.35;
        let notch_y = y + size_pt * 0.2;

        // 1. Left half (Solid black fill)
        let left_pts = [Point::new(x, apex_y), Point::new(x - half_w, base_y), Point::new(x, notch_y)];
        let left_pd = PathData::new(vec![SubPath::polyline(&left_pts, true)]);
        let left_app = Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 0.0);
        let lid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(lid, left_pd, left_app))?;

        // 2. Right half (White fill + 0.5 pt black border)
        let right_pts = [Point::new(x, apex_y), Point::new(x, notch_y), Point::new(x + half_w, base_y)];
        let right_pd = PathData::new(vec![SubPath::polyline(&right_pts, true)]);
        let right_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::BLACK), 0.5);
        let rid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(rid, right_pd, right_app))?;

        // 3. Center divider line (1.0 pt)
        let center_line = shapes::line(Point::new(x, apex_y), Point::new(x, y + size_pt * 0.45));
        let cl_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.8);
        let clid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(clid, center_line, cl_app))?;

        // 4. North 'N' label
        let text_style = CharStyle { size: 11.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let text_pos = Point::new(x - 4.5, apex_y - 6.0);
        let text_obj = TextObject::point(text_pos, "N", text_style);
        let tid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(tid, NodeKind::Text(Box::new(text_obj))))?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "size_pt": size_pt,
    }))
}

/// Adds composite graphic scale bar (SCALE 1:S) with alternating blocks.
fn add_scale_bar(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "sheet.addScaleBar";

    let (default_x, default_y) = (80.0, 520.0);
    let x = p.get("x").and_then(Value::as_f64).unwrap_or(default_x);
    let y = p.get("y").and_then(Value::as_f64).unwrap_or(default_y);
    let scale_ratio = p.get("scale_ratio").and_then(Value::as_f64).unwrap_or(40.0); // 1:40
    let total_m = p.get("total_m").and_then(Value::as_f64).unwrap_or(5.0); // 5 meters

    if !x.is_finite()
        || !y.is_finite()
        || !scale_ratio.is_finite()
        || scale_ratio <= 0.0
        || scale_ratio > 100_000.0
        || !total_m.is_finite()
        || total_m <= 0.0
        || total_m > 100_000.0
        || x.abs() > 100_000.0
        || y.abs() > 100_000.0
    {
        return Err(bad(C, "invalid scale bar parameters"));
    }

    // Points per meter on paper for scale 1:S:
    // 1 m in real world = 1000 mm / S on paper.
    // 1 mm = 72.0 / 25.4 pt ≈ 2.83464567 pt.
    let ppm = (1000.0 / scale_ratio) * (72.0 / 25.4);
    let total_length_pt = ppm * total_m;

    let group_id = s.edit("Add Graphic Scale Bar", |d, _| {
        let layer_id = get_or_create_sheet_layer(d, "축척 막대", Some(0.5), LineType::Solid, LegendSample::Line)?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some(format!("축척 막대 (SCALE 1:{scale_ratio:.0})"));

        d.insert(Some(layer_id), usize::MAX, group)?;

        let bar_h = 5.0;

        let black_app = Appearance::basic(Paint::solid(Color::BLACK), Paint::solid(Color::BLACK), 0.5);
        let white_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::BLACK), 0.5);
        let text_style = CharStyle { size: 8.5, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };

        // Segments: 0 to 1m, 1 to 2m, 2 to 3m, 3 to 5m
        let segments = [(0.0, 1.0, true), (1.0, 2.0, false), (2.0, 3.0, true), (3.0, total_m, false)];

        for (m_start, m_end, is_black) in segments {
            if m_start >= total_m {
                break;
            }
            let actual_end = m_end.min(total_m);
            let seg_x1 = x + m_start * ppm;
            let seg_x2 = x + actual_end * ppm;

            let rect_pd = shapes::rectangle(Rect::new(seg_x1, y, seg_x2, y + bar_h));
            let app = if is_black { black_app.clone() } else { white_app.clone() };
            let sid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::path(sid, rect_pd, app))?;

            // Division number text above start
            let num_pos = Point::new(seg_x1 - 3.0, y - 4.0);
            let num_text = format!("{m_start:.0}");
            let tid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(tid, NodeKind::Text(Box::new(TextObject::point(num_pos, &num_text, text_style.clone())))))?;
        }

        // Final division number text + unit
        let end_x = x + total_length_pt;
        let end_pos = Point::new(end_x - 3.0, y - 4.0);
        let end_text = format!("{total_m:.0} m");
        let etid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(etid, NodeKind::Text(Box::new(TextObject::point(end_pos, &end_text, text_style.clone())))))?;

        // Scale ratio label below bar: "SCALE 1:40"
        let scale_label = format!("SCALE 1 : {scale_ratio:.0}");
        let label_pos = Point::new(x + total_length_pt * 0.5 - 24.0, y + bar_h + 12.0);
        let stid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(stid, NodeKind::Text(Box::new(TextObject::point(label_pos, &scale_label, text_style)))))?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "scale_ratio": scale_ratio,
        "length_pt": total_length_pt,
    }))
}

/// Adds standard 4-row archaeological title block to '표제란' layer.
fn add_title_block(s: &mut Session, p: &Value) -> Result<Value> {
    let (default_x, default_y) = (450.0, 480.0);
    let x = p.get("x").and_then(Value::as_f64).unwrap_or(default_x);
    let y = p.get("y").and_then(Value::as_f64).unwrap_or(default_y);

    if !x.is_finite() || !y.is_finite() || x.abs() > 100_000.0 || y.abs() > 100_000.0 {
        return Err(bad("sheet.addTitleBlock", "invalid title block coordinates"));
    }

    let site = p.get("site").and_then(Value::as_str).unwrap_or("경주 황오동 고분군 발굴조사");
    let title = p.get("title").and_then(Value::as_str).unwrap_or("1호 주거지 평면 및 토층 단면도");
    let scale = p.get("scale").and_then(Value::as_str).unwrap_or("1:40 (단면 1:20)");
    let institute = p.get("institute").and_then(Value::as_str).unwrap_or("국립문화유산연구원");
    let date = p.get("date").and_then(Value::as_str).unwrap_or("2026. 10");
    let plate_no = p.get("plate_no").and_then(Value::as_str).unwrap_or("PLATE 3");

    let w = 320.0; // ~113 mm
    let h = 80.0; // ~28 mm
    let row_h = h / 4.0;
    let label_w = 64.0;

    let group_id = s.edit("Add Standard Title Block", |d, _| {
        let layer_id = get_or_create_sheet_layer(d, "표제란", Some(1.0), LineType::Solid, LegendSample::Line)?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some(format!("표제란 ({plate_no})"));

        let mut attrs = group.attrs.take().unwrap_or_default();
        attrs.data.push(("sheet:plate_no".into(), plate_no.into()));
        attrs.data.push(("sheet:title".into(), title.into()));
        attrs.data.push(("sheet:site".into(), site.into()));
        attrs.data.push(("sheet:scale".into(), scale.into()));
        attrs.data.push(("sheet:institute".into(), institute.into()));
        attrs.data.push(("sheet:date".into(), date.into()));
        group.attrs = Some(attrs);

        d.insert(Some(layer_id), usize::MAX, group)?;

        // Outer double border (top/bottom 1.5 pt, left/right 1.0 pt)
        let outer_box = shapes::rectangle(Rect::new(x, y, x + w, y + h));
        let outer_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::BLACK), 1.5);
        let obid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(obid, outer_box, outer_app))?;

        // Inner horizontal dividers (0.5 pt)
        let inner_line_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.5);
        for i in 1..4 {
            let ry = y + (i as f64) * row_h;
            let line = shapes::line(Point::new(x, ry), Point::new(x + w, ry));
            let lid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::path(lid, line, inner_line_app.clone()))?;
        }

        // Vertical divider for label column
        let vline = shapes::line(Point::new(x + label_w, y), Point::new(x + label_w, y + h));
        let vlid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(vlid, vline, inner_line_app))?;

        let label_style =
            CharStyle { size: 8.5, fill: Paint::solid(Color::rgb8(0x4B, 0x55, 0x63)), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let value_style = CharStyle { size: 9.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };

        let rows = [("유적명", site), ("도면명", title), ("축  척", scale), ("조사기관", &format!("{institute}  ({date})"))];

        for (idx, (lbl, val)) in rows.iter().enumerate() {
            let cy = y + (idx as f64) * row_h + row_h * 0.5 + 3.0;

            // Label text
            let l_pos = Point::new(x + 10.0, cy);
            let l_tid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(l_tid, NodeKind::Text(Box::new(TextObject::point(l_pos, lbl, label_style.clone())))))?;

            // Value text
            let v_pos = Point::new(x + label_w + 10.0, cy);
            let v_tid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(v_tid, NodeKind::Text(Box::new(TextObject::point(v_pos, val, value_style.clone())))))?;
        }

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "width": w,
        "height": h,
        "plate_no": plate_no,
    }))
}

/// Collects active drawing layers and generates sheet legend table on '범례표' layer.
fn generate_legend_table(s: &mut Session, p: &Value) -> Result<Value> {
    let (default_x, default_y) = (80.0, 360.0);
    let x = p.get("x").and_then(Value::as_f64).unwrap_or(default_x);
    let y = p.get("y").and_then(Value::as_f64).unwrap_or(default_y);
    let w = p.get("width").and_then(Value::as_f64).unwrap_or(180.0);

    if !x.is_finite() || !y.is_finite() || !w.is_finite() || !(20.0..=10_000.0).contains(&w) || x.abs() > 100_000.0 || y.abs() > 100_000.0 {
        return Err(bad("sheet.generateLegendTable", "invalid legend table parameters"));
    }

    let doc = s.active().ok_or_else(|| bad("sheet.generateLegendTable", "no active document"))?;

    // Collect layers with meaningful names (excluding sheet elements)
    let mut legend_items = Vec::new();
    for l in &doc.doc.layers {
        let name = l.display_name().trim().to_string();
        if !name.is_empty() && name != "표제란" && name != "방위표" && name != "축척 막대" && name != "범례표" {
            legend_items.push(name);
        }
    }

    if legend_items.is_empty() {
        legend_items.push("유구 윤곽선".into());
        legend_items.push("토층 경계선".into());
        legend_items.push("소토면".into());
    }

    let row_h = 18.0;
    let header_h = 24.0;
    let total_h = header_h + (legend_items.len() as f64) * row_h + 8.0;

    let group_id = s.edit("Generate Sheet Legend Table", |d, _| {
        let layer_id = get_or_create_sheet_layer(d, "범례표", Some(1.0), LineType::Solid, LegendSample::Line)?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some("도판 범례표 (Legend Table)".into());

        d.insert(Some(layer_id), usize::MAX, group)?;

        // Box border
        let rect_pd = shapes::rectangle(Rect::new(x, y, x + w, y + total_h));
        let box_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::BLACK), 1.0);
        let bid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(bid, rect_pd, box_app))?;

        // Header: "범    례 (LEGEND)"
        let head_style = CharStyle { size: 9.5, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let head_pos = Point::new(x + w * 0.5 - 30.0, y + 16.0);
        let htid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(htid, NodeKind::Text(Box::new(TextObject::point(head_pos, "범  례 (LEGEND)", head_style)))))?;

        // Header bottom divider line
        let hline = shapes::line(Point::new(x, y + header_h), Point::new(x + w, y + header_h));
        let hlid = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::path(hlid, hline, Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.8)))?;

        // Items
        let item_style =
            CharStyle { size: 8.5, fill: Paint::solid(Color::rgb8(0x1F, 0x29, 0x37)), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let sample_line_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.5);

        for (idx, name) in legend_items.iter().enumerate() {
            let ry = y + header_h + (idx as f64) * row_h + row_h * 0.5 + 3.0;

            // Sample line (24 pt length)
            let s_line = shapes::line(Point::new(x + 12.0, ry - 3.0), Point::new(x + 36.0, ry - 3.0));
            let slid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::path(slid, s_line, sample_line_app.clone()))?;

            // Name text
            let n_pos = Point::new(x + 44.0, ry);
            let ntid = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(ntid, NodeKind::Text(Box::new(TextObject::point(n_pos, name, item_style.clone())))))?;
        }

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "rows": legend_items.len(),
    }))
}

/// Sets sheet publication metadata attributes on document.
fn set_sheet_props(s: &mut Session, p: &Value) -> Result<Value> {
    let title = p.get("title").and_then(Value::as_str);
    let plate_no = p.get("plate_no").and_then(Value::as_str);
    let site = p.get("site").and_then(Value::as_str);
    let institute = p.get("institute").and_then(Value::as_str);
    let scale = p.get("scale").and_then(Value::as_str);
    let date = p.get("date").and_then(Value::as_str);

    s.edit("Set Sheet Properties", |d, _| {
        if let Some(first_layer) = d.layers.first() {
            let layer_id = first_layer.id;
            if let Some(node) = d.node_mut(layer_id) {
                let mut attrs = node.attrs.take().unwrap_or_default();
                if let Some(t) = title {
                    attrs.data.retain(|(key, _)| key != "sheet:title");
                    attrs.data.push(("sheet:title".into(), t.into()));
                }
                if let Some(pn) = plate_no {
                    attrs.data.retain(|(key, _)| key != "sheet:plate_no");
                    attrs.data.push(("sheet:plate_no".into(), pn.into()));
                }
                if let Some(st) = site {
                    attrs.data.retain(|(key, _)| key != "sheet:site");
                    attrs.data.push(("sheet:site".into(), st.into()));
                }
                if let Some(ins) = institute {
                    attrs.data.retain(|(key, _)| key != "sheet:institute");
                    attrs.data.push(("sheet:institute".into(), ins.into()));
                }
                if let Some(sc) = scale {
                    attrs.data.retain(|(key, _)| key != "sheet:scale");
                    attrs.data.push(("sheet:scale".into(), sc.into()));
                }
                if let Some(dt) = date {
                    attrs.data.retain(|(key, _)| key != "sheet:date");
                    attrs.data.push(("sheet:date".into(), dt.into()));
                }
                node.attrs = Some(attrs);
            }
        }
        Ok(NodeId(0))
    })?;

    Ok(json!({
        "title": title,
        "plate_no": plate_no,
        "site": site,
        "institute": institute,
        "scale": scale,
        "date": date,
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
                "width": 1190, // A3 width
                "height": 842  // A3 height
            }),
        )
        .unwrap();
        s
    }

    #[test]
    fn test_add_north_arrow() {
        let mut s = session_with_doc();
        let res = s.execute("sheet.addNorthArrow", &json!({ "x": 1050.0, "y": 100.0, "size_pt": 79.37 })).unwrap();
        assert_eq!(res["x"], 1050.0);
        assert_eq!(res["y"], 100.0);
        assert_eq!(res["size_pt"], 79.37);
    }

    #[test]
    fn test_add_scale_bar() {
        let mut s = session_with_doc();
        let res = s.execute("sheet.addScaleBar", &json!({ "x": 100.0, "y": 700.0, "scale_ratio": 40.0, "total_m": 5.0 })).unwrap();
        assert_eq!(res["x"], 100.0);
        assert_eq!(res["scale_ratio"], 40.0);
        assert!(res["length_pt"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn test_add_title_block() {
        let mut s = session_with_doc();
        let res = s
            .execute(
                "sheet.addTitleBlock",
                &json!({
                    "x": 800.0,
                    "y": 720.0,
                    "site": "경주 황오동 고분군",
                    "title": "1호 주거지 종합도판",
                    "scale": "1:40",
                    "institute": "국립문화유산연구원",
                    "plate_no": "PLATE 3"
                }),
            )
            .unwrap();
        assert_eq!(res["plate_no"], "PLATE 3");
        assert_eq!(res["width"], 320.0);
    }

    #[test]
    fn test_generate_legend_table_and_props() {
        let mut s = session_with_doc();
        let leg_res = s.execute("sheet.generateLegendTable", &json!({ "x": 100.0, "y": 500.0 })).unwrap();
        assert!(leg_res["rows"].as_u64().unwrap() >= 1);

        let props_res = s
            .execute(
                "sheet.setProps",
                &json!({
                    "plate_no": "PLATE 3",
                    "title": "1호 주거지 실측 종합도판",
                    "site": "경주 유적",
                    "institute": "연구원"
                }),
            )
            .unwrap();
        assert_eq!(props_res["plate_no"], "PLATE 3");
        assert_eq!(props_res["title"], "1호 주거지 실측 종합도판");
    }
}
