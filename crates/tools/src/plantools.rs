//! Archaeological CAD Plan Tools (Trowel, J009).
//!
//! Interactive tools:
//! - [`SectionLineTool`]: Draws section lines (Shift+A) with cut ticks, sight arrows and labels.
//! - [`SpotHeightTool`]: Places elevation spot heights (Shift+Z).
//! - [`FindNumberTool`]: Places circled artifact find numbers (1, 2, 3...).
//! - [`PostholeTool`]: Draws postholes with radius and P1, P2... depth annotation.
//! - [`SlopeHachureTool`]: Draws slope/embankment hachure ticks along an edge.

use serde_json::json;
use vectorcraft_geom::Point;

use crate::{Action, Cursor, Overlay, PointerEvent, PointerKind, Tool, ToolContext};

/// Section line tool (`section-line`, `Shift+A`).
#[derive(Clone, Debug)]
pub struct SectionLineTool {
    start: Option<Point>,
    current: Point,
    label_idx: usize,
    flip: bool,
}

impl Default for SectionLineTool {
    fn default() -> Self {
        Self { start: None, current: Point::ZERO, label_idx: 0, flip: false }
    }
}

impl SectionLineTool {
    pub fn new() -> Self {
        Self::default()
    }

    fn current_label(&self) -> String {
        let b = b'A' + (self.label_idx % 26) as u8;
        let suffix = if self.label_idx >= 26 { (self.label_idx / 26 + 1).to_string() } else { String::new() };
        format!("{}{}", b as char, suffix)
    }
}

impl Tool for SectionLineTool {
    fn id(&self) -> &'static str {
        "section-line"
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
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist >= 6.0 {
                        let label = self.current_label();
                        out.push(Action::Exec(
                            "drawing.addSectionLine".to_string(),
                            json!({
                                "x1": start.x,
                                "y1": start.y,
                                "x2": ev.pos.x,
                                "y2": ev.pos.y,
                                "label": label,
                                "flip": ev.mods.alt || self.flip,
                            }),
                        ));
                        self.label_idx += 1;
                    }
                }
            }
            _ => {}
        }
        out
    }

    fn overlays(&self, _cx: &ToolContext) -> Vec<Overlay> {
        if let Some(start) = self.start {
            vec![
                Overlay::Line { a: start, b: self.current, color: [20, 20, 19], dashed: true },
                Overlay::Label { p: self.current, text: format!("Section {}", self.current_label()), color: [20, 20, 19] },
            ]
        } else {
            vec![]
        }
    }
}

/// Elevation spot height tool (`spot-height`, `Shift+Z`).
#[derive(Clone, Debug)]
pub struct SpotHeightTool {
    pub elevation: f64,
}

impl Default for SpotHeightTool {
    fn default() -> Self {
        Self { elevation: 152.0 }
    }
}

impl SpotHeightTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for SpotHeightTool {
    fn id(&self) -> &'static str {
        "spot-height"
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: crate::Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        let mut out = Vec::new();
        if ev.kind == PointerKind::Up {
            out.push(Action::Exec(
                "drawing.addSpotHeight".to_string(),
                json!({
                    "x": ev.pos.x,
                    "y": ev.pos.y,
                    "elevation": self.elevation,
                }),
            ));
        }
        out
    }
}

/// Circled artifact find number tool (`find-number`).
#[derive(Clone, Debug)]
pub struct FindNumberTool {
    pub next_number: u32,
}

impl Default for FindNumberTool {
    fn default() -> Self {
        Self { next_number: 1 }
    }
}

impl FindNumberTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for FindNumberTool {
    fn id(&self) -> &'static str {
        "find-number"
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: crate::Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        let mut out = Vec::new();
        if ev.kind == PointerKind::Up {
            out.push(Action::Exec(
                "drawing.addFindNumber".to_string(),
                json!({
                    "x": ev.pos.x,
                    "y": ev.pos.y,
                    "number": self.next_number,
                }),
            ));
            self.next_number += 1;
        }
        out
    }
}

/// Posthole tool (`posthole`).
#[derive(Clone, Debug)]
pub struct PostholeTool {
    start: Option<Point>,
    radius: f64,
    pub next_number: u32,
}

impl Default for PostholeTool {
    fn default() -> Self {
        Self { start: None, radius: 13.0, next_number: 1 }
    }
}

impl PostholeTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for PostholeTool {
    fn id(&self) -> &'static str {
        "posthole"
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: crate::Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        let mut out = Vec::new();
        match ev.kind {
            PointerKind::Down => {
                self.start = Some(ev.pos);
                self.radius = 13.0;
            }
            PointerKind::Drag => {
                if let Some(start) = self.start {
                    let dx = ev.pos.x - start.x;
                    let dy = ev.pos.y - start.y;
                    self.radius = (dx * dx + dy * dy).sqrt().max(4.0);
                }
            }
            PointerKind::Up => {
                if let Some(start) = self.start.take() {
                    out.push(Action::Exec(
                        "drawing.addPosthole".to_string(),
                        json!({
                            "x": start.x,
                            "y": start.y,
                            "radius": self.radius,
                            "number": self.next_number,
                        }),
                    ));
                    self.next_number += 1;
                    self.radius = 13.0;
                }
            }
            _ => {}
        }
        out
    }
}

/// Slope hachure ticks tool (`hachure`).
#[derive(Clone, Debug, Default)]
pub struct SlopeHachureTool {
    points: Vec<Point>,
}

impl SlopeHachureTool {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Tool for SlopeHachureTool {
    fn id(&self) -> &'static str {
        "hachure"
    }

    fn cursor(&self, _cx: &ToolContext, _p: Point, _mods: crate::Mods) -> Cursor {
        Cursor::Crosshair
    }

    fn pointer(&mut self, _cx: &ToolContext, ev: &PointerEvent) -> Vec<Action> {
        let mut out = Vec::new();
        match ev.kind {
            PointerKind::Down => {
                self.points.clear();
                self.points.push(ev.pos);
            }
            PointerKind::Drag => {
                if let Some(last) = self.points.last() {
                    let dx = ev.pos.x - last.x;
                    let dy = ev.pos.y - last.y;
                    if (dx * dx + dy * dy).sqrt() >= 8.0 {
                        self.points.push(ev.pos);
                    }
                }
            }
            PointerKind::Up => {
                self.points.push(ev.pos);
                if self.points.len() >= 2 {
                    let pts: Vec<[f64; 2]> = self.points.iter().map(|p| [p.x, p.y]).collect();
                    out.push(Action::Exec(
                        "drawing.addSlopeTicks".to_string(),
                        json!({
                            "points": pts,
                            "flip": ev.mods.alt,
                        }),
                    ));
                }
                self.points.clear();
            }
            _ => {}
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_line_tool_actions() {
        let mut tool = SectionLineTool::new();
        assert_eq!(tool.id(), "section-line");

        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p1 = PointerEvent::new(PointerKind::Down, 100.0, 100.0);
        let actions = tool.pointer(&cx, &p1);
        assert!(actions.is_empty());

        let p2 = PointerEvent::new(PointerKind::Up, 300.0, 100.0);
        let actions = tool.pointer(&cx, &p2);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "drawing.addSectionLine");
                assert_eq!(params["label"], "A");
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }

    #[test]
    fn test_spot_height_tool_action() {
        let mut tool = SpotHeightTool::new();
        assert_eq!(tool.id(), "spot-height");

        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p = PointerEvent::new(PointerKind::Up, 150.0, 200.0);
        let actions = tool.pointer(&cx, &p);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "drawing.addSpotHeight");
                assert_eq!(params["x"], 150.0);
                assert_eq!(params["y"], 200.0);
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }

    #[test]
    fn test_find_number_tool_action() {
        let mut tool = FindNumberTool::new();
        assert_eq!(tool.id(), "find-number");

        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p = PointerEvent::new(PointerKind::Up, 120.0, 180.0);
        let actions = tool.pointer(&cx, &p);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "drawing.addFindNumber");
                assert_eq!(params["x"], 120.0);
                assert_eq!(params["y"], 180.0);
                assert_eq!(params["number"], 1);
            }
            other => panic!("unexpected action: {:?}", other),
        }
        assert_eq!(tool.next_number, 2);
    }

    #[test]
    fn test_posthole_tool_action() {
        let mut tool = PostholeTool::new();
        assert_eq!(tool.id(), "posthole");

        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p1 = PointerEvent::new(PointerKind::Down, 200.0, 200.0);
        assert!(tool.pointer(&cx, &p1).is_empty());

        let p2 = PointerEvent::new(PointerKind::Drag, 220.0, 200.0);
        assert!(tool.pointer(&cx, &p2).is_empty());
        assert!((tool.radius - 20.0).abs() < 1e-4);

        let p3 = PointerEvent::new(PointerKind::Up, 220.0, 200.0);
        let actions = tool.pointer(&cx, &p3);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "drawing.addPosthole");
                assert_eq!(params["x"], 200.0);
                assert_eq!(params["y"], 200.0);
                assert!((params["radius"].as_f64().unwrap() - 20.0).abs() < 1e-4);
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }

    #[test]
    fn test_slope_hachure_tool_action() {
        let mut tool = SlopeHachureTool::new();
        assert_eq!(tool.id(), "hachure");

        let (d, _id) = crate::testutil::doc_with_rect();
        let sel = vectorcraft_doc::Selection::default();
        let paint = crate::PaintDefaults::default();
        let cx = crate::testutil::cx(&d, &sel, &paint);

        let p1 = PointerEvent::new(PointerKind::Down, 100.0, 100.0);
        assert!(tool.pointer(&cx, &p1).is_empty());

        let p2 = PointerEvent::new(PointerKind::Drag, 150.0, 100.0);
        assert!(tool.pointer(&cx, &p2).is_empty());

        let p3 = PointerEvent::new(PointerKind::Up, 200.0, 100.0);
        let actions = tool.pointer(&cx, &p3);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            Action::Exec(cmd, params) => {
                assert_eq!(cmd, "drawing.addSlopeTicks");
                let pts = params["points"].as_array().unwrap();
                assert!(pts.len() >= 2);
            }
            other => panic!("unexpected action: {:?}", other),
        }
    }
}
