//! Scale and Korean National TM Coordinate System engine commands (Trowel, J006).

use serde_json::{Value, json};

use super::*;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "drawing.setCoord",
            "Set Scale and Coordinate System",
            [],
            None,
            "{scale?: number, epsg?: number, origin_x?: number, origin_y?: number, anchor_x?: number, anchor_y?: number, datum?: string} set drawing scale and Korean national TM coordinate system (Trowel) → {coord}",
            has_doc,
            set_coord
        ),
        cmd!(
            "drawing.getCoord",
            "Get Scale and Coordinate System",
            [],
            None,
            "{} get current drawing scale and coordinate reference system → {coord?: DocCoord}",
            has_doc,
            get_coord
        ),
    ]
}

fn set_coord(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.setCoord";

    let scale_opt = if let Some(v) = p.get("scale") {
        let sc = v.as_f64().ok_or_else(|| bad(C, format!("scale must be a positive number, got {v}")))?;
        if sc <= 0.0 || !sc.is_finite() {
            return Err(bad(C, format!("scale must be positive and finite, got {sc}")));
        }
        Some(sc)
    } else {
        None
    };

    let epsg_opt = if let Some(v) = p.get("epsg") {
        let ep = v.as_u64().ok_or_else(|| bad(C, format!("epsg must be a valid integer code, got {v}")))?;
        Some(ep as u32)
    } else {
        None
    };

    let origin_x_opt = if let Some(v) = p.get("origin_x") {
        let x = v.as_f64().ok_or_else(|| bad(C, format!("origin_x must be a number, got {v}")))?;
        if !x.is_finite() {
            return Err(bad(C, "origin_x must be finite"));
        }
        Some(x)
    } else {
        None
    };

    let origin_y_opt = if let Some(v) = p.get("origin_y") {
        let y = v.as_f64().ok_or_else(|| bad(C, format!("origin_y must be a number, got {v}")))?;
        if !y.is_finite() {
            return Err(bad(C, "origin_y must be finite"));
        }
        Some(y)
    } else {
        None
    };

    let anchor_x_opt = if let Some(v) = p.get("anchor_x") {
        let x = v.as_f64().ok_or_else(|| bad(C, format!("anchor_x must be a number, got {v}")))?;
        if !x.is_finite() {
            return Err(bad(C, "anchor_x must be finite"));
        }
        Some(x)
    } else {
        None
    };

    let anchor_y_opt = if let Some(v) = p.get("anchor_y") {
        let y = v.as_f64().ok_or_else(|| bad(C, format!("anchor_y must be a number, got {v}")))?;
        if !y.is_finite() {
            return Err(bad(C, "anchor_y must be finite"));
        }
        Some(y)
    } else {
        None
    };

    let datum_opt = if let Some(v) = p.get("datum") {
        let dt = v.as_str().ok_or_else(|| bad(C, format!("datum must be a string, got {v}")))?;
        Some(dt)
    } else {
        None
    };

    let mut applied_coord = None;

    s.edit("Set Scale and Coordinate System", |d, _sel| {
        let mut cur = d.coord.clone().unwrap_or_default();
        if let Some(sc) = scale_opt {
            cur.scale = sc;
        }
        if let Some(e) = epsg_opt {
            cur.epsg = e;
        }
        if let Some(ox) = origin_x_opt {
            cur.origin_x = ox;
        }
        if let Some(oy) = origin_y_opt {
            cur.origin_y = oy;
        }
        if let Some(ax) = anchor_x_opt {
            cur.anchor_x = ax;
        }
        if let Some(ay) = anchor_y_opt {
            cur.anchor_y = ay;
        }
        if let Some(dt) = datum_opt {
            cur.elevation_datum = dt.to_string();
        }
        applied_coord = Some(cur.clone());
        d.coord = Some(cur);
        Ok(())
    })?;

    Ok(json!({ "coord": applied_coord }))
}

fn get_coord(s: &mut Session, _p: &Value) -> Result<Value> {
    let doc = s.doc()?;
    Ok(json!({ "coord": doc.doc.coord }))
}
