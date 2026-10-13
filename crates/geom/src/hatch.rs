//! Hatch line generation for closed vector paths.
//!
//! Generates parallel diagonal hatch lines at a specified angle and spacing inside a closed path.
//! Used for archaeological CAD drawings (e.g. cut faces, section strata) and vector illustrations.
//!
//! All calculations are finite-safe with strictly capped line counts to prevent infinite loops.

use kurbo::{BezPath, PathEl, Point, Shape};

/// Standard paper hatch spacing: 1.0 mm in PostScript points (72.0 / 25.4 ≈ 2.8346457 pt).
pub const HATCH_GAP_PT: f64 = 72.0 / 25.4;
/// Standard cut-face outer stroke width: 0.8 pt.
pub const CUT_FACE_STROKE_WIDTH: f64 = 0.8;
/// Standard cut-face hatch stroke width: 0.1 pt.
pub const HATCH_STROKE_WIDTH: f64 = 0.1;

/// Maximum number of scanlines generated per path to guarantee termination and memory safety.
const MAX_SCANLINES: usize = 20_000;

/// Generate diagonal hatch lines filling `path` with spacing `gap` (in points) at angle `angle_rad` (in radians).
///
/// Returns a [`BezPath`] containing disconnected line segments (`MoveTo` followed by `LineTo`).
/// If `path` is empty or degenerate, returns an empty [`BezPath`].
pub fn hatch(path: &BezPath, gap: f64, angle_rad: f64) -> BezPath {
    let mut out = BezPath::new();
    let b = path.bounding_box();
    if !b.width().is_finite() || !b.height().is_finite() || b.width() <= 0.0 || b.height() <= 0.0 {
        return out;
    }

    let (sin_a, cos_a) = angle_rad.sin_cos();
    if !sin_a.is_finite() || !cos_a.is_finite() {
        return out;
    }

    // Coordinate rotation by -angle:
    // x' =  x * cos + y * sin
    // y' = -x * sin + y * cos
    // In rotated frame, hatch lines are horizontal scanlines y' = const.
    let rot = |p: Point| Point::new(p.x * cos_a + p.y * sin_a, -p.x * sin_a + p.y * cos_a);

    // Inverse rotation by +angle:
    // x = x' * cos - y' * sin
    // y = x' * sin + y' * cos
    let unrot = |p: Point| Point::new(p.x * cos_a - p.y * sin_a, p.x * sin_a + p.y * cos_a);

    let mut edges: Vec<(Point, Point)> = Vec::new();
    let mut first = Point::ZERO;
    let mut last = Point::ZERO;

    // Flatten curves with tolerance proportional to gap.
    let tol = (gap.max(1e-3) * 0.1).clamp(0.01, 2.0);
    kurbo::flatten(path.iter(), tol, |el| match el {
        PathEl::MoveTo(p) => {
            if last != first {
                edges.push((rot(last), rot(first)));
            }
            first = p;
            last = p;
        }
        PathEl::LineTo(p) => {
            edges.push((rot(last), rot(p)));
            last = p;
        }
        PathEl::ClosePath => {
            if last != first {
                edges.push((rot(last), rot(first)));
            }
            last = first;
        }
        _ => {}
    });
    if last != first {
        edges.push((rot(last), rot(first)));
    }

    if edges.is_empty() {
        return out;
    }

    let mut y0 = f64::INFINITY;
    let mut y1 = f64::NEG_INFINITY;
    for (a, c) in &edges {
        y0 = y0.min(a.y.min(c.y));
        y1 = y1.max(a.y.max(c.y));
    }

    if !y0.is_finite() || !y1.is_finite() || y1 <= y0 {
        return out;
    }

    // Safety checks on gap to prevent runaway loops
    let span = y1 - y0;
    let min_allowed_gap = span / (MAX_SCANLINES as f64);
    let gap = gap.max(min_allowed_gap).max(1e-4);

    let mut y = (y0 / gap).floor() * gap + gap * 0.5;
    let mut xs: Vec<f64> = Vec::new();
    let mut count = 0;

    while y < y1 && count < MAX_SCANLINES {
        count += 1;
        xs.clear();
        for (a, c) in &edges {
            if (a.y <= y) != (c.y <= y) && (c.y - a.y).abs() > 1e-12 {
                let t = (y - a.y) / (c.y - a.y);
                xs.push(a.x + t * (c.x - a.x));
            }
        }
        xs.sort_by(|u, v| u.total_cmp(v));

        let chunks = xs.chunks_exact(2);
        for pair in chunks {
            let p1 = unrot(Point::new(pair[0], y));
            let p2 = unrot(Point::new(pair[1], y));
            out.move_to(p1);
            out.line_to(p2);
        }
        y += gap;
    }

    out
}

/// Generate diagonal hatch lines with angle specified in degrees.
pub fn hatch_deg(path: &BezPath, gap: f64, angle_deg: f64) -> BezPath {
    hatch(path, gap, angle_deg.to_radians())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Rect;

    #[test]
    fn hatch_rectangle_deg45() {
        let r = Rect::new(0.0, 0.0, 40.0, 30.0);
        let bp = r.to_path(0.1);
        let h = hatch_deg(&bp, HATCH_GAP_PT, 45.0);
        let b = h.bounding_box();
        assert!(b.x0 >= -1e-6 && b.y0 >= -1e-6);
        assert!(b.x1 <= 40.0 + 1e-6 && b.y1 <= 30.0 + 1e-6);
        assert!(h.elements().len() >= 10);
    }

    #[test]
    fn hatch_rectangle_deg135() {
        let r = Rect::new(10.0, 10.0, 50.0, 40.0);
        let bp = r.to_path(0.1);
        let h = hatch_deg(&bp, HATCH_GAP_PT, 135.0);
        let b = h.bounding_box();
        assert!(b.x0 >= 10.0 - 1e-6 && b.y0 >= 10.0 - 1e-6);
        assert!(b.x1 <= 50.0 + 1e-6 && b.y1 <= 40.0 + 1e-6);
        assert!(h.elements().len() >= 10);
    }

    #[test]
    fn hatch_degenerate_path_returns_empty() {
        let mut bp = BezPath::new();
        bp.move_to(Point::new(10.0, 10.0));
        bp.line_to(Point::new(10.0, 10.0));
        let h = hatch_deg(&bp, 2.0, 45.0);
        assert!(h.is_empty());
    }
}
