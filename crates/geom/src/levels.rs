//! Level line clipping and interval subtraction against obstacle polygons (J015).
//!
//! Generates horizontal level line intervals outside obstacle regions (e.g. soil/strata/cut faces/stones).

use kurbo::{BezPath, PathEl, Point, Shape};

/// Given a horizontal span `(x0, x1)` at height `y`, subtracts the interior of all closed
/// obstacle paths and returns the remaining empty intervals `Vec<(f64, f64)>`.
///
/// Intervals with length < `min_len` (e.g. 0.5 pt) are pruned to avoid sub-pixel fragments.
pub fn subtract_horizontal_obstacles(y: f64, span: (f64, f64), obstacles: &[BezPath], min_len: f64) -> Vec<(f64, f64)> {
    if !y.is_finite() || !span.0.is_finite() || !span.1.is_finite() || span.1 <= span.0 {
        return Vec::new();
    }

    // 1. Collect all obstacle intersection intervals at y.
    let mut obstacle_intervals: Vec<(f64, f64)> = Vec::new();

    for path in obstacles {
        let b = path.bounding_box();
        // Quick AABB rejection
        if y < b.y0 || y > b.y1 {
            continue;
        }

        // Flatten path into line edges
        let mut edges: Vec<(Point, Point)> = Vec::new();
        let mut first = Point::ZERO;
        let mut last = Point::ZERO;
        let tol = 0.1;
        kurbo::flatten(path.iter(), tol, |el| match el {
            PathEl::MoveTo(p) => {
                if last != first {
                    edges.push((last, first));
                }
                first = p;
                last = p;
            }
            PathEl::LineTo(p) => {
                edges.push((last, p));
                last = p;
            }
            PathEl::ClosePath => {
                if last != first {
                    edges.push((last, first));
                }
                last = first;
            }
            _ => {}
        });
        if last != first {
            edges.push((last, first));
        }

        let mut xs: Vec<f64> = Vec::new();
        for (a, c) in &edges {
            if (a.y <= y) != (c.y <= y) && (c.y - a.y).abs() > 1e-12 {
                let t = (y - a.y) / (c.y - a.y);
                xs.push(a.x + t * (c.x - a.x));
            }
        }
        xs.sort_by(|u, v| u.total_cmp(v));

        for pair in xs.chunks_exact(2) {
            let left = pair[0].min(pair[1]);
            let right = pair[0].max(pair[1]);
            if right > left {
                obstacle_intervals.push((left, right));
            }
        }
    }

    // 2. Merge overlapping or adjacent obstacle intervals.
    if obstacle_intervals.is_empty() {
        return vec![span];
    }

    obstacle_intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::with_capacity(obstacle_intervals.len());
    for (start, end) in obstacle_intervals {
        if let Some(last) = merged.last_mut() {
            if start <= last.1 + 1e-6 {
                last.1 = last.1.max(end);
            } else {
                merged.push((start, end));
            }
        } else {
            merged.push((start, end));
        }
    }

    // 3. Subtract merged obstacle intervals from `span = (x0, x1)`.
    let mut out: Vec<(f64, f64)> = Vec::new();
    let (mut curr_x, span_end) = span;

    for (obs_start, obs_end) in merged {
        if obs_end <= curr_x {
            continue;
        }
        if obs_start >= span_end {
            break;
        }
        if obs_start > curr_x {
            let seg_end = obs_start.min(span_end);
            if seg_end - curr_x >= min_len {
                out.push((curr_x, seg_end));
            }
        }
        curr_x = curr_x.max(obs_end);
    }

    if span_end - curr_x >= min_len {
        out.push((curr_x, span_end));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Rect;

    #[test]
    fn test_subtract_no_obstacles() {
        let span = (10.0, 100.0);
        let res = subtract_horizontal_obstacles(50.0, span, &[], 0.5);
        assert_eq!(res, vec![(10.0, 100.0)]);
    }

    #[test]
    fn test_subtract_single_obstacle_in_middle() {
        let span = (0.0, 100.0);
        let rect = Rect::new(30.0, 20.0, 70.0, 80.0).to_path(0.1);
        let res = subtract_horizontal_obstacles(50.0, span, &[rect], 0.5);
        assert_eq!(res.len(), 2);
        assert!((res[0].0 - 0.0).abs() < 1e-4);
        assert!((res[0].1 - 30.0).abs() < 1e-4);
        assert!((res[1].0 - 70.0).abs() < 1e-4);
        assert!((res[1].1 - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_subtract_obstacle_outside_y_range() {
        let span = (0.0, 100.0);
        let rect = Rect::new(30.0, 20.0, 70.0, 40.0).to_path(0.1);
        let res = subtract_horizontal_obstacles(50.0, span, &[rect], 0.5);
        assert_eq!(res, vec![(0.0, 100.0)]);
    }

    #[test]
    fn test_subtract_overlapping_obstacles() {
        let span = (0.0, 100.0);
        let r1 = Rect::new(20.0, 10.0, 50.0, 60.0).to_path(0.1);
        let r2 = Rect::new(40.0, 10.0, 80.0, 60.0).to_path(0.1);
        let res = subtract_horizontal_obstacles(30.0, span, &[r1, r2], 0.5);
        assert_eq!(res.len(), 2);
        assert!((res[0].0 - 0.0).abs() < 1e-4);
        assert!((res[0].1 - 20.0).abs() < 1e-4);
        assert!((res[1].0 - 80.0).abs() < 1e-4);
        assert!((res[1].1 - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_subtract_completely_covered() {
        let span = (20.0, 80.0);
        let rect = Rect::new(10.0, 0.0, 90.0, 100.0).to_path(0.1);
        let res = subtract_horizontal_obstacles(50.0, span, &[rect], 0.5);
        assert!(res.is_empty());
    }
}
