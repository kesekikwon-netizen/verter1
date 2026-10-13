//! Tests for `drawing.makeLevelLines` (Trowel, J015).

#![allow(clippy::unwrap_used)]

use serde_json::json;
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Node, NodeKind};
use vectorcraft_geom::{PathData, Point, SubPath};

use crate::Session;

#[test]
fn test_make_level_lines_in_section_template() {
    let mut s = Session::new();
    s.execute("drawing.newSection", &json!({})).unwrap();

    // Find "단면선 (잘린 땅)" layer to add a trench profile line
    let section_layer_id = {
        let st = s.doc().unwrap();
        st.doc.layers.iter().find(|l| l.display_name().trim() == "단면선 (잘린 땅)").map(|l| l.id).unwrap()
    };

    // Draw an open trench profile: (200, 300) -> (400, 450) -> (600, 300)
    s.edit("Draw Section Profile", |d, _| {
        let stroke = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.8);
        let pd = PathData::new(vec![SubPath::polyline(&[Point::new(200.0, 300.0), Point::new(400.0, 450.0), Point::new(600.0, 300.0)], false)]);
        let n = Node::path(d.alloc_id(), pd, stroke);
        d.insert(Some(section_layer_id), usize::MAX, n)?;
        Ok(())
    })
    .unwrap();

    // Execute drawing.makeLevelLines
    let res = s.execute("drawing.makeLevelLines", &json!({})).unwrap();

    let lines_50 = res["lines_50cm"].as_u64().unwrap();
    let lines_10 = res["lines_10cm"].as_u64().unwrap();
    let labels = res["labels"].as_u64().unwrap();

    assert!(lines_50 > 0, "must generate 50 cm lines: got {lines_50}");
    assert!(lines_10 > 0, "must generate 10 cm lines: got {lines_10}");
    assert_eq!(lines_50, labels, "each 50 cm line has an elevation label");

    // Inspect the generated layers
    let st = s.doc().unwrap();
    let layer_50 = st.doc.layers.iter().find(|l| l.display_name().trim() == "해발선 50 cm").unwrap();
    let layer_10 = st.doc.layers.iter().find(|l| l.display_name().trim() == "해발선 10 cm").unwrap();
    let layer_labels = st.doc.layers.iter().find(|l| l.display_name().trim() == "해발 숫자").unwrap();

    assert!(!layer_50.children().unwrap().is_empty());
    assert!(!layer_10.children().unwrap().is_empty());
    assert!(!layer_labels.children().unwrap().is_empty());

    // Verify label text format: ends with 'm'
    for c in layer_labels.children().unwrap() {
        if let NodeKind::Text(t) = &c.kind {
            let txt = t.plain_text();
            assert!(txt.ends_with('m'), "label must end with m: {txt}");
        }
    }

    // Re-run: idempotency test
    let res2 = s.execute("drawing.makeLevelLines", &json!({})).unwrap();
    assert_eq!(res2["lines_50cm"], res["lines_50cm"]);
    assert_eq!(res2["lines_10cm"], res["lines_10cm"]);
    assert_eq!(res2["labels"], res["labels"]);

    // Test Undo
    s.execute("edit.undo", &json!({})).unwrap();
    // After undoing the second run, the layers still have the first run's lines
    let st = s.doc().unwrap();
    let layer_50 = st.doc.layers.iter().find(|l| l.display_name().trim() == "해발선 50 cm").unwrap();
    assert!(!layer_50.children().unwrap().is_empty());

    // Undo the first run
    s.execute("edit.undo", &json!({})).unwrap();
    let st = s.doc().unwrap();
    let layer_50 = st.doc.layers.iter().find(|l| l.display_name().trim() == "해발선 50 cm").unwrap();
    assert!(layer_50.children().unwrap().is_empty(), "level lines cleared after undo");
}
