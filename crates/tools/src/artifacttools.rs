//! Archaeological CAD Artifact Drafting Tools (Trowel, J011).
//!
//! Interactive tools:
//! - [`RimArcTool`]: 3-point rim or base arc diameter fitting tool (`rim-arc`).
//! - [`CenterlineTool`]: Centerline (℄) placement and vertical symmetry guide (`centerline`).
//! - [`WallCutTool`]: Vessel wall cross-section drafting tool (`wall-cut`).

use serde_json::json;
use vectorcraft_geom::Point;

use crate::{Action, Cursor, Overlay, PointerEvent, PointerKind, Tool, ToolContext};

/// 3-Point Rim or Base Arc Fitting Tool (`rim-arc`).
#[derive(Clone, Debug)]
pub struct RimArcTool {
    points: Vec<Point>,
    current: Point,
    kind: &'static str,
}

impl Default for RimArcTool {
    fn default() -> Self {
        Self { points: Vec::new(), current: Point::new(0.0, 0.0), kind: "rim" }
    }
}

impl RimArcTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_kind(kind: &'static str) -> Self {
        Self { points: Vec::new(), current: Point::new(0.0, 0.0), kind }
    }
}

impl Tool for RimArcTool {
    fn id(&self) -> &'static str {
        "rim-arc"
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: crate::Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        let mut out = Vec::new();
        match ev.kind {
            PointerKind::Move | PointerKind::Drag => {
                self.current = ev.pos;
            }
            PointerKind::Up => {
                self.current = ev.pos;
                self.points.push(ev.pos);
                if self.points.len() >= 3 {
                    let pts: Vec<[f64; 2]> = self.points.iter().take(3).map(|p| [p.x, p.y]).collect();
                    out.push(Action::Exec(
                        "artifact.fitRimArc".to_string(),
                        json!({
                            "points": pts,
                            "kind": self.kind,
                        }),
                    ));
                    self.points.clear();
                }
            }
            _ => {}
        }
        out
    }

    fn overlays(&self, _cx: &ToolContext) -> Vec<Overlay> {
        let mut ovls = Vec::new();
        for &p in &self.points {
            ovls.push(Overlay::Handle { p, color: [59, 130, 246] });
        }
        if let Some(&first) = self.points.first() {
            if self.points.len() == 1 {
                ovls.push(Overlay::Line { a: first, b: self.current, color: [59, 130, 246], dashed: true });
            } else if let Some(&second) = self.points.get(1) {
                ovls.push(Overlay::Line { a: first, b: second, color: [59, 130, 246], dashed: true });
                ovls.push(Overlay::Line { a: second, b: self.current, color: [59, 130, 246], dashed: true });
            }
        }
        ovls
    }
}

/// Artifact Centerline (℄) tool (`centerline`).
#[derive(Clone, Debug, Default)]
pub struct CenterlineTool {
    start: Option<Point>,
    current: Point,
}

impl CenterlineTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for CenterlineTool {
    fn id(&self) -> &'static str {
        "centerline"
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: crate::Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        let mut out = Vec::new();
        match ev.kind {
            PointerKind::Down => {
                self.start = Some(ev.pos);
                self.current = ev.pos;
            }
            PointerKind::Drag => {
                self.current = ev.pos;
            }
            PointerKind::Up => {
                if let Some(start) = self.start.take() {
                    let dy = (ev.pos.y - start.y).abs();
                    let (top_y, bot_y, x) = if dy > 15.0 {
                        let y1 = start.y.min(ev.pos.y);
                        let y2 = start.y.max(ev.pos.y);
                        let mid_x = (start.x + ev.pos.x) * 0.5;
                        (y1, y2, mid_x)
                    } else {
                        // Single click fallback: create vertical centerline of default 300 pt span
                        (start.y - 150.0, start.y + 150.0, start.x)
                    };

                    out.push(Action::Exec(
                        "artifact.setCenterline".to_string(),
                        json!({
                            "x": x,
                            "y1": top_y,
                            "y2": bot_y,
                        }),
                    ));
                }
            }
            _ => {}
        }
        out
    }

    fn overlays(&self, _cx: &ToolContext) -> Vec<Overlay> {
        if let Some(start) = self.start {
            let x = (start.x + self.current.x) * 0.5;
            let top_y = start.y.min(self.current.y);
            let bot_y = start.y.max(self.current.y);
            vec![
                Overlay::Line { a: Point::new(x, top_y), b: Point::new(x, bot_y), color: [30, 41, 59], dashed: true },
                Overlay::Label { p: Point::new(x - 4.0, top_y - 6.0), text: "℄".into(), color: [30, 41, 59] },
            ]
        } else {
            vec![]
        }
    }
}

/// Vessel Wall Cut cross-section tool (`wall-cut`).
#[derive(Clone, Debug, Default)]
pub struct WallCutTool {
    points: Vec<Point>,
    current: Point,
}

impl WallCutTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for WallCutTool {
    fn id(&self) -> &'static str {
        "wall-cut"
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: crate::Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        let mut out = Vec::new();
        match ev.kind {
            PointerKind::Down => {
                if self.points.is_empty() {
                    self.points.push(ev.pos);
                } else if let Some(last) = self.points.last() {
                    let d = (ev.pos.x - last.x).hypot(ev.pos.y - last.y);
                    if d > 2.0 {
                        self.points.push(ev.pos);
                    }
                }
                self.current = ev.pos;
            }
            PointerKind::Drag => {
                self.current = ev.pos;
                if let Some(last) = self.points.last() {
                    let d = (ev.pos.x - last.x).hypot(ev.pos.y - last.y);
                    if d > 6.0 {
                        self.points.push(ev.pos);
                    }
                }
            }
            PointerKind::Up => {
                self.current = ev.pos;
                if self.points.len() >= 3 {
                    let pts: Vec<[f64; 2]> = self.points.iter().map(|p| [p.x, p.y]).collect();
                    out.push(Action::Exec(
                        "artifact.addWallCut".to_string(),
                        json!({
                            "points": pts,
                        }),
                    ));
                    self.points.clear();
                }
            }
            _ => {}
        }
        out
    }

    fn overlays(&self, _cx: &ToolContext) -> Vec<Overlay> {
        let mut ovls = Vec::new();
        if self.points.len() >= 2 {
            for i in 0..(self.points.len() - 1) {
                if let (Some(&a), Some(&b)) = (self.points.get(i), self.points.get(i + 1)) {
                    ovls.push(Overlay::Line { a, b, color: [196, 102, 134], dashed: false });
                }
            }
        }
        if let Some(&last) = self.points.last() {
            ovls.push(Overlay::Line { a: last, b: self.current, color: [196, 102, 134], dashed: true });
        }
        ovls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rim_arc_tool_actions() {
        let mut tool = RimArcTool::new();
        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p1 = PointerEvent::new(PointerKind::Up, 100.0, 200.0);
        assert!(tool.pointer(&cx, &p1).is_empty());

        let p2 = PointerEvent::new(PointerKind::Up, 200.0, 100.0);
        assert!(tool.pointer(&cx, &p2).is_empty());

        let p3 = PointerEvent::new(PointerKind::Up, 300.0, 200.0);
        let acts = tool.pointer(&cx, &p3);
        assert_eq!(acts.len(), 1);
        match &acts[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "artifact.fitRimArc");
                assert_eq!(params["points"].as_array().unwrap().len(), 3);
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }

    #[test]
    fn test_centerline_tool_actions() {
        let mut tool = CenterlineTool::new();
        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p1 = PointerEvent::new(PointerKind::Down, 300.0, 50.0);
        let _ = tool.pointer(&cx, &p1);
        let p2 = PointerEvent::new(PointerKind::Drag, 300.0, 250.0);
        let _ = tool.pointer(&cx, &p2);
        let p3 = PointerEvent::new(PointerKind::Up, 300.0, 450.0);
        let acts = tool.pointer(&cx, &p3);

        assert_eq!(acts.len(), 1);
        match &acts[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "artifact.setCenterline");
                assert_eq!(params["x"], 300.0);
                assert_eq!(params["y1"], 50.0);
                assert_eq!(params["y2"], 450.0);
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }

    #[test]
    fn test_wall_cut_tool_actions() {
        let mut tool = WallCutTool::new();
        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p1 = PointerEvent::new(PointerKind::Down, 350.0, 100.0);
        let _ = tool.pointer(&cx, &p1);
        let p2 = PointerEvent::new(PointerKind::Down, 380.0, 200.0);
        let _ = tool.pointer(&cx, &p2);
        let p3 = PointerEvent::new(PointerKind::Down, 340.0, 300.0);
        let _ = tool.pointer(&cx, &p3);
        let p4 = PointerEvent::new(PointerKind::Up, 340.0, 300.0);
        let acts = tool.pointer(&cx, &p4);

        assert_eq!(acts.len(), 1);
        match &acts[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "artifact.addWallCut");
                assert_eq!(params["points"].as_array().unwrap().len(), 3);
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }
}
