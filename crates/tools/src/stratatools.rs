//! Archaeological CAD Section and Soil Stratum Tools (Trowel, J010).
//!
//! Interactive tools:
//! - [`StratumLineTool`]: Draws stratigraphic boundary lines (`stratum-line`, `Shift+S`).
//! - [`StratumBadgeTool`]: Places circled stratum number badges (`stratum-badge`, `Shift+E`).
//! - [`HearthCutTool`]: Draws burnt soil / hearth pit cross-sections (`hearth-cut`).

use serde_json::json;
use vectorcraft_geom::Point;

use crate::{Action, Cursor, Overlay, PointerEvent, PointerKind, Tool, ToolContext};

/// Stratum boundary line tool (`stratum-line`, `Shift+S`).
#[derive(Clone, Debug, Default)]
pub struct StratumLineTool {
    points: Vec<Point>,
    current: Point,
}

impl StratumLineTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for StratumLineTool {
    fn id(&self) -> &'static str {
        "stratum-line"
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
                if self.points.len() >= 2 {
                    let anchors: Vec<[f64; 2]> = self.points.iter().map(|p| [p.x, p.y]).collect();
                    out.push(Action::Exec(
                        "drawing.addStratumLine".to_string(),
                        json!({
                            "anchors": anchors,
                            "closed": false,
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
                    ovls.push(Overlay::Line { a, b, color: [20, 20, 19], dashed: false });
                }
            }
        } else if let Some(&start) = self.points.first() {
            ovls.push(Overlay::Line { a: start, b: self.current, color: [20, 20, 19], dashed: true });
        }
        ovls
    }
}

/// Stratum badge marker tool (`stratum-badge`, `Shift+E`).
#[derive(Clone, Debug)]
pub struct StratumBadgeTool {
    next_number: u32,
    preview: Option<Point>,
}

impl Default for StratumBadgeTool {
    fn default() -> Self {
        Self { next_number: 1, preview: None }
    }
}

impl StratumBadgeTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for StratumBadgeTool {
    fn id(&self) -> &'static str {
        "stratum-badge"
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: crate::Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        let mut out = Vec::new();
        match ev.kind {
            PointerKind::Move => {
                self.preview = Some(ev.pos);
            }
            PointerKind::Up => {
                let num = self.next_number;
                out.push(Action::Exec(
                    "drawing.addStratumBadge".to_string(),
                    json!({
                        "x": ev.pos.x,
                        "y": ev.pos.y,
                        "number": num,
                    }),
                ));
                self.next_number += 1;
            }
            _ => {}
        }
        out
    }

    fn overlays(&self, _cx: &ToolContext) -> Vec<Overlay> {
        if let Some(pos) = self.preview {
            let sym = if (1..=20).contains(&self.next_number) {
                char::from_u32(0x2460 + (self.next_number - 1)).unwrap_or('①').to_string()
            } else {
                self.next_number.to_string()
            };
            vec![
                Overlay::Handle { p: pos, color: [20, 20, 19] },
                Overlay::Label { p: Point::new(pos.x + 12.0, pos.y), text: format!("Stratum {sym}"), color: [20, 20, 19] },
            ]
        } else {
            vec![]
        }
    }
}

/// Hearth / burnt soil cut cross section tool (`hearth-cut`).
#[derive(Clone, Debug, Default)]
pub struct HearthCutTool {
    start: Option<Point>,
    current: Point,
}

impl HearthCutTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for HearthCutTool {
    fn id(&self) -> &'static str {
        "hearth-cut"
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
                    let dx = ev.pos.x - start.x;
                    let dy = ev.pos.y - start.y;
                    let w = dx.abs().max(20.0);
                    let h = dy.abs().max(10.0);
                    let p1 = start;
                    let p2 = Point::new(start.x + w * 0.5, start.y + h);
                    let p3 = Point::new(start.x + w, start.y);

                    out.push(Action::Exec(
                        "drawing.addHearthCut".to_string(),
                        json!({
                            "anchors": [
                                [p1.x, p1.y],
                                [p2.x, p2.y],
                                [p3.x, p3.y]
                            ],
                            "label": "소토면"
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
            vec![
                Overlay::Line { a: start, b: self.current, color: [196, 102, 134], dashed: true },
                Overlay::Label { p: self.current, text: "소토면".into(), color: [196, 102, 134] },
            ]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stratum_line_tool_actions() {
        let mut tool = StratumLineTool::new();
        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p1 = PointerEvent::new(PointerKind::Down, 10.0, 10.0);
        let _ = tool.pointer(&cx, &p1);
        let p2 = PointerEvent::new(PointerKind::Drag, 50.0, 20.0);
        let _ = tool.pointer(&cx, &p2);
        let p3 = PointerEvent::new(PointerKind::Up, 100.0, 15.0);
        let acts = tool.pointer(&cx, &p3);

        assert_eq!(acts.len(), 1);
        match &acts[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "drawing.addStratumLine");
                assert!(params["anchors"].as_array().unwrap().len() >= 2);
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }

    #[test]
    fn test_stratum_badge_tool_action() {
        let mut tool = StratumBadgeTool::new();
        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p1 = PointerEvent::new(PointerKind::Up, 150.0, 120.0);
        let acts = tool.pointer(&cx, &p1);

        assert_eq!(acts.len(), 1);
        match &acts[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "drawing.addStratumBadge");
                assert_eq!(params["number"], 1);
                assert_eq!(params["x"], 150.0);
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }
}
