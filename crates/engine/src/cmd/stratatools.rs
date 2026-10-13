//! Archaeological Section and Soil Stratum View engine commands (Trowel, J010).
//!
//! Provides the foundational drafting commands for stratigraphic excavation sections:
//! - Stratum Boundary Line (`drawing.addStratumLine`)
//! - Stratum Badge Marker (`drawing.addStratumBadge`)
//! - Hearth / Burnt Clay Section (`drawing.addHearthCut`)
//! - Three-Line Stratum Table (`drawing.generateStrataTable`)
//! - Section Properties (`section.setProps`)
//! - Section Metrics (`section.calculateMetrics`)

use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::legend::{Legend, LegendGroup, LegendSample, LineType};
use vectorcraft_doc::{Appearance, CharStyle, Document, Node, NodeId, NodeKind, TextObject};
use vectorcraft_geom::{PathData, Point, Rect, SubPath, shapes};

use super::layer::make_layer;
use super::*;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "drawing.addStratumLine",
            "Add Stratum Boundary Line",
            [],
            None,
            "{anchors: [[x, y], ...], closed?: bool} add 0.3 pt stratigraphic boundary line to '토층선' layer (Trowel, J010) → {id, anchors, closed}",
            has_doc,
            add_stratum_line
        ),
        cmd!(
            "drawing.addStratumBadge",
            "Add Stratum Badge",
            [],
            None,
            "{x: number, y: number, number?: number, name?: string, munsell?: string, texture?: string, inclusions?: string} add circled stratum marker and layer attributes (Trowel, J010) → {id, x, y, number, name}",
            has_doc,
            add_stratum_badge
        ),
        cmd!(
            "drawing.addHearthCut",
            "Add Hearth / Burnt Clay Section",
            [],
            None,
            "{anchors: [[x, y], ...], label?: string} add clay-filled hearth/burnt soil cut cross section (Trowel, J010) → {id, label}",
            has_doc,
            add_hearth_cut
        ),
        cmd!(
            "drawing.generateStrataTable",
            "Generate Strata Explanation Table",
            [],
            None,
            "{x: number, y: number, width?: number} generate three-line table (삼선표) summarizing strata metadata (Trowel, J010) → {id, x, y, rows, width}",
            has_doc,
            generate_strata_table
        ),
        cmd!(
            "section.setProps",
            "Set Section Properties",
            [],
            None,
            "{name?: string, datum_el?: number, datum_y?: number} set section identifier, reference datum elevation, and datum line Y (Trowel, J010) → {name, datum_el, datum_y}",
            has_doc,
            set_section_props
        ),
        cmd!(
            "section.calculateMetrics",
            "Calculate Section Metrics",
            [],
            None,
            "{datum_y?: number} calculate section horizontal width (m) and maximum excavation depth below datum (m) (Trowel, J010) → {width_m, max_depth_m, width_pt, max_depth_pt}",
            has_doc,
            calculate_section_metrics
        ),
    ]
}

pub(crate) fn get_or_create_section_layer(
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
            group: Some(LegendGroup::Section),
            strata: strata.map(str::to_string),
            favourite: weight.is_some_and(|w| (w - 0.2).abs() < 1e-6 || (w - 0.3).abs() < 1e-6 || (w - 0.5).abs() < 1e-6),
            ..Legend::default()
        }));
    }
    let at = d.layers.len();
    let id = d.insert(None, at, n)?;
    Ok(id)
}

fn next_stratum_number(d: &Document) -> u32 {
    let mut max_num = 0;
    d.walk(|n| {
        if !n.is_layer()
            && let Some(attrs) = &n.attrs
        {
            for (k, v) in &attrs.data {
                if k == "stratum:number"
                    && let Ok(val) = v.parse::<u32>()
                    && val > max_num
                {
                    max_num = val;
                }
            }
        }
    });
    max_num + 1
}

fn stratum_circle_symbol(n: u32) -> String {
    if (1..=20).contains(&n) {
        let ch = char::from_u32(0x2460 + (n - 1)).unwrap_or('①');
        ch.to_string()
    } else {
        n.to_string()
    }
}

fn add_stratum_line(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.addStratumLine";

    let arr = p.get("anchors").or_else(|| p.get("points")).and_then(Value::as_array).ok_or_else(|| bad(C, "anchors or points array is required"))?;
    if arr.len() < 2 {
        return Err(bad(C, "at least 2 anchor points are required"));
    }

    let closed = p.get("closed").and_then(Value::as_bool).unwrap_or(false);

    let mut points = Vec::with_capacity(arr.len());
    for item in arr {
        let (x, y) = if let Some(xy) = item.as_array() {
            if xy.len() < 2 {
                return Err(bad(C, "invalid anchor coordinates"));
            }
            (xy.first().and_then(Value::as_f64).unwrap_or(0.0), xy.get(1).and_then(Value::as_f64).unwrap_or(0.0))
        } else if let Some(obj) = item.as_object() {
            (obj.get("x").and_then(Value::as_f64).unwrap_or(0.0), obj.get("y").and_then(Value::as_f64).unwrap_or(0.0))
        } else {
            return Err(bad(C, "invalid anchor format"));
        };

        if !x.is_finite() || !y.is_finite() {
            return Err(bad(C, "coordinates must be finite"));
        }
        points.push(Point::new(x, y));
    }

    let path_id = s.edit("Add Stratum Line", |d, _| {
        let layer_id = get_or_create_section_layer(d, "토층선", Some(0.3), LineType::Solid, LegendSample::Line, Some("stratum_boundary"))?;

        let pd = PathData::new(vec![SubPath::polyline(&points, closed)]);
        let app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.3);
        let mut node = Node::path(d.alloc_id(), pd, app);
        node.name = Some("토층선 (0.3 pt)".into());
        let id = d.insert(Some(layer_id), usize::MAX, node)?;
        Ok(id)
    })?;

    Ok(json!({
        "id": path_id.0,
        "points": points.len(),
        "closed": closed,
    }))
}

fn add_stratum_badge(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.addStratumBadge";

    let x = f64_req(p, "x", C)?;
    let y = f64_req(p, "y", C)?;

    if !x.is_finite() || !y.is_finite() {
        return Err(bad(C, "coordinates must be finite"));
    }

    let doc_ref = &s.doc()?.doc;
    let number = p.get("number").and_then(Value::as_u64).map(|n| n as u32).unwrap_or_else(|| next_stratum_number(doc_ref));

    let name = p.get("name").and_then(Value::as_str).unwrap_or(match number {
        1 => "표토",
        2 => "암갈색 사질점토",
        3 => "주거지 바닥 다짐토",
        4 => "생토층",
        _ => "퇴적토",
    });

    let munsell = p.get("munsell").and_then(Value::as_str).unwrap_or(match number {
        1 => "10YR 4/3 갈색",
        2 => "10YR 3/2 암갈색",
        3 => "10YR 2/2 흑갈색",
        4 => "7.5YR 5/6 명황갈색",
        _ => "10YR 4/4 황갈색",
    });

    let texture = p.get("texture").and_then(Value::as_str).unwrap_or(match number {
        1 => "사질점토",
        2 => "점토질모래",
        3 => "점토",
        4 => "점토 (풍화암반)",
        _ => "사질점토",
    });

    let inclusions = p.get("inclusions").and_then(Value::as_str).unwrap_or(match number {
        1 => "표토 및 현대 식생 부식토, 소량의 자기편",
        2 => "암적갈색 소토립 다량 함유, 무문토기 구연부편",
        3 => "주거지 바닥 다짐토, 목탄편 집중, 갈판 완형 출토",
        4 => "기반 풍화암반 잔류 점토층 (무유구 기반층)",
        _ => "목탄립 및 소토립 미량 함유",
    });

    let group_id = s.edit("Add Stratum Badge", |d, _| {
        let layer_id =
            get_or_create_section_layer(d, "토층 번호", Some(0.2), LineType::Solid, LegendSample::Mark("①".into()), Some("stratum_badge"))?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        let sym = stratum_circle_symbol(number);
        group.name = Some(format!("토층 {sym} {name}"));

        // Store structured metadata attributes
        let mut attrs = group.attrs.take().unwrap_or_default();
        attrs.data.push(("stratum:number".into(), number.to_string()));
        attrs.data.push(("stratum:name".into(), name.into()));
        attrs.data.push(("stratum:munsell".into(), munsell.into()));
        attrs.data.push(("stratum:texture".into(), texture.into()));
        attrs.data.push(("stratum:inclusions".into(), inclusions.into()));
        group.attrs = Some(attrs);

        d.insert(Some(layer_id), usize::MAX, group)?;

        // 1. Circle badge: radius 9.0 pt, white fill, 1.0 pt black stroke
        let circle_pd = shapes::ellipse(Rect::new(x - 9.0, y - 9.0, x + 9.0, y + 9.0));
        let circle_app = Appearance::basic(Paint::solid(Color::WHITE), Paint::solid(Color::BLACK), 1.0);
        let circle_node = Node::path(d.alloc_id(), circle_pd, circle_app);
        d.insert(Some(gid), usize::MAX, circle_node)?;

        // 2. Centered circled number / digit (9 pt bold)
        let style = CharStyle { size: 9.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let text_offset_x = if number <= 20 {
            4.0
        } else if number < 100 {
            6.0
        } else {
            8.0
        };
        let sym_text = stratum_circle_symbol(number);
        let text_pos = Point::new(x - text_offset_x, y + 3.5);
        let text_obj = TextObject::point(text_pos, &sym_text, style);
        let text_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text_obj)));
        d.insert(Some(gid), usize::MAX, text_node)?;

        // 3. Label text next to circle (10 pt regular)
        let label_style = CharStyle { size: 10.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let label_pos = Point::new(x + 14.0, y + 3.5);
        let label_obj = TextObject::point(label_pos, name, label_style);
        let label_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(label_obj)));
        d.insert(Some(gid), usize::MAX, label_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "number": number,
        "name": name,
        "munsell": munsell,
        "texture": texture,
        "inclusions": inclusions,
    }))
}

fn add_hearth_cut(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.addHearthCut";

    let arr = p.get("anchors").or_else(|| p.get("points")).and_then(Value::as_array).ok_or_else(|| bad(C, "anchors or points array is required"))?;
    if arr.len() < 2 {
        return Err(bad(C, "at least 2 anchor points are required"));
    }

    let label = p.get("label").and_then(Value::as_str).unwrap_or("소토면");

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
            return Err(bad(C, "invalid coordinate format"));
        };
        if !x.is_finite() || !y.is_finite() {
            return Err(bad(C, "coordinates must be finite"));
        }
        points.push(Point::new(x, y));
    }

    let group_id = s.edit("Add Hearth Cut", |d, _| {
        let layer_id = get_or_create_section_layer(d, "내부 시설", Some(0.5), LineType::Solid, LegendSample::Line, Some("hearth_feature"))?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some("소토면 (Hearth Pit Section)".into());
        d.insert(Some(layer_id), usize::MAX, group)?;

        let pd = PathData::new(vec![SubPath::polyline(&points, true)]);

        // Clay fill (#F3DEDA) + clay stroke (#C46686 / 1.2 pt)
        let fill_color = Color::rgb8(0xF3, 0xDE, 0xDA);
        let stroke_color = Color::rgb8(0xC4, 0x66, 0x86);
        let app = Appearance::basic(Paint::solid(fill_color), Paint::solid(stroke_color), 1.2);
        let path_node = Node::path(d.alloc_id(), pd, app);
        d.insert(Some(gid), usize::MAX, path_node)?;

        // Centered text "소토면"
        let center_x = points.iter().map(|p| p.x).sum::<f64>() / (points.len() as f64);
        let center_y = points.iter().map(|p| p.y).sum::<f64>() / (points.len() as f64);
        let text_style = CharStyle { size: 9.0, fill: Paint::solid(stroke_color), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let text_pos = Point::new(center_x - 12.0, center_y + 3.0);
        let text_obj = TextObject::point(text_pos, label, text_style);
        let text_node = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text_obj)));
        d.insert(Some(gid), usize::MAX, text_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "label": label,
        "points": points.len(),
    }))
}

struct StratumRow {
    number: u32,
    symbol: String,
    munsell: String,
    texture: String,
    inclusions: String,
}

fn generate_strata_table(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.generateStrataTable";

    let (default_x, default_y) = {
        let doc = &s.doc()?.doc;
        let mut max_y = 350.0;
        doc.walk(|n| {
            if !n.is_layer()
                && let NodeKind::Path { ref path, .. } = n.kind
                && let Some(bb) = path.bounds()
                && bb.max_y().is_finite()
                && bb.max_y() > max_y
            {
                max_y = bb.max_y();
            }
        });
        (50.0, max_y + 30.0)
    };

    let x = p.get("x").and_then(Value::as_f64).unwrap_or(default_x);
    let y = p.get("y").and_then(Value::as_f64).unwrap_or(default_y);
    let width = p.get("width").and_then(Value::as_f64).unwrap_or(640.0);

    if !x.is_finite() || !y.is_finite() || !width.is_finite() || width <= 10.0 {
        return Err(bad(C, "invalid dimensions"));
    }

    let doc_ref = &s.doc()?.doc;
    let mut collected: Vec<StratumRow> = Vec::new();

    doc_ref.walk(|n| {
        if !n.is_layer()
            && let Some(attrs) = &n.attrs
        {
            let mut num = None;
            let mut munsell = None;
            let mut texture = None;
            let mut inclusions = None;

            for (k, v) in &attrs.data {
                match k.as_str() {
                    "stratum:number" => num = v.parse::<u32>().ok(),
                    "stratum:munsell" => munsell = Some(v.clone()),
                    "stratum:texture" => texture = Some(v.clone()),
                    "stratum:inclusions" => inclusions = Some(v.clone()),
                    _ => {}
                }
            }

            if let Some(n) = num {
                collected.push(StratumRow {
                    number: n,
                    symbol: stratum_circle_symbol(n),
                    munsell: munsell.unwrap_or_else(|| "10YR 4/3 갈색".into()),
                    texture: texture.unwrap_or_else(|| "사질점토".into()),
                    inclusions: inclusions.unwrap_or_else(|| "퇴적토".into()),
                });
            }
        }
    });

    collected.sort_by_key(|r| r.number);
    collected.dedup_by_key(|r| r.number);

    // If document has no stratum badges yet, provide standard four archaeological layers:
    if collected.is_empty() {
        collected = vec![
            StratumRow {
                number: 1,
                symbol: "①".into(),
                munsell: "10YR 4/3 갈색".into(),
                texture: "사질점토".into(),
                inclusions: "표토 및 현대 식생 부식토, 소량의 자기편".into(),
            },
            StratumRow {
                number: 2,
                symbol: "②".into(),
                munsell: "10YR 3/2 암갈색".into(),
                texture: "점토질모래".into(),
                inclusions: "암적갈색 소토립 다량 함유, 무문토기 구연부편".into(),
            },
            StratumRow {
                number: 3,
                symbol: "③".into(),
                munsell: "10YR 2/2 흑갈색".into(),
                texture: "점토".into(),
                inclusions: "주거지 바닥 다짐토, 목탄편 집중, 갈판 완형 출토".into(),
            },
            StratumRow {
                number: 4,
                symbol: "④".into(),
                munsell: "7.5YR 5/6 명황갈색".into(),
                texture: "점토".into(),
                inclusions: "기반 풍화암반 잔류 점토층 (무유구 기반층)".into(),
            },
        ];
    }

    let row_count = collected.len();
    let header_h = 24.0;
    let row_h = 24.0;
    let total_h = header_h + (row_count as f64) * row_h;

    let group_id = s.edit("Generate Strata Table", |d, _| {
        let layer_id = get_or_create_section_layer(d, "주기 및 주기표", Some(0.2), LineType::Solid, LegendSample::Line, Some("strata_table"))?;

        let gid = d.alloc_id();
        let mut group = Node::new(gid, NodeKind::Group { children: vec![], clip: false });
        group.name = Some("토층 해설표 (삼선표)".into());
        d.insert(Some(layer_id), usize::MAX, group)?;

        // Background box: #FAF9F5, light border 1.0 pt
        let bg_rect = Rect::new(x, y, x + width, y + total_h);
        let bg_pd = shapes::rectangle(bg_rect);
        let bg_app = Appearance::basic(Paint::solid(Color::rgb8(0xFA, 0xF9, 0xF5)), Paint::solid(Color::rgb8(0xD8, 0xD6, 0xCD)), 1.0);
        let bg_node = Node::path(d.alloc_id(), bg_pd, bg_app);
        d.insert(Some(gid), usize::MAX, bg_node)?;

        // Line 1: Top border (1.5 pt black line)
        let top_line_pd = shapes::line(Point::new(x, y), Point::new(x + width, y));
        let top_line_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 1.5);
        let top_node = Node::path(d.alloc_id(), top_line_pd, top_line_app);
        d.insert(Some(gid), usize::MAX, top_node)?;

        // Line 2: Header separator (1.0 pt line)
        let sep_y = y + header_h;
        let sep_line_pd = shapes::line(Point::new(x, sep_y), Point::new(x + width, sep_y));
        let sep_line_app = Appearance::basic(Paint::None, Paint::solid(Color::rgb8(0x87, 0x86, 0x7F)), 1.0);
        let sep_node = Node::path(d.alloc_id(), sep_line_pd, sep_line_app);
        d.insert(Some(gid), usize::MAX, sep_node)?;

        // Header column texts (11 pt bold)
        let header_style = CharStyle { size: 11.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let h_y = y + 16.0;

        let col_layer = TextObject::point(Point::new(x + 12.0, h_y), "층위", header_style.clone());
        let nid1 = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(nid1, NodeKind::Text(Box::new(col_layer))))?;

        let col_color = TextObject::point(Point::new(x + 60.0, h_y), "토색 (Munsell)", header_style.clone());
        let nid2 = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(nid2, NodeKind::Text(Box::new(col_color))))?;

        let col_tex = TextObject::point(Point::new(x + 180.0, h_y), "토성", header_style.clone());
        let nid3 = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(nid3, NodeKind::Text(Box::new(col_tex))))?;

        let col_inc = TextObject::point(Point::new(x + 280.0, h_y), "포함유물 및 퇴적 특징", header_style);
        let nid4 = d.alloc_id();
        d.insert(Some(gid), usize::MAX, Node::new(nid4, NodeKind::Text(Box::new(col_inc))))?;

        // Rows
        let row_style = CharStyle { size: 10.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };
        let mono_style = CharStyle { size: 9.0, fill: Paint::solid(Color::BLACK), font_family: "Noto Sans KR".into(), ..CharStyle::default() };

        for (i, row) in collected.iter().enumerate() {
            let row_y = sep_y + (i as f64) * row_h + 16.0;

            // Stratum symbol
            let sym_obj = TextObject::point(Point::new(x + 12.0, row_y), &row.symbol, mono_style.clone());
            let rid1 = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(rid1, NodeKind::Text(Box::new(sym_obj))))?;

            // Munsell color
            let color_obj = TextObject::point(Point::new(x + 60.0, row_y), &row.munsell, row_style.clone());
            let rid2 = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(rid2, NodeKind::Text(Box::new(color_obj))))?;

            // Texture
            let tex_obj = TextObject::point(Point::new(x + 180.0, row_y), &row.texture, row_style.clone());
            let rid3 = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(rid3, NodeKind::Text(Box::new(tex_obj))))?;

            // Inclusions
            let inc_obj = TextObject::point(Point::new(x + 280.0, row_y), &row.inclusions, row_style.clone());
            let rid4 = d.alloc_id();
            d.insert(Some(gid), usize::MAX, Node::new(rid4, NodeKind::Text(Box::new(inc_obj))))?;
        }

        // Line 3: Bottom border (1.5 pt black line)
        let bot_y = y + total_h;
        let bot_line_pd = shapes::line(Point::new(x, bot_y), Point::new(x + width, bot_y));
        let bot_line_app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 1.5);
        let bid = d.alloc_id();
        let bot_node = Node::path(bid, bot_line_pd, bot_line_app);
        d.insert(Some(gid), usize::MAX, bot_node)?;

        Ok(gid)
    })?;

    Ok(json!({
        "id": group_id.0,
        "x": x,
        "y": y,
        "width": width,
        "height": total_h,
        "rows": row_count,
    }))
}

fn set_section_props(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p.get("name").and_then(Value::as_str);
    let datum_el = p.get("datum_el").or_else(|| p.get("datum_m")).and_then(Value::as_f64);
    let datum_y = p.get("datum_y").and_then(Value::as_f64);

    s.edit("Set Section Properties", |d, _| {
        // Tag root or first layer with section properties
        if let Some(first_layer) = d.layers.first() {
            let layer_id = first_layer.id;
            if let Some(node) = d.node_mut(layer_id) {
                let mut attrs = node.attrs.take().unwrap_or_default();
                if let Some(n) = name {
                    attrs.data.retain(|(k, _)| k != "section:name");
                    attrs.data.push(("section:name".into(), n.into()));
                }
                if let Some(el) = datum_el {
                    attrs.data.retain(|(k, _)| k != "section:datum_el");
                    attrs.data.push(("section:datum_el".into(), el.to_string()));
                }
                if let Some(dy) = datum_y {
                    attrs.data.retain(|(k, _)| k != "section:datum_y");
                    attrs.data.push(("section:datum_y".into(), dy.to_string()));
                }
                node.attrs = Some(attrs);
            }
        }
        Ok(NodeId(0))
    })?;

    Ok(json!({
        "name": name,
        "datum_el": datum_el,
        "datum_y": datum_y,
    }))
}

fn calculate_section_metrics(s: &mut Session, p: &Value) -> Result<Value> {
    let doc = &s.doc()?.doc;
    let ppm = doc.coord.as_ref().map(|c| c.points_per_meter()).unwrap_or(141.73228);

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    doc.walk(|n| {
        if !n.is_layer()
            && let NodeKind::Path { ref path, .. } = n.kind
            && let Some(bb) = path.bounds()
        {
            if bb.min_x() < min_x {
                min_x = bb.min_x();
            }
            if bb.max_x() > max_x {
                max_x = bb.max_x();
            }
            if bb.min_y() < min_y {
                min_y = bb.min_y();
            }
            if bb.max_y() > max_y {
                max_y = bb.max_y();
            }
        }
    });

    let stored_datum_y = doc
        .layers
        .first()
        .and_then(|l| doc.node(l.id))
        .and_then(|n| n.attrs.as_ref())
        .and_then(|a| a.data.iter().find(|(k, _)| k == "section:datum_y").and_then(|(_, v)| v.parse::<f64>().ok()));

    let datum_y = p.get("datum_y").and_then(Value::as_f64).or(stored_datum_y).unwrap_or_else(|| if min_y.is_finite() { min_y } else { 100.0 });

    let width_pt = if min_x.is_finite() && max_x.is_finite() && max_x > min_x { max_x - min_x } else { 0.0 };
    let depth_pt = if max_y.is_finite() && max_y > datum_y { max_y - datum_y } else { 0.0 };

    let width_m = width_pt / ppm;
    let max_depth_m = depth_pt / ppm;

    Ok(json!({
        "width_m": width_m,
        "max_depth_m": max_depth_m,
        "width_pt": width_pt,
        "max_depth_pt": depth_pt,
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
                "width": 1000,
                "height": 700
            }),
        )
        .unwrap();
        s
    }

    #[test]
    fn test_add_stratum_line() {
        let mut s = session_with_doc();
        let res = s
            .execute(
                "drawing.addStratumLine",
                &json!({
                    "anchors": [
                        [100.0, 150.0],
                        [300.0, 160.0],
                        [500.0, 155.0]
                    ],
                    "closed": false
                }),
            )
            .unwrap();

        assert_eq!(res["points"], 3);
        assert_eq!(res["closed"], false);

        let doc = &s.doc().unwrap().doc;
        let layer = doc.layers.iter().find(|l| l.display_name() == "토층선").unwrap();
        assert_eq!(layer.legend().as_ref().unwrap().weight, Some(0.3));
    }

    #[test]
    fn test_add_stratum_badge_and_table() {
        let mut s = session_with_doc();

        // Add badge ①
        let b1 = s
            .execute(
                "drawing.addStratumBadge",
                &json!({
                    "x": 200.0,
                    "y": 120.0,
                    "number": 1,
                    "name": "표토",
                    "munsell": "10YR 4/3 갈색",
                    "texture": "사질점토",
                    "inclusions": "식생 부식토"
                }),
            )
            .unwrap();
        assert_eq!(b1["number"], 1);
        assert_eq!(b1["name"], "표토");

        // Add badge ②
        let b2 = s
            .execute(
                "drawing.addStratumBadge",
                &json!({
                    "x": 300.0,
                    "y": 160.0,
                    "number": 2,
                    "name": "퇴적토",
                    "munsell": "10YR 3/2 암갈색",
                    "texture": "점토질모래",
                    "inclusions": "소토립 다량"
                }),
            )
            .unwrap();
        assert_eq!(b2["number"], 2);

        // Generate table
        let tbl = s
            .execute(
                "drawing.generateStrataTable",
                &json!({
                    "x": 70.0,
                    "y": 380.0,
                    "width": 640.0
                }),
            )
            .unwrap();
        assert_eq!(tbl["rows"], 2);
        assert_eq!(tbl["width"], 640.0);
    }

    #[test]
    fn test_add_hearth_cut() {
        let mut s = session_with_doc();
        let res = s
            .execute(
                "drawing.addHearthCut",
                &json!({
                    "anchors": [
                        [370.0, 240.0],
                        [410.0, 258.0],
                        [450.0, 238.0]
                    ],
                    "label": "소토면"
                }),
            )
            .unwrap();
        assert_eq!(res["label"], "소토면");
        assert_eq!(res["points"], 3);
    }

    #[test]
    fn test_section_props_and_metrics_and_rollback() {
        let mut s = session_with_doc();
        s.execute("section.setProps", &json!({"name": "A-A' 남벽", "datum_el": 150.0, "datum_y": 100.0})).unwrap();

        // Draw a stratum boundary line
        s.execute(
            "drawing.addStratumLine",
            &json!({
                "anchors": [[100.0, 100.0], [200.0, 150.0], [300.0, 200.0]]
            }),
        )
        .unwrap();

        let m = s.execute("section.calculateMetrics", &json!({})).unwrap();
        assert!((m["width_pt"].as_f64().unwrap() - 200.0).abs() < 1e-4);
        assert!((m["max_depth_pt"].as_f64().unwrap() - 100.0).abs() < 1e-4);

        // Test transaction rollback
        s.execute("edit.undo", &json!({})).unwrap();
        let doc_after_undo = &s.doc().unwrap().doc;
        let count =
            doc_after_undo.layers.iter().find(|l| l.display_name() == "토층선").map(|l| l.children().map(|c| c.len()).unwrap_or(0)).unwrap_or(0);
        assert_eq!(count, 0, "undo rolls back created stratum line");

        s.execute("edit.redo", &json!({})).unwrap();
        let doc_after_redo = &s.doc().unwrap().doc;
        let count_redo =
            doc_after_redo.layers.iter().find(|l| l.display_name() == "토층선").map(|l| l.children().map(|c| c.len()).unwrap_or(0)).unwrap_or(0);
        assert_eq!(count_redo, 1, "redo restores stratum line");
    }
}
