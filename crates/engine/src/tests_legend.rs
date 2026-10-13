//! Trowel legend rows (J005): a top-level layer's legend row is the default look of its lines.
//! Changing a row restyles the lines that follow it while lines set apart ("따로") stay; picking a
//! row sets the pen (and with `move`, brings the selection); Revert to the Legend, Change Weights at
//! Once, the three legend groups and Add to Legend work through `legend.*`, one undo step each.

use serde_json::{Value, json};
use vectorcraft_doc::{Legend, LegendGroup, LegendSample, LineType, Node, NodeId, legend};

use super::*;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 400, "height": 400})).unwrap();
    s
}

fn id(v: &Value) -> NodeId {
    NodeId(v["id"].as_u64().unwrap())
}

fn run(s: &mut Session, cmd: &str, p: Value) -> Value {
    s.execute(cmd, &p).unwrap_or_else(|e| panic!("{cmd} {p}: {e}"))
}

/// A new top-level layer, current.
fn layer(s: &mut Session, name: &str) -> NodeId {
    id(&run(s, "layer.new", json!({"name": name, "top": true})))
}

/// A rectangle (a closed, stroked path) in the current layer.
fn rect(s: &mut Session, x: f64) -> NodeId {
    id(&run(s, "shape.rectangle", json!({"x": x, "y": 0, "width": 20, "height": 20})))
}

/// An open two-point path in the current layer.
fn open_line(s: &mut Session, x: f64) -> NodeId {
    id(&run(s, "path.create", json!({"anchors": [{"x": x, "y": 50}, {"x": x + 20.0, "y": 50}], "closed": false})))
}

fn set_weight(s: &mut Session, ids: &[NodeId], w: f64) {
    run(s, "stroke.set", json!({"weight": w, "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>()}));
}

fn set_dash(s: &mut Session, ids: &[NodeId], t: &LineType) {
    run(s, "stroke.set", json!({"dash": t.dash_pt(), "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>()}));
}

fn node(s: &Session, id: NodeId) -> Node {
    s.doc().unwrap().doc.node(id).unwrap_or_else(|| panic!("no node {id:?}")).clone()
}

fn weight(s: &Session, id: NodeId) -> f64 {
    node(s, id).appearance.stroke().unwrap().width
}

fn line_type(s: &Session, id: NodeId) -> LineType {
    LineType::of(node(s, id).appearance.stroke().unwrap())
}

fn legend_of(s: &Session, layer: NodeId) -> Option<Legend> {
    node(s, layer).legend().cloned()
}

fn parent(s: &Session, id: NodeId) -> Option<NodeId> {
    s.doc().unwrap().doc.parent_of(id)
}

fn depth(s: &Session) -> usize {
    s.doc().unwrap().history.undo.len()
}

fn doc_json(s: &Session) -> Value {
    serde_json::to_value(&*s.doc().unwrap().doc).unwrap()
}

fn current(s: &Session) -> Option<NodeId> {
    s.doc().unwrap().current_layer()
}

fn overrides(s: &Session, layer: NodeId) -> Vec<NodeId> {
    let n = node(s, layer);
    let Some(row) = n.legend() else { return vec![] };
    legend::lines(&n).into_iter().filter(|l| legend::stroke_of(l).is_some_and(|st| row.differs(st))).map(|l| l.id).collect()
}

// ---------- legend.set ----------

#[test]
fn setting_a_row_restyles_its_matching_lines_and_keeps_overrides() {
    let mut s = session();
    let row = layer(&mut s, "굴광선");
    let lines: Vec<NodeId> = (0..4).map(|i| rect(&mut s, f64::from(i) * 30.0)).collect();
    set_weight(&mut s, &lines, 0.8);
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.8}));
    set_weight(&mut s, &lines[3..], 0.5);
    assert_eq!(overrides(&s, row), vec![lines[3]]);
    let before = depth(&s);
    let r = run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.6}));
    assert_eq!(r["lines"], json!(3));
    for l in &lines[..3] {
        assert_eq!(weight(&s, *l), 0.6);
    }
    assert_eq!(weight(&s, lines[3]), 0.5, "a line set apart keeps its weight");
    assert_eq!(legend_of(&s, row).unwrap().weight, Some(0.6));
    assert_eq!(depth(&s), before + 1, "one undo step");
    run(&mut s, "edit.undo", json!({}));
    for l in &lines[..3] {
        assert_eq!(weight(&s, *l), 0.8);
    }
    assert_eq!(weight(&s, lines[3]), 0.5);
    assert_eq!(legend_of(&s, row).unwrap().weight, Some(0.8));
}

#[test]
fn a_new_look_leaves_existing_lines_alone() {
    let mut s = session();
    let row = layer(&mut s, "바닥선");
    let a = rect(&mut s, 0.0);
    let b = rect(&mut s, 30.0);
    set_weight(&mut s, &[b], 0.5);
    let r = run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.5}));
    assert_eq!(r["lines"], json!(0));
    assert_eq!(weight(&s, a), 1.0, "giving a layer its first look changes no line");
    assert_eq!(weight(&s, b), 0.5);
    assert_eq!(overrides(&s, row), vec![a]);
}

#[test]
fn line_type_follows_the_row() {
    let mut s = session();
    let row = layer(&mut s, "단면 위치선");
    let a = rect(&mut s, 0.0);
    let b = open_line(&mut s, 30.0);
    set_weight(&mut s, &[a, b], 0.5);
    run(&mut s, "stroke.set", json!({"cap": "round", "ids": [a.0]}));
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.5}));
    let r = run(&mut s, "legend.set", json!({"id": row.0, "line": "dashed"}));
    assert_eq!(r["lines"], json!(2));
    let st = node(&s, a).appearance.stroke().unwrap().clone();
    assert_eq!(st.dash.as_ref().map(|d| d.pattern.clone()), LineType::Dashed.dash_pt());
    assert_eq!(st.cap, vectorcraft_doc::LineCap::Round, "the cap is not the look");
    assert_eq!(st.paint, vectorcraft_color::Paint::solid(vectorcraft_color::Color::BLACK));
    assert_eq!(line_type(&s, b), LineType::Dashed);
    run(&mut s, "legend.set", json!({"id": row.0, "line": "dashdot"}));
    assert_eq!(line_type(&s, a), LineType::DashDot);
    run(&mut s, "legend.set", json!({"id": row.0, "line": [4, 2]}));
    assert_eq!(line_type(&s, b), LineType::Custom(vec![4.0, 2.0]));
    assert_eq!(legend_of(&s, row).unwrap().line, LineType::Custom(vec![4.0, 2.0]));
    run(&mut s, "legend.set", json!({"id": row.0, "line": "solid"}));
    assert_eq!(node(&s, a).appearance.stroke().unwrap().dash, None);
    assert_eq!(line_type(&s, b), LineType::Solid);
}

#[test]
fn legend_set_writes_the_row_fields() {
    let mut s = session();
    let row = layer(&mut s, "기준점");
    run(&mut s, "legend.set", json!({"id": row.0, "favourite": true, "sample": {"mark": "△"}, "strata": "control_points", "group": "plan"}));
    let l = legend_of(&s, row).unwrap();
    assert!(l.favourite);
    assert_eq!(l.sample, LegendSample::Mark("△".into()));
    assert_eq!(l.strata.as_deref(), Some("control_points"));
    assert_eq!(l.group, Some(LegendGroup::Plan));
    assert_eq!(l.weight, None);
    run(&mut s, "legend.set", json!({"id": row.0, "strata": null, "group": null, "sample": "area"}));
    let l = legend_of(&s, row).unwrap();
    assert_eq!((l.strata, l.group, l.sample), (None, None, LegendSample::Area));
    run(&mut s, "legend.set", json!({"weight": 5000}));
    assert_eq!(legend_of(&s, row).unwrap().weight, Some(1000.0), "the current layer, clamped as stroke.set");
    run(&mut s, "legend.set", json!({"id": row.0, "weight": -1}));
    assert_eq!(legend_of(&s, row).unwrap().weight, Some(0.0));
    run(&mut s, "legend.set", json!({"id": row.0, "clear": true}));
    assert_eq!(legend_of(&s, row), None);
}

#[test]
fn locked_rows_refuse_and_locked_lines_are_skipped() {
    let mut s = session();
    let row = layer(&mut s, "돌");
    let a = rect(&mut s, 0.0);
    let b = rect(&mut s, 30.0);
    set_weight(&mut s, &[a, b], 0.8);
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.8}));
    run(&mut s, "layer.setProps", json!({"ids": [b.0], "locked": true}));
    let r = run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.6}));
    assert_eq!(r["lines"], json!(1));
    assert_eq!((weight(&s, a), weight(&s, b)), (0.6, 0.8), "a locked line keeps its weight");
    run(&mut s, "layer.setProps", json!({"ids": [row.0], "locked": true}));
    let before = (doc_json(&s), depth(&s));
    assert!(s.execute("legend.set", &json!({"id": row.0, "weight": 0.4})).is_err());
    assert_eq!((doc_json(&s), depth(&s)), before);
}

#[test]
fn legend_set_checks_its_params() {
    let mut s = session();
    let row = layer(&mut s, "row");
    let a = rect(&mut s, 0.0);
    let sub = id(&run(&mut s, "layer.newSublayer", json!({"parent": row.0})));
    let before = (doc_json(&s), depth(&s));
    for p in [
        json!({}),
        json!({"id": row.0}),
        json!({"id": row.0, "weight": "thin"}),
        json!({"id": row.0, "weight": "NaN"}),
        json!({"id": row.0, "line": "wavy"}),
        json!({"id": row.0, "line": [-1, 2]}),
        json!({"id": row.0, "line": [0, 0]}),
        json!({"id": row.0, "line": []}),
        json!({"id": row.0, "line": ["a"]}),
        json!({"id": row.0, "favourite": "yes"}),
        json!({"id": row.0, "sample": 5}),
        json!({"id": row.0, "sample": {"mark": 3}}),
        json!({"id": row.0, "group": "maps"}),
        json!({"id": row.0, "strata": 7}),
        json!({"id": sub.0, "weight": 1}),
        json!({"id": a.0, "weight": 1}),
        json!({"id": 999_999, "weight": 1}),
        json!({"id": "x", "weight": 1}),
    ] {
        assert!(s.execute("legend.set", &p).is_err(), "{p} should be refused");
        assert_eq!((doc_json(&s), depth(&s)), before, "{p} changed the document");
    }
}

#[test]
fn inspect_reports_the_legend() {
    let mut s = session();
    let row = layer(&mut s, "굴광선 (유구 면)");
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.8, "favourite": true, "strata": "feature_poly"}));
    let v = crate::inspect::node_summary(&node(&s, row));
    assert_eq!(v["legend"], json!({"weight": 0.8, "favourite": true, "strata": "feature_poly"}));
    let plain = layer(&mut s, "plain");
    assert_eq!(crate::inspect::node_summary(&node(&s, plain))["legend"], Value::Null);
}

/// A layer moved into another keeps its legend, but a sublayer is no row: the summaries leave it
/// out (code review 2).
#[test]
fn inspect_leaves_out_a_sublayers_legend() {
    let mut s = session();
    let row = layer(&mut s, "바닥선");
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.5}));
    let top = layer(&mut s, "위");
    run(&mut s, "node.move", json!({"id": row.0, "parent": top.0, "index": 0}));
    assert_eq!(parent(&s, row), Some(top));
    assert!(legend_of(&s, row).is_some(), "the layer keeps it");
    let doc = crate::inspect::document(&s);
    let sub = &doc["layers"][0]["children"][0];
    assert_eq!((&sub["id"], &sub["legend"]), (&json!(row.0), &Value::Null), "{sub}");
    let one = run(&mut s, "document.node", json!({"id": row.0, "summary": true}));
    assert_eq!(one["legend"], Value::Null, "{one}");
}

// ---------- legend.pick ----------

#[test]
fn picking_a_row_sets_the_current_layer_and_the_pen() {
    let mut s = session();
    let row = layer(&mut s, "단면 위치선");
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.2, "line": "dashdot"}));
    let plain = layer(&mut s, "plain");
    let marks = layer(&mut s, "기준점");
    run(&mut s, "legend.set", json!({"id": marks.0, "favourite": true}));
    run(&mut s, "select.none", json!({}));
    let before = (doc_json(&s), depth(&s));
    run(&mut s, "legend.pick", json!({"id": row.0}));
    assert_eq!(current(&s), Some(row));
    let pen = s.new_art_stroke();
    assert_eq!(pen.width, 0.2);
    assert_eq!(pen.dash.as_ref().map(|d| d.pattern.clone()), LineType::DashDot.dash_pt());
    assert_eq!(pen.paint, vectorcraft_color::Paint::solid(vectorcraft_color::Color::BLACK), "the colour is not the look");
    assert_eq!((doc_json(&s), depth(&s)), before, "picking is not an undo step");
    run(&mut s, "legend.pick", json!({"id": plain.0}));
    assert_eq!(current(&s), Some(plain));
    assert_eq!(s.new_art_stroke().width, 0.2, "a layer without a look leaves the pen");
    run(&mut s, "legend.pick", json!({"id": marks.0}));
    assert_eq!(current(&s), Some(marks));
    assert_eq!(LineType::of(&s.new_art_stroke()), LineType::DashDot);
    assert!(s.execute("legend.pick", &json!({})).is_err(), "pick needs an id");
}

#[test]
fn picking_with_move_brings_the_selection_and_its_look() {
    let mut s = session();
    let row = layer(&mut s, "바닥선");
    let inside = rect(&mut s, 0.0);
    set_weight(&mut s, &[inside], 0.8);
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.8}));
    let other = layer(&mut s, "Layer 2");
    let a = rect(&mut s, 30.0);
    let b = open_line(&mut s, 60.0);
    // stroke.set's weight also sets up the next object drawn, so a and b were drawn at 0.8.
    set_weight(&mut s, &[a, b], 1.0);
    run(&mut s, "select.set", json!({"ids": [a.0, b.0, inside.0]}));
    let before = depth(&s);
    let r = run(&mut s, "legend.pick", json!({"id": row.0, "move": true}));
    assert_eq!(r["moved"], json!(2));
    assert_eq!((parent(&s, a), parent(&s, b), parent(&s, inside)), (Some(row), Some(row), Some(row)));
    let kids: Vec<NodeId> = node(&s, row).children().unwrap().iter().map(|c| c.id).collect();
    assert_eq!(kids, vec![inside, a, b], "moved to the top, in their order");
    assert_eq!((weight(&s, a), weight(&s, b), weight(&s, inside)), (0.8, 0.8, 0.8));
    assert_eq!(current(&s), Some(row));
    let mut sel = s.doc().unwrap().selection.objects.clone();
    sel.sort();
    let mut want = vec![a, b, inside];
    want.sort();
    assert_eq!(sel, want, "the selection stays");
    assert_eq!(depth(&s), before + 1);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!((parent(&s, a), parent(&s, b)), (Some(other), Some(other)));
    assert_eq!((weight(&s, a), weight(&s, b)), (1.0, 1.0));
}

#[test]
fn picking_a_locked_or_hidden_row_does_not_move() {
    let mut s = session();
    let row = layer(&mut s, "바닥선");
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.8}));
    let other = layer(&mut s, "Layer 2");
    let a = rect(&mut s, 30.0);
    for prop in ["locked", "visible"] {
        run(&mut s, "layer.setProps", json!({"ids": [row.0], "locked": prop == "locked", "visible": prop != "visible"}));
        run(&mut s, "select.set", json!({"ids": [a.0]}));
        let before = (doc_json(&s), depth(&s), current(&s));
        assert!(s.execute("legend.pick", &json!({"id": row.0, "move": true})).is_err(), "{prop}");
        assert_eq!((doc_json(&s), depth(&s), current(&s)), before, "{prop}: nothing changes");
        assert_eq!(parent(&s, a), Some(other));
        run(&mut s, "layer.setProps", json!({"ids": [row.0], "locked": false, "visible": true}));
    }
}

/// A locked or hidden row takes no line, so it isn't picked: the pen would take a look its lines
/// can't have (they would go to another layer and show as set apart).
#[test]
fn picking_a_locked_or_hidden_row_is_refused() {
    let mut s = session();
    let row = layer(&mut s, "조사구역 경계");
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 1.0, "line": "dashdot"}));
    let other = layer(&mut s, "시굴격자");
    run(&mut s, "legend.set", json!({"id": other.0, "weight": 0.5}));
    run(&mut s, "legend.pick", json!({"id": other.0}));
    run(&mut s, "select.none", json!({}));
    for prop in ["locked", "visible"] {
        run(&mut s, "layer.setProps", json!({"ids": [row.0], "locked": prop == "locked", "visible": prop != "visible"}));
        let before = (doc_json(&s), depth(&s), current(&s), s.new_art_stroke());
        assert!(s.execute("legend.pick", &json!({"id": row.0})).is_err(), "{prop}");
        assert_eq!((doc_json(&s), depth(&s), current(&s), s.new_art_stroke()), before, "{prop}: the current layer and the pen stay");
        run(&mut s, "layer.setProps", json!({"ids": [row.0], "locked": false, "visible": true}));
    }
    run(&mut s, "legend.pick", json!({"id": row.0}));
    assert_eq!(current(&s), Some(row));
}

// ---------- legend.revert ----------

#[test]
fn revert_brings_overrides_back() {
    let mut s = session();
    let plain_layer = layer(&mut s, "plain");
    let plain = rect(&mut s, 200.0);
    let row = layer(&mut s, "단면 위치선");
    let lines: Vec<NodeId> = (0..3).map(|i| open_line(&mut s, f64::from(i) * 30.0)).collect();
    set_weight(&mut s, &lines, 0.2);
    set_dash(&mut s, &lines, &LineType::DashDot);
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.2, "line": "dashdot"}));
    let (a, b) = (lines[0], lines[1]);
    set_weight(&mut s, &[a], 0.5);
    set_dash(&mut s, &[b], &LineType::Solid);
    assert_eq!(overrides(&s, row), vec![b, a]);
    let before = depth(&s);
    let r = run(&mut s, "legend.revert", json!({"ids": [a.0, plain.0]}));
    assert_eq!(r["lines"], json!(1), "a line outside a row with a look is not reverted");
    assert_eq!((weight(&s, a), line_type(&s, a)), (0.2, LineType::DashDot));
    assert_eq!(line_type(&s, b), LineType::Solid);
    assert_eq!(depth(&s), before + 1);
    run(&mut s, "select.set", json!({"ids": [b.0]}));
    let r = run(&mut s, "legend.revert", json!({}));
    assert_eq!(r["lines"], json!(1));
    assert_eq!(line_type(&s, b), LineType::DashDot);
    set_weight(&mut s, &[a, b], 0.5);
    run(&mut s, "layer.setProps", json!({"ids": [b.0], "locked": true}));
    let r = run(&mut s, "legend.revert", json!({"all": true}));
    assert_eq!((r["lines"].clone(), r["skipped"].clone()), (json!(1), json!(1)));
    assert_eq!((weight(&s, a), weight(&s, b)), (0.2, 0.5));
    assert_eq!(parent(&s, plain), Some(plain_layer));
    // Nothing selected and no ids: the pen takes the current row's look.
    run(&mut s, "select.none", json!({}));
    run(&mut s, "layer.setCurrent", json!({"id": row.0}));
    run(&mut s, "stroke.set", json!({"weight": 3, "dash": null}));
    let before = (doc_json(&s), depth(&s));
    run(&mut s, "legend.revert", json!({}));
    assert_eq!((s.new_art_stroke().width, LineType::of(&s.new_art_stroke())), (0.2, LineType::DashDot));
    assert_eq!((doc_json(&s), depth(&s)), before, "the pen is not the document");
    // Nothing to revert: no undo step.
    let before = depth(&s);
    run(&mut s, "legend.revert", json!({"all": true}));
    assert_eq!(depth(&s), before);
}

// ---------- legend.weights ----------

/// Rows 1.0 dash-dot, 0.8, 0.5, 0.2 and 0.25 (each with a line), an override 0.7087 in the 0.8 row,
/// a locked 0.5 line, a hidden 0.5 row, and a plain layer with lines 0.7087 and 0.5.
struct WeightsDoc {
    rows: Vec<NodeId>,
    row_lines: Vec<NodeId>,
    over: NodeId,
    locked: NodeId,
    hidden_row: NodeId,
    hidden_line: NodeId,
    plain: Vec<NodeId>,
}

fn weights_doc() -> (Session, WeightsDoc) {
    let mut s = session();
    let plain_layer = layer(&mut s, "plain");
    let p1 = open_line(&mut s, 0.0);
    let p2 = open_line(&mut s, 30.0);
    set_weight(&mut s, &[p1], 0.7087);
    set_weight(&mut s, &[p2], 0.5);
    let _ = plain_layer;
    let mut rows = vec![];
    let mut row_lines = vec![];
    for (i, (name, w, line)) in
        [("경계", 1.0, "dashdot"), ("굴광선", 0.8, "solid"), ("바닥선", 0.5, "solid"), ("단면 위치선", 0.2, "solid"), ("얇은 선", 0.25, "solid")]
            .into_iter()
            .enumerate()
    {
        let row = layer(&mut s, name);
        let l = open_line(&mut s, f64::from(i as u32) * 30.0);
        set_weight(&mut s, &[l], w);
        if line == "dashdot" {
            set_dash(&mut s, &[l], &LineType::DashDot);
        }
        run(&mut s, "legend.set", json!({"id": row.0, "weight": w, "line": line}));
        rows.push(row);
        row_lines.push(l);
    }
    run(&mut s, "layer.setCurrent", json!({"id": rows[1].0}));
    let over = open_line(&mut s, 200.0);
    set_weight(&mut s, &[over], 0.7087);
    run(&mut s, "layer.setCurrent", json!({"id": rows[2].0}));
    let locked = open_line(&mut s, 230.0);
    set_weight(&mut s, &[locked], 0.5);
    run(&mut s, "layer.setProps", json!({"ids": [locked.0], "locked": true}));
    let hidden_row = layer(&mut s, "숨긴 줄");
    let hidden_line = open_line(&mut s, 260.0);
    set_weight(&mut s, &[hidden_line], 0.5);
    run(&mut s, "legend.set", json!({"id": hidden_row.0, "weight": 0.5}));
    run(&mut s, "layer.setProps", json!({"ids": [hidden_row.0], "visible": false}));
    run(&mut s, "select.none", json!({}));
    (s, WeightsDoc { rows, row_lines, over, locked, hidden_row, hidden_line, plain: vec![p1, p2] })
}

fn row_weight(s: &Session, row: NodeId) -> Option<f64> {
    legend_of(s, row).and_then(|l| l.weight)
}

#[test]
fn weights_by_ratio_round_to_a_thousandth() {
    let (mut s, w) = weights_doc();
    let before = (doc_json(&s), depth(&s));
    let r = run(&mut s, "legend.weights", json!({"how": "ratio", "ratio": 0.8}));
    let want = [0.8, 0.64, 0.4, 0.16, 0.2];
    for ((row, line), to) in w.rows.iter().zip(&w.row_lines).zip(want) {
        assert_eq!(row_weight(&s, *row), Some(to), "row {row:?}");
        assert_eq!(weight(&s, *line), to, "its line follows");
    }
    assert_eq!(line_type(&s, w.row_lines[0]), LineType::DashDot, "line types stay");
    assert_eq!(weight(&s, w.over), 0.7087, "a line set apart stays without `overrides`");
    assert_eq!((weight(&s, w.plain[0]), weight(&s, w.plain[1])), (0.567, 0.4), "lines on plain layers follow the rule");
    assert_eq!(weight(&s, w.locked), 0.5, "locked lines are skipped");
    assert_eq!(r["skipped"], json!(1));
    assert_eq!((row_weight(&s, w.hidden_row), weight(&s, w.hidden_line)), (Some(0.4), 0.4), "hidden rows change");
    // Rows: the five plus the hidden one; lines: five row lines, the hidden one, two plain.
    assert_eq!(r["rows"].as_array().map(Vec::len), Some(6));
    assert_eq!(r["lines"], json!(8));
    let first = &r["rows"][0];
    assert!(first["id"].is_u64() && first["name"].is_string() && first["from"].is_f64() && first["to"].is_f64(), "{first}");
    assert_eq!(depth(&s), before.1 + 1, "one undo step");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(doc_json(&s), before.0, "one undo brings every weight back");
    run(&mut s, "legend.weights", json!({"how": "ratio", "ratio": 0.8, "overrides": true}));
    assert_eq!(weight(&s, w.over), 0.567, "with `overrides`, set-apart lines too");
}

#[test]
fn weights_by_value_change_only_that_weight() {
    let (mut s, w) = weights_doc();
    set_weight(&mut s, &[w.over], 0.5);
    let r = run(&mut s, "legend.weights", json!({"how": "value", "from": 0.5000000001, "to": 0.4}));
    assert_eq!(row_weight(&s, w.rows[2]), Some(0.4));
    assert_eq!(weight(&s, w.row_lines[2]), 0.4);
    assert_eq!(row_weight(&s, w.rows[1]), Some(0.8), "other weights stay");
    assert_eq!((weight(&s, w.plain[0]), weight(&s, w.plain[1])), (0.7087, 0.4));
    assert_eq!(weight(&s, w.over), 0.5, "an override of 0.5 in the 0.8 row stays without `overrides`");
    assert_eq!(r["skipped"], json!(1));
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "legend.weights", json!({"how": "value", "from": 0.5, "to": 0.4, "overrides": true}));
    assert_eq!(weight(&s, w.over), 0.4);
}

#[test]
fn weights_step_along_the_chip_ladder() {
    let mut s = session();
    layer(&mut s, "plain");
    let ws = [0.5, 0.6, 1.0, 0.1, 0.0, 0.25, 2.0];
    let lines: Vec<NodeId> = ws.iter().enumerate().map(|(i, _)| open_line(&mut s, i as f64 * 30.0)).collect();
    for (l, w) in lines.iter().zip(ws) {
        set_weight(&mut s, &[*l], w);
    }
    let row = layer(&mut s, "row");
    let rl = open_line(&mut s, 300.0);
    set_weight(&mut s, &[rl], 0.5);
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.5}));
    run(&mut s, "legend.weights", json!({"how": "step", "step": 1}));
    let up: Vec<f64> = lines.iter().map(|l| weight(&s, *l)).collect();
    assert_eq!(up, vec![0.8, 0.8, 1.0, 0.2, 0.1, 0.3, 2.0]);
    assert_eq!((row_weight(&s, row), weight(&s, rl)), (Some(0.8), 0.8));
    run(&mut s, "edit.undo", json!({}));
    run(&mut s, "legend.weights", json!({"how": "step", "step": -1}));
    let down: Vec<f64> = lines.iter().map(|l| weight(&s, *l)).collect();
    assert_eq!(down, vec![0.3, 0.5, 0.8, 0.1, 0.0, 0.2, 1.0]);
    assert_eq!(row_weight(&s, row), Some(0.3));
}

#[test]
fn weights_plan_matches_the_command() {
    for p in [
        json!({"how": "ratio", "ratio": 0.8}),
        json!({"how": "ratio", "ratio": 1.25, "overrides": true}),
        json!({"how": "value", "from": 0.5, "to": 0.35}),
        json!({"how": "step", "step": -1}),
    ] {
        let (mut s, _) = weights_doc();
        let plan = crate::cmd::legend::plan_weights(&s.doc().unwrap().doc, &p).unwrap();
        let r = run(&mut s, "legend.weights", p.clone());
        assert_eq!(serde_json::to_value(&plan.rows).unwrap(), r["rows"], "{p}");
        assert_eq!(json!(plan.lines.len()), r["lines"], "{p}");
        assert_eq!(json!(plan.skipped), r["skipped"], "{p}");
        for (id, to) in &plan.lines {
            assert_eq!(weight(&s, *id), *to, "{p}");
        }
    }
}

#[test]
fn legend_weights_checks_its_params() {
    let (mut s, _) = weights_doc();
    let before = (doc_json(&s), depth(&s));
    for p in [
        json!({}),
        json!({"how": "zoom"}),
        json!({"how": "ratio"}),
        json!({"how": "ratio", "ratio": 0}),
        json!({"how": "ratio", "ratio": -1}),
        json!({"how": "ratio", "ratio": "x"}),
        json!({"how": "value", "to": 0.4}),
        json!({"how": "value", "from": 0.5}),
        json!({"how": "value", "from": "a", "to": 0.4}),
        json!({"how": "step"}),
        json!({"how": "step", "step": 2}),
        json!({"how": "step", "step": 0}),
        json!({"how": "ratio", "ratio": 0.8, "overrides": "yes"}),
    ] {
        assert!(s.execute("legend.weights", &p).is_err(), "{p} should be refused");
        assert_eq!((doc_json(&s), depth(&s)), before, "{p} changed the document");
    }
}

// ---------- legend.addGroup ----------

/// A row as the group tables give it: name, weight, line, favourite, strata, sample.
type RowLook = (String, Option<f64>, LineType, bool, Option<String>, LegendSample);

/// The top-level layers' rows, top first.
fn rows_top_first(s: &Session) -> Vec<RowLook> {
    s.doc()
        .unwrap()
        .doc
        .layers
        .iter()
        .rev()
        .filter_map(|l| {
            let g = l.legend()?;
            Some((l.display_name().to_string(), g.weight, g.line.clone(), g.favourite, g.strata.clone(), g.sample.clone()))
        })
        .collect()
}

#[test]
fn adding_the_plan_group_makes_its_rows_in_order() {
    let mut s = session();
    let first = s.doc().unwrap().doc.layers[0].id;
    let before = (doc_json(&s), depth(&s));
    let r = run(&mut s, "legend.addGroup", json!({"group": "plan"}));
    assert_eq!(r["added"].as_array().map(Vec::len), Some(15));
    assert_eq!(r["skipped"], json!(0));
    use LegendSample::{Area, Line, Mark};
    use LineType::{DashDot, Solid};
    let m = |s: &str| Mark(s.to_string());
    let k = |s: &str| Some(s.to_string());
    let want: Vec<RowLook> = vec![
        ("조사구역 경계".into(), Some(1.0), DashDot, false, k("survey_area"), Area),
        ("시굴격자".into(), Some(0.5), Solid, false, k("trial_trench"), Area),
        ("굴광선 (유구 면)".into(), Some(0.8), Solid, true, k("feature_poly"), Area),
        ("바닥선".into(), Some(0.5), Solid, true, None, Area),
        ("유구 선 (구·벽구)".into(), Some(0.5), Solid, false, k("feature_line"), Line),
        ("단면 위치선".into(), Some(0.2), DashDot, true, k("section_line"), Line),
        ("기준점".into(), None, Solid, false, k("control_points"), m("△")),
        ("유물 출토 위치".into(), Some(0.2), Solid, false, k("artifact_point"), m("③")),
        ("내부 시설".into(), Some(0.5), Solid, false, None, Area),
        ("돌".into(), Some(0.5), Solid, false, None, Area),
        ("경사 표시".into(), Some(0.2), Solid, false, None, Area),
        ("표고점".into(), None, Solid, false, None, m("·152")),
        ("격자 · 도곽".into(), Some(0.1), Solid, false, None, Line),
        ("글자 · 번호".into(), None, Solid, false, None, m("가")),
        ("밑그림".into(), None, Solid, false, None, Area),
    ];
    assert_eq!(rows_top_first(&s), want);
    let layers = &s.doc().unwrap().doc.layers;
    assert_eq!(layers.first().map(|l| l.id), Some(first), "the rows go above the drawing's layers");
    assert!(layers.iter().all(|l| l.legend().is_none_or(|g| g.group == Some(LegendGroup::Plan))));
    let under = layers.iter().find(|l| l.display_name() == "밑그림").unwrap();
    assert!(under.is_template() && under.locked, "the underlay row is a template layer");
    assert_eq!(depth(&s), before.1 + 1, "one undo step");
    let again = run(&mut s, "legend.addGroup", json!({"group": "plan"}));
    assert_eq!((again["added"].clone(), again["skipped"].clone()), (json!([]), json!(15)));
    assert_eq!(depth(&s), before.1 + 1, "adding nothing is no undo step");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(doc_json(&s), before.0);
}

#[test]
fn section_and_finds_groups_set_their_layer_options() {
    let mut s = session();
    let r = run(&mut s, "legend.addGroup", json!({"group": "section"}));
    assert_eq!(r["added"].as_array().map(Vec::len), Some(12));
    let names: Vec<String> = rows_top_first(&s).into_iter().map(|r| r.0).collect();
    assert_eq!(
        names,
        [
            "단면선 (잘린 땅)",
            "잘린 돌·유물·기와",
            "보이는 돌·유물",
            "해발선 50 cm",
            "해발선 10 cm",
            "해발 숫자",
            "토층선",
            "불분명한 토층선",
            "교란",
            "생토",
            "토층 번호",
            "desca 원본"
        ]
    );
    let rows = rows_top_first(&s);
    assert_eq!((rows[0].1, rows[0].3), (Some(0.8), true));
    assert_eq!((rows[7].1, rows[7].2.clone()), (Some(0.5), LineType::Dotted));
    assert_eq!(rows[5].5, LegendSample::Mark("152.5m".into()));
    let layers = &s.doc().unwrap().doc.layers;
    let desca = layers.iter().find(|l| l.display_name() == "desca 원본").unwrap();
    assert!(desca.is_template() && !desca.visible, "desca's original is a hidden template");
    let r = run(&mut s, "legend.addGroup", json!({"group": "finds"}));
    assert_eq!(r["added"].as_array().map(Vec::len), Some(7));
    let finds: Vec<(String, Option<f64>, LineType)> = rows_top_first(&s).into_iter().take(7).map(|r| (r.0, r.1, r.2)).collect();
    assert_eq!(
        finds,
        vec![
            ("외형선".into(), Some(0.8), LineType::Solid),
            ("단면".into(), Some(0.8), LineType::Solid),
            ("내면선".into(), Some(0.5), LineType::Solid),
            ("중심선".into(), Some(0.2), LineType::DashDot),
            ("문양 · 조정흔".into(), Some(0.2), LineType::Solid),
            ("복원선".into(), Some(0.5), LineType::Dashed),
            ("계측선".into(), None, LineType::Solid),
        ]
    );
    let layers = &s.doc().unwrap().doc.layers;
    let measure = layers.iter().find(|l| l.display_name() == "계측선").unwrap();
    assert!(matches!(measure.kind, vectorcraft_doc::NodeKind::Layer { printable: false, .. }), "measuring lines don't print");
    assert!(s.execute("legend.addGroup", &json!({"group": "maps"})).is_err());
    assert!(s.execute("legend.addGroup", &json!({})).is_err());
}

#[test]
fn rows_already_there_are_not_added_twice() {
    let mut s = session();
    let same_name = layer(&mut s, " 굴광선 (유구 면) ");
    let same_key = layer(&mut s, "경계");
    run(&mut s, "legend.set", json!({"id": same_key.0, "strata": "survey_area", "weight": 0.7}));
    let r = run(&mut s, "legend.addGroup", json!({"group": "plan"}));
    assert_eq!((r["added"].as_array().map(Vec::len), r["skipped"].clone()), (Some(13), json!(2)));
    assert_eq!(legend_of(&s, same_name), None, "an existing layer is left as it is");
    assert_eq!(legend_of(&s, same_key).unwrap().weight, Some(0.7));
}

// ---------- legend.add ----------

#[test]
fn adding_a_row_takes_the_selected_lines_look_and_moves_them() {
    let mut s = session();
    let base = s.doc().unwrap().doc.layers[0].id;
    let top = layer(&mut s, "위");
    run(&mut s, "layer.setCurrent", json!({"id": base.0}));
    let a = rect(&mut s, 0.0);
    let b = open_line(&mut s, 30.0);
    set_weight(&mut s, &[a], 0.35);
    set_dash(&mut s, &[a], &LineType::Dashed);
    run(&mut s, "select.set", json!({"ids": [a.0, b.0]}));
    let before = (doc_json(&s), depth(&s));
    let r = run(&mut s, "legend.add", json!({"name": "3호 수혈 목탄층"}));
    let row = id(&r);
    assert_eq!(r["moved"], json!(2));
    let layers: Vec<NodeId> = s.doc().unwrap().doc.layers.iter().map(|l| l.id).collect();
    assert_eq!(layers, vec![base, row, top], "directly above the current layer");
    assert_eq!(node(&s, row).display_name(), "3호 수혈 목탄층");
    let l = legend_of(&s, row).unwrap();
    assert_eq!((l.weight, l.line.clone(), l.sample.clone()), (Some(0.35), LineType::Dashed, LegendSample::Area));
    assert_eq!((parent(&s, a), parent(&s, b)), (Some(row), Some(row)));
    assert_eq!((weight(&s, a), weight(&s, b)), (0.35, 1.0), "moved lines keep their look");
    assert_eq!(overrides(&s, row), vec![b]);
    assert_eq!(current(&s), Some(row));
    assert_eq!(depth(&s), before.1 + 1, "one undo step");
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(doc_json(&s), before.0);
}

#[test]
fn adding_a_row_with_nothing_selected_takes_the_pen() {
    let mut s = session();
    run(&mut s, "select.none", json!({}));
    run(&mut s, "stroke.set", json!({"weight": 0.2, "dash": LineType::Dotted.dash_pt()}));
    let r = run(&mut s, "legend.add", json!({}));
    let row = id(&r);
    assert_eq!(r["moved"], json!(0));
    let l = legend_of(&s, row).unwrap();
    assert_eq!((l.weight, l.line, l.sample), (Some(0.2), LineType::Dotted, LegendSample::Line));
    assert_eq!(node(&s, row).display_name(), "Layer 2");
}

/// The look comes from the first selected line, as the palette shows it (`shown_stroke`), not the
/// lowest one.
#[test]
fn adding_a_row_takes_the_first_selected_lines_look() {
    let mut s = session();
    let low = rect(&mut s, 0.0);
    let high = rect(&mut s, 30.0);
    set_weight(&mut s, &[low], 0.35);
    set_dash(&mut s, &[low], &LineType::Dashed);
    set_weight(&mut s, &[high], 0.5);
    run(&mut s, "select.set", json!({"ids": [high.0, low.0]}));
    assert_eq!(s.shown_stroke().map(|st| st.width), Some(0.5), "the palette shows the first selected line");
    let row = id(&run(&mut s, "legend.add", json!({})));
    let l = legend_of(&s, row).unwrap();
    assert_eq!((l.weight, l.line), (Some(0.5), LineType::Solid));
    assert_eq!(overrides(&s, row), vec![low]);
}

/// A row's line type is kept as a file keeps it: a line with more dashes than a row holds
/// ([`legend::MAX_DASHES`]) makes a solid row, which a save and an open give back as it was (code
/// review 2 C15).
#[test]
fn a_row_from_a_line_with_many_dashes_opens_again_as_it_was() {
    let mut s = session();
    let a = open_line(&mut s, 0.0);
    let dash: Vec<f64> = (0..=legend::MAX_DASHES).map(|i| 1.0 + i as f64).collect();
    run(&mut s, "stroke.set", json!({"dash": dash, "ids": [a.0]}));
    assert_eq!(line_type(&s, a), LineType::Custom(dash.clone()), "the line keeps its {} dashes", dash.len());
    run(&mut s, "select.set", json!({"ids": [a.0]}));
    let row = id(&run(&mut s, "legend.add", json!({})));
    let here = legend_of(&s, row).unwrap();
    let back = vectorcraft_format::load(&vectorcraft_format::save(&s.doc().unwrap().doc, false)).unwrap();
    assert_eq!(back.legend(row), Some(&here), "the row after a save and an open");
    assert_eq!(here.line, LineType::Solid);
    assert_eq!(overrides(&s, row), vec![a], "its line is set apart");
}

/// Each undo step is named after its command, which the Edit menu shows translated.
#[test]
fn legend_undo_steps_are_named_after_their_commands() {
    let label = |s: &Session| s.doc().unwrap().history.undo.last().unwrap().label.clone();
    let mut s = session();
    let row = layer(&mut s, "바닥선");
    run(&mut s, "legend.set", json!({"id": row.0, "weight": 0.5}));
    assert_eq!(label(&s), "Set Legend Entry");
    let other = layer(&mut s, "Layer 3");
    let a = rect(&mut s, 0.0);
    set_weight(&mut s, &[a], 0.8);
    run(&mut s, "select.set", json!({"ids": [a.0]}));
    run(&mut s, "legend.pick", json!({"id": row.0, "move": true}));
    assert_eq!(parent(&s, a), Some(row));
    assert_eq!(label(&s), "Use Legend Entry");
    set_weight(&mut s, &[a], 0.3);
    run(&mut s, "legend.revert", json!({"ids": [a.0]}));
    assert_eq!(weight(&s, a), 0.5);
    assert_eq!(label(&s), "Revert to the Legend");
    run(&mut s, "legend.weights", json!({"how": "ratio", "ratio": 0.8}));
    assert_eq!(label(&s), "Change Weights at Once");
    run(&mut s, "legend.addGroup", json!({"group": "finds"}));
    assert_eq!(label(&s), "Add Legend Group");
    run(&mut s, "layer.setCurrent", json!({"id": other.0}));
    run(&mut s, "legend.add", json!({}));
    assert_eq!(label(&s), "Add to Legend");
    let specs = crate::cmd::legend::specs();
    for l in
        ["Cut Face", "Set Legend Entry", "Use Legend Entry", "Revert to the Legend", "Change Weights at Once", "Add Legend Group", "Add to Legend"]
    {
        assert!(specs.iter().any(|c| c.label == l), "{l} is a command's name");
    }
}

#[test]
fn legend_set_handles_hatch_field() {
    use vectorcraft_doc::Hatch;
    let mut s = session();
    let row = layer(&mut s, "토층");
    run(&mut s, "legend.set", json!({"id": row.0, "hatch": "45"}));
    assert_eq!(legend_of(&s, row).unwrap().hatch, Hatch::Deg45);
    run(&mut s, "legend.set", json!({"id": row.0, "hatch": "135"}));
    assert_eq!(legend_of(&s, row).unwrap().hatch, Hatch::Deg135);
    run(&mut s, "legend.set", json!({"id": row.0, "hatch": "alt"}));
    assert_eq!(legend_of(&s, row).unwrap().hatch, Hatch::Alternate);
    run(&mut s, "legend.set", json!({"id": row.0, "hatch": "none"}));
    assert_eq!(legend_of(&s, row).unwrap().hatch, Hatch::None);
    assert!(s.execute("legend.set", &json!({"id": row.0, "hatch": "invalid"})).is_err());
}

#[test]
fn cut_face_creates_0_8pt_border_and_0_1pt_hatches() {
    let mut s = session();
    let a = rect(&mut s, 0.0);
    run(&mut s, "select.set", json!({"ids": [a.0]}));
    let res = run(&mut s, "drawing.cutFace", json!({}));
    assert_eq!(res["faces"], 1);
    assert!(res["lines"].as_u64().unwrap() > 0);
    assert!(res["warning"].is_null());

    // Outer border is 0.8 pt
    let face_stroke = node(&s, a).appearance.stroke().unwrap().clone();
    assert_eq!(face_stroke.width, 0.8);

    // Document contains hatch node with 0.1 pt stroke
    let doc = &s.doc().unwrap().doc;
    let hatch_node = doc.layers.iter().flat_map(|l| l.children().into_iter().flatten()).find(|n| n.name.as_deref().unwrap_or("").starts_with("빗금"));
    assert!(hatch_node.is_some());
    let hatch_stroke = hatch_node.unwrap().appearance.stroke().unwrap();
    assert_eq!(hatch_stroke.width, 0.1);

    // Undo restores state
    run(&mut s, "edit.undo", json!({}));
    let restored_stroke = node(&s, a).appearance.stroke().unwrap().clone();
    assert_ne!(restored_stroke.width, 0.8);
}

#[test]
fn cut_face_alternates_angles_for_adjacent_faces() {
    let mut s = session();
    // Two adjacent rectangles sharing an edge at x=40
    let a = NodeId(run(&mut s, "shape.rectangle", json!({"x": 0.0, "y": 0.0, "width": 40.0, "height": 30.0}))["id"].as_u64().unwrap());
    let b = NodeId(run(&mut s, "shape.rectangle", json!({"x": 40.0, "y": 0.0, "width": 40.0, "height": 30.0}))["id"].as_u64().unwrap());
    run(&mut s, "select.set", json!({"ids": [a.0, b.0]}));
    let res = run(&mut s, "drawing.cutFace", json!({}));
    assert_eq!(res["faces"], 2);
    assert!(res["warning"].is_null());

    let doc = &s.doc().unwrap().doc;
    let hatch_names: Vec<String> =
        doc.layers.iter().flat_map(|l| l.children().into_iter().flatten()).filter_map(|n| n.name.clone()).filter(|n| n.starts_with("빗금")).collect();
    assert_eq!(hatch_names.len(), 2);
    assert!(hatch_names.iter().any(|n| n.contains("45°")));
    assert!(hatch_names.iter().any(|n| n.contains("135°")));
}

#[test]
fn cut_face_refuses_when_no_closed_shapes_selected() {
    let mut s = session();
    run(&mut s, "select.set", json!({"ids": []}));
    assert!(s.execute("drawing.cutFace", &json!({})).is_err());
}
