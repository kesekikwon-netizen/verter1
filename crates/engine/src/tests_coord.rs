//! Unit tests for scale and Korean National TM Coordinate System engine commands (J006).

#![allow(clippy::unwrap_used)]

use crate::Session;
use serde_json::json;

fn session_with_doc() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({ "width": 841.89, "height": 595.28 })).unwrap();
    s
}

#[test]
fn test_set_and_get_coord() {
    let mut s = session_with_doc();

    // Initially no coord is set
    let init = s.execute("drawing.getCoord", &json!({})).unwrap();
    assert_eq!(init["coord"], json!(null));

    // Set scale 1:20 and EPSG:5186
    let res = s
        .execute(
            "drawing.setCoord",
            &json!({
                "scale": 20.0,
                "epsg": 5186,
                "origin_x": 147_999.305,
                "origin_y": 98_157.620,
                "anchor_x": 100.0,
                "anchor_y": 200.0,
                "datum": "해발",
            }),
        )
        .unwrap();

    let coord = &res["coord"];
    assert_eq!(coord["scale"], 20.0);
    assert_eq!(coord["epsg"], 5186);
    assert_eq!(coord["origin_x"], 147_999.305);
    assert_eq!(coord["origin_y"], 98_157.620);
    assert_eq!(coord["elevation_datum"], "해발");

    // Verify getCoord
    let got = s.execute("drawing.getCoord", &json!({})).unwrap();
    assert_eq!(got["coord"]["scale"], 20.0);
    assert_eq!(got["coord"]["epsg"], 5186);

    // Verify undo restores previous state
    s.execute("edit.undo", &json!({})).unwrap();
    let after_undo = s.execute("drawing.getCoord", &json!({})).unwrap();
    assert_eq!(after_undo["coord"], json!(null));

    // Verify redo
    s.execute("edit.redo", &json!({})).unwrap();
    let after_redo = s.execute("drawing.getCoord", &json!({})).unwrap();
    assert_eq!(after_redo["coord"]["scale"], 20.0);
    assert_eq!(after_redo["coord"]["epsg"], 5186);
}

#[test]
fn test_set_coord_invalid_parameters() {
    let mut s = session_with_doc();

    assert!(s.execute("drawing.setCoord", &json!({ "scale": -10.0 })).is_err());
    assert!(s.execute("drawing.setCoord", &json!({ "scale": 0.0 })).is_err());
    assert!(s.execute("drawing.setCoord", &json!({ "scale": "invalid" })).is_err());
    assert!(s.execute("drawing.setCoord", &json!({ "epsg": "not_an_int" })).is_err());
    assert!(s.execute("drawing.setCoord", &json!({ "origin_x": "bad" })).is_err());
}

#[test]
fn test_templates_have_default_coord() {
    let mut s = Session::new();

    // Plan template
    s.execute("drawing.newPlan", &json!({})).unwrap();
    let c_plan = s.execute("drawing.getCoord", &json!({})).unwrap();
    assert_eq!(c_plan["coord"]["scale"], 20.0);
    assert_eq!(c_plan["coord"]["epsg"], 5186);

    // Section template
    s.execute("drawing.newSection", &json!({})).unwrap();
    let c_sec = s.execute("drawing.getCoord", &json!({})).unwrap();
    assert_eq!(c_sec["coord"]["scale"], 20.0);
    assert_eq!(c_sec["coord"]["epsg"], 5186);

    // Finds template (1:1)
    s.execute("drawing.newFinds", &json!({})).unwrap();
    let c_finds = s.execute("drawing.getCoord", &json!({})).unwrap();
    assert_eq!(c_finds["coord"]["scale"], 1.0);

    // Sheet template (1:100)
    s.execute("drawing.newSheet", &json!({})).unwrap();
    let c_sheet = s.execute("drawing.getCoord", &json!({})).unwrap();
    assert_eq!(c_sheet["coord"]["scale"], 100.0);
}
