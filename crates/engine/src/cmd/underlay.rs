//! Scanned underlay alignment and GCP error analysis engine commands (Trowel, J008).
//!
//! Provides 2-point similarity registration (rotation, uniform scale, translation)
//! and 4-point affine registration (independent axis scale, shear, rotation, translation)
//! to align scanned excavation grid drawings to document/national TM coordinates,
//! reporting residual errors and RMSE against the national heritage 2.0 mm tolerance.

use serde_json::{Value, json};
use vectorcraft_doc::DocCoord;
use vectorcraft_doc::{Document, ImageObject, NodeId, NodeKind};
use vectorcraft_geom::{Affine, Point};

use super::*;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "underlay.align",
            "Align Scanned Underlay",
            [],
            None,
            "{id?: NodeId, method?: string, gcps: [{scan: [x,y], target?: [x,y], target_tm?: [e,n]}], scan_is_pixel?: bool} align scanned underlay image using 2-point similarity or 4-point affine registration (Trowel) → {id, matrix, residuals_pt, residuals_mm, rmse_pt, rmse_mm, pass}",
            has_doc,
            underlay_align
        ),
        cmd!(
            "underlay.estimate",
            "Estimate Underlay Alignment",
            [],
            None,
            "{method?: string, gcps: [{scan: [x,y], target?: [x,y], target_tm?: [e,n]}]} calculate registration matrix and RMSE without modifying document (Trowel) → {matrix, residuals_pt, residuals_mm, rmse_pt, rmse_mm, pass}",
            has_doc,
            underlay_estimate
        ),
    ]
}

/// A ground control point (GCP) matching a scan coordinate to a target coordinate.
#[derive(Clone, Debug, PartialEq)]
pub struct GcpPoint {
    pub scan: Point,
    pub target: Point,
    pub target_tm: Option<Point>,
}

/// Registration estimation results including residuals and RMSE.
#[derive(Clone, Debug, PartialEq)]
pub struct AlignmentResult {
    pub matrix: Affine,
    pub residuals_pt: Vec<f64>,
    pub residuals_mm: Vec<f64>,
    pub rmse_pt: f64,
    pub rmse_mm: f64,
    pub pass: bool,
}

/// Parse one GCP point from a JSON object.
pub fn parse_gcp(v: &Value, coord: Option<&DocCoord>) -> Result<GcpPoint> {
    let scan_arr = v.get("scan").and_then(|s| s.as_array()).ok_or_else(|| bad("underlay", "GCP requires 'scan': [x, y]"))?;
    if scan_arr.len() < 2 {
        return Err(bad("underlay", "GCP 'scan' must have at least 2 numbers"));
    }
    let sx = scan_arr[0].as_f64().ok_or_else(|| bad("underlay", "scan X must be a number"))?;
    let sy = scan_arr[1].as_f64().ok_or_else(|| bad("underlay", "scan Y must be a number"))?;
    if !sx.is_finite() || !sy.is_finite() {
        return Err(bad("underlay", "scan coordinates must be finite"));
    }
    let scan = Point::new(sx, sy);

    let (target, target_tm) = if let Some(t_arr) = v.get("target").and_then(|t| t.as_array()) {
        if t_arr.len() < 2 {
            return Err(bad("underlay", "GCP 'target' must have at least 2 numbers"));
        }
        let tx = t_arr[0].as_f64().ok_or_else(|| bad("underlay", "target X must be a number"))?;
        let ty = t_arr[1].as_f64().ok_or_else(|| bad("underlay", "target Y must be a number"))?;
        if !tx.is_finite() || !ty.is_finite() {
            return Err(bad("underlay", "target coordinates must be finite"));
        }
        let tm = coord.map(|c| {
            let (tm_x, tm_y) = c.pt_to_tm(tx, ty);
            Point::new(tm_x, tm_y)
        });
        (Point::new(tx, ty), tm)
    } else if let Some(tm_arr) = v.get("target_tm").and_then(|t| t.as_array()) {
        if tm_arr.len() < 2 {
            return Err(bad("underlay", "GCP 'target_tm' must have at least 2 numbers"));
        }
        let ex = tm_arr[0].as_f64().ok_or_else(|| bad("underlay", "target_tm Easting must be a number"))?;
        let ny = tm_arr[1].as_f64().ok_or_else(|| bad("underlay", "target_tm Northing must be a number"))?;
        if !ex.is_finite() || !ny.is_finite() {
            return Err(bad("underlay", "target_tm coordinates must be finite"));
        }
        let c = coord.cloned().unwrap_or_default();
        let (pt_x, pt_y) = c.tm_to_pt(ex, ny);
        (Point::new(pt_x, pt_y), Some(Point::new(ex, ny)))
    } else {
        return Err(bad("underlay", "GCP requires either 'target' [pt_x, pt_y] or 'target_tm' [easting, northing]"));
    };

    Ok(GcpPoint { scan, target, target_tm })
}

/// Solves 2-point (or N-point least-squares) similarity transformation:
/// uniform scaling, rotation, and translation (4 degrees of freedom).
pub fn solve_similarity(gcps: &[GcpPoint]) -> Result<AlignmentResult> {
    if gcps.len() < 2 {
        return Err(bad("underlay", "2-point similarity registration requires at least 2 GCPs"));
    }
    let n = gcps.len() as f64;
    let (mut sum_sx, mut sum_sy) = (0.0, 0.0);
    let (mut sum_tx, mut sum_ty) = (0.0, 0.0);
    for g in gcps {
        sum_sx += g.scan.x;
        sum_sy += g.scan.y;
        sum_tx += g.target.x;
        sum_ty += g.target.y;
    }
    let mean_s = Point::new(sum_sx / n, sum_sy / n);
    let mean_t = Point::new(sum_tx / n, sum_ty / n);

    let mut denom = 0.0;
    let mut num_a = 0.0;
    let mut num_b = 0.0;
    for g in gcps {
        let dx = g.scan.x - mean_s.x;
        let dy = g.scan.y - mean_s.y;
        let d_x_target = g.target.x - mean_t.x;
        let d_y_target = g.target.y - mean_t.y;
        denom += dx * dx + dy * dy;
        num_a += dx * d_x_target + dy * d_y_target;
        num_b += dx * d_y_target - dy * d_x_target;
    }

    if denom < 1e-12 {
        return Err(bad("underlay", "GCP scan points are coincident or too close"));
    }

    let a = num_a / denom;
    let b = num_b / denom;
    let c = -b;
    let d = a;
    let e = mean_t.x - (a * mean_s.x + c * mean_s.y);
    let f = mean_t.y - (b * mean_s.x + d * mean_s.y);

    let matrix = Affine::new([a, b, c, d, e, f]);
    evaluate_residuals(matrix, gcps)
}

/// Solves 4-point (or N >= 3 least-squares) affine transformation:
/// independent axis scaling, shear, rotation, and translation (6 degrees of freedom).
pub fn solve_affine(gcps: &[GcpPoint]) -> Result<AlignmentResult> {
    if gcps.len() < 3 {
        return Err(bad("underlay", "Affine registration requires at least 3 GCPs (typically 4)"));
    }
    let n = gcps.len() as f64;
    let (mut sum_sx, mut sum_sy) = (0.0, 0.0);
    let (mut sum_tx, mut sum_ty) = (0.0, 0.0);
    for g in gcps {
        sum_sx += g.scan.x;
        sum_sy += g.scan.y;
        sum_tx += g.target.x;
        sum_ty += g.target.y;
    }
    let mean_s = Point::new(sum_sx / n, sum_sy / n);
    let mean_t = Point::new(sum_tx / n, sum_ty / n);

    let (mut c_xx, mut c_xy, mut c_yy) = (0.0, 0.0, 0.0);
    let (mut s_x_target_x, mut s_y_target_x) = (0.0, 0.0);
    let (mut s_x_target_y, mut s_y_target_y) = (0.0, 0.0);

    for g in gcps {
        let dx = g.scan.x - mean_s.x;
        let dy = g.scan.y - mean_s.y;
        let d_x_target = g.target.x - mean_t.x;
        let d_y_target = g.target.y - mean_t.y;
        c_xx += dx * dx;
        c_xy += dx * dy;
        c_yy += dy * dy;
        s_x_target_x += dx * d_x_target;
        s_y_target_x += dy * d_x_target;
        s_x_target_y += dx * d_y_target;
        s_y_target_y += dy * d_y_target;
    }

    let det = c_xx * c_yy - c_xy * c_xy;
    if det.abs() < 1e-12 {
        return Err(bad("underlay", "GCP scan points are collinear; affine transformation cannot be determined"));
    }

    let a = (c_yy * s_x_target_x - c_xy * s_y_target_x) / det;
    let c = (-c_xy * s_x_target_x + c_xx * s_y_target_x) / det;
    let b = (c_yy * s_x_target_y - c_xy * s_y_target_y) / det;
    let d = (-c_xy * s_x_target_y + c_xx * s_y_target_y) / det;
    let e = mean_t.x - (a * mean_s.x + c * mean_s.y);
    let f = mean_t.y - (b * mean_s.x + d * mean_s.y);

    let matrix = Affine::new([a, b, c, d, e, f]);
    evaluate_residuals(matrix, gcps)
}

/// Evaluates residuals for each GCP point and computes RMSE in points and paper millimeters.
pub fn evaluate_residuals(matrix: Affine, gcps: &[GcpPoint]) -> Result<AlignmentResult> {
    if !matrix.as_coeffs().iter().all(|v| v.is_finite()) {
        return Err(bad("underlay", "Computed transformation coefficients are non-finite"));
    }
    const PT_TO_MM: f64 = 25.4 / 72.0;

    let mut residuals_pt = Vec::with_capacity(gcps.len());
    let mut residuals_mm = Vec::with_capacity(gcps.len());
    let mut sum_sq_pt = 0.0;
    let mut sum_sq_mm = 0.0;

    for g in gcps {
        let mapped = matrix * g.scan;
        let dx = mapped.x - g.target.x;
        let dy = mapped.y - g.target.y;
        let r_pt = dx.hypot(dy);
        let r_mm = r_pt * PT_TO_MM;

        residuals_pt.push(r_pt);
        residuals_mm.push(r_mm);
        sum_sq_pt += r_pt * r_pt;
        sum_sq_mm += r_mm * r_mm;
    }

    let n = gcps.len() as f64;
    let rmse_pt = (sum_sq_pt / n).sqrt();
    let rmse_mm = (sum_sq_mm / n).sqrt();
    // National heritage excavation standard tolerance: 2.0 mm
    let pass = rmse_mm <= 2.0;

    Ok(AlignmentResult { matrix, residuals_pt, residuals_mm, rmse_pt, rmse_mm, pass })
}

/// Finds the most relevant image node (template layer image or active image).
pub fn find_underlay_image(d: &Document, preferred: Option<NodeId>) -> Option<(NodeId, ImageObject, String)> {
    if let Some(id) = preferred
        && let Some(n) = d.node(id)
        && let NodeKind::Image(im) = &n.kind
    {
        let name = n.name.clone().unwrap_or_else(|| im.link.as_ref().map(|l| l.path.clone()).unwrap_or_else(|| im.key.clone()));
        return Some((id, im.clone(), name));
    }

    for layer in &d.layers {
        if layer.is_template()
            && let Some(children) = layer.children()
        {
            for c in children {
                if let NodeKind::Image(im) = &c.kind {
                    let name = c.name.clone().unwrap_or_else(|| im.link.as_ref().map(|l| l.path.clone()).unwrap_or_else(|| im.key.clone()));
                    return Some((c.id, im.clone(), name));
                }
            }
        }
    }

    for layer in &d.layers {
        if let Some(children) = layer.children() {
            for c in children {
                if let NodeKind::Image(im) = &c.kind {
                    let name = c.name.clone().unwrap_or_else(|| im.link.as_ref().map(|l| l.path.clone()).unwrap_or_else(|| im.key.clone()));
                    return Some((c.id, im.clone(), name));
                }
            }
        }
    }

    None
}

fn compute_alignment(p: &Value, coord: Option<&DocCoord>) -> Result<(AlignmentResult, Vec<GcpPoint>)> {
    let gcp_vals = p.get("gcps").and_then(|g| g.as_array()).ok_or_else(|| bad("underlay", "Missing 'gcps' array"))?;
    if gcp_vals.is_empty() {
        return Err(bad("underlay", "'gcps' array cannot be empty"));
    }

    let mut gcps = Vec::with_capacity(gcp_vals.len());
    for v in gcp_vals {
        gcps.push(parse_gcp(v, coord)?);
    }

    let method_str = p.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let is_two_point = match method_str {
        "twoPoint" | "two_point" | "similarity" => true,
        "affine" | "fourPoint" | "four_point" => false,
        _ => gcps.len() <= 2,
    };

    let result = if is_two_point { solve_similarity(&gcps)? } else { solve_affine(&gcps)? };

    Ok((result, gcps))
}

fn underlay_align(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "underlay.align";
    let coord = s.doc()?.doc.coord.clone();
    let (result, _gcps) = compute_alignment(p, coord.as_ref())?;

    let preferred_id = p.get("id").and_then(|v| v.as_u64()).map(NodeId);
    let scan_is_pixel = p.get("scan_is_pixel").and_then(|v| v.as_bool()).unwrap_or(true);

    let doc = s.doc_mut()?;
    let (target_id, old_xf) = {
        let (id, im, _name) =
            find_underlay_image(&doc.doc, preferred_id).ok_or_else(|| bad(C, "No image node found in template layer or document to align"))?;
        (id, im.xf)
    };

    let new_xf = if scan_is_pixel { result.matrix } else { result.matrix * old_xf };

    s.edit("Align Scanned Underlay", |d, _sel| {
        let node = d.node_mut(target_id).ok_or_else(|| bad(C, format!("Node {target_id:?} not found")))?;
        if let NodeKind::Image(ref mut im) = node.kind {
            im.xf = new_xf;
            Ok(())
        } else {
            Err(bad(C, "Target node is not an image"))
        }
    })?;

    Ok(json!({
        "id": target_id.0,
        "matrix": result.matrix.as_coeffs(),
        "residuals_pt": result.residuals_pt,
        "residuals_mm": result.residuals_mm,
        "rmse_pt": result.rmse_pt,
        "rmse_mm": result.rmse_mm,
        "pass": result.pass,
    }))
}

fn underlay_estimate(s: &mut Session, p: &Value) -> Result<Value> {
    let coord = s.doc()?.doc.coord.clone();
    let (result, _gcps) = compute_alignment(p, coord.as_ref())?;
    Ok(json!({
        "matrix": result.matrix.as_coeffs(),
        "residuals_pt": result.residuals_pt,
        "residuals_mm": result.residuals_mm,
        "rmse_pt": result.rmse_pt,
        "rmse_mm": result.rmse_mm,
        "pass": result.pass,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_point_similarity_exact() {
        let p1 = Point::new(100.0, 100.0);
        let p2 = Point::new(300.0, 100.0);

        // Map through scale 2.0, 90 deg rotation, and translation (50.0, 80.0)
        let model = Affine::translate((50.0, 80.0)) * Affine::rotate(std::f64::consts::FRAC_PI_2) * Affine::scale(2.0);
        let q1 = model * p1;
        let q2 = model * p2;

        let gcps = vec![GcpPoint { scan: p1, target: q1, target_tm: None }, GcpPoint { scan: p2, target: q2, target_tm: None }];

        let res = solve_similarity(&gcps).expect("similarity solution");
        for (a, b) in res.matrix.as_coeffs().iter().zip(model.as_coeffs().iter()) {
            assert!((a - b).abs() < 1e-9, "coeff diff: {a} vs {b}");
        }
        assert!(res.rmse_pt < 1e-9);
        assert!(res.rmse_mm < 1e-9);
        assert!(res.pass);
    }

    #[test]
    fn test_four_point_affine_exact() {
        let pts = [Point::new(0.0, 0.0), Point::new(100.0, 0.0), Point::new(100.0, 50.0), Point::new(0.0, 50.0)];

        // Arbitrary non-uniform scale + shear + translation
        let model = Affine::new([1.5, 0.4, -0.2, 1.8, 120.0, 340.0]);
        let gcps: Vec<_> = pts.iter().map(|&p| GcpPoint { scan: p, target: model * p, target_tm: None }).collect();

        let res = solve_affine(&gcps).expect("affine solution");
        for (a, b) in res.matrix.as_coeffs().iter().zip(model.as_coeffs().iter()) {
            assert!((a - b).abs() < 1e-9, "coeff diff: {a} vs {b}");
        }
        assert!(res.rmse_pt < 1e-9);
        assert!(res.rmse_mm < 1e-9);
        assert!(res.pass);
    }

    #[test]
    fn test_collinear_points_error() {
        let pts = [Point::new(10.0, 10.0), Point::new(20.0, 20.0), Point::new(30.0, 30.0), Point::new(40.0, 40.0)];
        let gcps: Vec<_> = pts.iter().map(|&p| GcpPoint { scan: p, target: p, target_tm: None }).collect();
        let err = solve_affine(&gcps).unwrap_err();
        assert!(err.to_string().contains("collinear"));
    }

    #[test]
    fn test_coincident_points_error() {
        let gcps = vec![
            GcpPoint { scan: Point::new(50.0, 50.0), target: Point::new(10.0, 10.0), target_tm: None },
            GcpPoint { scan: Point::new(50.0, 50.0), target: Point::new(20.0, 20.0), target_tm: None },
        ];
        let err = solve_similarity(&gcps).unwrap_err();
        assert!(err.to_string().contains("coincident"));
    }

    #[test]
    fn test_field_excavation_gcp_realistic_rmse() {
        // Archaeological field excavation grid: 10m x 10m trench surveyed at 1:20 scale.
        // Paper dimension for 10m: 10,000 mm / 20 = 500 mm.
        // 500 mm * (72 / 25.4) = 1417.32 pt.
        // At 300 DPI (1 pt = 4.167 px), scan dimension is ~5905 px.
        // Let scan points be corners with slight scanner shear and paper stretch:
        let coord = DocCoord::new(20.0, 5186, 148_000.0, 98_150.0);
        let p_scan = [Point::new(420.0, 380.0), Point::new(2840.0, 395.0), Point::new(2850.0, 2815.0), Point::new(415.0, 2800.0)];

        // Theoretical affine mapping
        let model = Affine::translate((100.0, 150.0)) * Affine::rotate(0.015) * Affine::scale_non_uniform(0.24, 0.241);

        // Add sub-millimeter field survey / paper distortion noise (approx 0.6 mm ~ 0.9 mm paper error)
        // 1 mm on paper = 72.0 / 25.4 pt ~ 2.835 pt.
        let mm_to_pt = 72.0 / 25.4;
        let noise = [
            (0.64 * mm_to_pt * 0.707, 0.64 * mm_to_pt * 0.707),
            (-0.82 * mm_to_pt * 0.707, 0.82 * mm_to_pt * 0.707),
            (0.95 * mm_to_pt * 0.707, -0.95 * mm_to_pt * 0.707),
            (-0.71 * mm_to_pt * 0.707, -0.71 * mm_to_pt * 0.707),
        ];

        let mut gcps = Vec::new();
        for (i, p) in p_scan.into_iter().enumerate() {
            let base_target = model * p;
            let target = Point::new(base_target.x + noise[i].0, base_target.y + noise[i].1);
            let (tm_x, tm_y) = coord.pt_to_tm(target.x, target.y);
            gcps.push(GcpPoint { scan: p, target, target_tm: Some(Point::new(tm_x, tm_y)) });
        }

        let res = solve_affine(&gcps).expect("affine alignment");
        // Verify RMSE is around 0.6 mm (well within national heritage standard <= 2.0 mm)
        assert!(res.rmse_mm <= 2.0, "rmse_mm: {}", res.rmse_mm);
        assert!(res.rmse_mm > 0.1, "rmse_mm should reflect distortion noise: {}", res.rmse_mm);
        assert!(res.pass);
    }

    #[test]
    fn test_exceeds_tolerance_warning() {
        let p_scan = [Point::new(100.0, 100.0), Point::new(500.0, 100.0), Point::new(500.0, 500.0), Point::new(100.0, 500.0)];
        // Add 5.0 mm error
        let mm_to_pt = 72.0 / 25.4;
        let p_target = [
            Point::new(100.0, 100.0),
            Point::new(500.0, 100.0),
            Point::new(500.0, 500.0 + 5.0 * mm_to_pt),
            Point::new(100.0, 500.0 - 5.0 * mm_to_pt),
        ];
        let gcps: Vec<_> = p_scan.iter().zip(p_target.iter()).map(|(&s, &t)| GcpPoint { scan: s, target: t, target_tm: None }).collect();
        let res = solve_affine(&gcps).expect("affine solution");
        assert!(res.rmse_mm > 2.0, "rmse_mm should exceed tolerance: {}", res.rmse_mm);
        assert!(!res.pass, "pass should be false for > 2.0 mm error");
    }

    #[test]
    fn test_underlay_align_and_undo_redo() {
        use vectorcraft_doc::Node;

        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 800, "height": 600})).expect("new doc");

        // Insert a template layer with an image
        let img_id = s
            .edit("Setup test underlay", |d, _sel| {
                let mut layer = Node::new(
                    d.alloc_id(),
                    NodeKind::Layer {
                        color: vectorcraft_doc::LayerColor::Preset(0),
                        template: true,
                        printable: false,
                        children: vec![],
                        clip: false,
                        preview: true,
                        dim_images: None,
                        legend: None,
                    },
                );
                layer.name = Some("밑그림 · 현장실측.png".to_string());

                let img_node = Node::new(
                    d.alloc_id(),
                    NodeKind::Image(ImageObject {
                        key: "scan1".to_string(),
                        width: 3000,
                        height: 3000,
                        xf: Affine::scale(0.5),
                        link: None,
                        placement: Default::default(),
                    }),
                );
                let id = img_node.id;
                layer.children_mut().ok_or_else(|| bad("test", "not a layer"))?.push(std::sync::Arc::new(img_node));
                d.insert(None, 0, layer).map_err(|e| bad("test", e.to_string()))?;
                Ok(id)
            })
            .expect("setup test underlay");

        // Run underlay.align with 2 points
        let p = json!({
            "id": img_id.0,
            "method": "twoPoint",
            "gcps": [
                {"scan": [0.0, 0.0], "target": [100.0, 100.0]},
                {"scan": [1000.0, 0.0], "target": [600.0, 100.0]}
            ]
        });

        let align_res = s.execute("underlay.align", &p).expect("underlay.align should succeed");
        assert_eq!(align_res["id"].as_u64(), Some(img_id.0));
        assert!(align_res["pass"].as_bool().unwrap_or(false));

        // Verify image xf changed to scale 0.5 (from 1000 to 500) and translate 100, 100
        {
            let d = s.doc().unwrap();
            if let Some(NodeKind::Image(im)) = d.doc.node(img_id).map(|n| &n.kind) {
                assert!((im.xf.as_coeffs()[0] - 0.5).abs() < 1e-6);
                assert!((im.xf.as_coeffs()[4] - 100.0).abs() < 1e-6);
            } else {
                panic!("Image not found");
            }
        }

        // Test Undo
        s.execute("edit.undo", &json!({})).expect("undo should succeed");
        {
            let d = s.doc().unwrap();
            if let Some(NodeKind::Image(im)) = d.doc.node(img_id).map(|n| &n.kind) {
                // Restored to initial xf: Affine::scale(0.5) with translate 0, 0
                assert!((im.xf.as_coeffs()[4] - 0.0).abs() < 1e-6);
            } else {
                panic!("Image not found after undo");
            }
        }

        // Test Redo
        s.execute("edit.redo", &json!({})).expect("redo should succeed");
        {
            let d = s.doc().unwrap();
            if let Some(NodeKind::Image(im)) = d.doc.node(img_id).map(|n| &n.kind) {
                assert!((im.xf.as_coeffs()[4] - 100.0).abs() < 1e-6);
            } else {
                panic!("Image not found after redo");
            }
        }
    }
}
