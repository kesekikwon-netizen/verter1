//! Trowel's window drawn and clicked in headless egui frames: 1600 × 988 like the mockup
//! (`win-plan`, design 2.1 §3) unless a test says otherwise, with the app's own title bar and one
//! document open.

use egui::{Event, PointerButton, Pos2, Rect, pos2, vec2};
use serde_json::json;
use vectorcraft_doc::NodeId;
use vectorcraft_engine::Session;

use super::*;
use crate::VectorcraftApp;
use crate::theme::Tokens;

/// The ribbon's tiles, left to right.
const TILES: [&str; 23] = [
    "open",
    "save",
    "import",
    "plan",
    "section",
    "finds",
    "sheet",
    "align",
    "underlay",
    "pen",
    "pencil",
    "section-line",
    "spot-height",
    "bucket",
    "north",
    "scale-bar",
    "legend",
    "title-block",
    "grid",
    "magnet",
    "pdf",
    "dxf",
    "strata",
];

/// The tool column's slots, top to bottom, by their first tool.
const SLOTS: [&str; 14] = [
    "selection",
    "directSelection",
    "pen",
    "pencil",
    "lineSegment",
    "rectangle",
    "type",
    "rotate",
    "scale",
    "scissors",
    "livePaintBucket",
    "eyedropper",
    "hand",
    "zoom",
];

/// The archaeology tools under them.
const ARCH: [&str; 5] = ["section-line", "spot-height", "find-number", "posthole", "hachure"];

/// Text painted in the last frame: what it says, where and in what colour, and the part of it
/// that shows (inside its clip).
struct Painted {
    text: String,
    rect: Rect,
    color: egui::Color32,
    font: egui::FontId,
    shown: Rect,
}

struct Harness {
    app: VectorcraftApp,
    ctx: egui::Context,
    screen: egui::Vec2,
    time: f64,
    texts: Vec<Painted>,
    shapes: Vec<egui::epaint::ClippedShape>,
}

impl Harness {
    /// The first start (Trowel's window) with a document named "Site 1" open, 1600 × 988.
    fn new() -> Self {
        Self::sized(1600.0, 988.0)
    }

    /// The same in a `w` × `h` window.
    fn sized(w: f32, h: f32) -> Self {
        let mut app = VectorcraftApp::new(Session::new(), crate::Services::default());
        app.custom_titlebar = true;
        let lang = app.ui_language();
        crate::workspaces::start_trowel(&mut app.ui, lang);
        app.run("file.new", json!({"name": "Site 1", "width": 400, "height": 300})).unwrap();
        let mut h = Harness { app, ctx: egui::Context::default(), screen: vec2(w, h), time: 0.0, texts: vec![], shapes: vec![] };
        // The first frame installs the fonts, the second lays the window out, the third sees it.
        for _ in 0..3 {
            h.frame(vec![]);
        }
        h
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.time += 0.05;
        let raw =
            egui::RawInput { time: Some(self.time), screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.screen)), events, ..Default::default() };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(raw, |ui| {
            app.logic(ui.ctx());
            app.ui(ui);
        });
        out.textures_delta.clear();
        self.texts = out
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) => {
                    let format = t.galley.job.sections.first().map(|s| &s.format);
                    let color = format.map(|f| f.color).filter(|c| *c != egui::Color32::PLACEHOLDER);
                    let rect = t.visual_bounding_rect();
                    Some(Painted {
                        text: t.galley.text().to_string(),
                        rect,
                        color: color.unwrap_or(t.fallback_color),
                        font: format.map(|f| f.font_id.clone()).unwrap_or_default(),
                        shown: rect.intersect(c.clip_rect),
                    })
                }
                _ => None,
            })
            .collect();
        self.shapes = out.shapes;
    }

    /// A few frames with nothing new, so what the last one changed is laid out and seen.
    fn settle(&mut self) {
        for _ in 0..2 {
            self.frame(vec![]);
        }
    }

    /// Where the widget `id` was in the last frame.
    fn rect(&self, id: egui::Id) -> Option<Rect> {
        self.ctx.viewport(|vp| vp.prev_pass.widgets.get(id).map(|w| w.rect))
    }

    fn panel(&self, id: &str) -> Rect {
        egui::PanelState::load(&self.ctx, egui::Id::new(id)).unwrap_or_else(|| panic!("no panel {id}")).outer_rect
    }

    fn press(&mut self, at: Pos2, button: PointerButton) {
        self.frame(vec![Event::PointerMoved(at)]);
        self.frame(vec![Event::PointerButton { pos: at, button, pressed: true, modifiers: Default::default() }]);
        self.frame(vec![Event::PointerButton { pos: at, button, pressed: false, modifiers: Default::default() }]);
        self.settle();
    }

    fn click_at(&mut self, at: Pos2) {
        self.press(at, PointerButton::Primary);
    }

    fn click(&mut self, id: egui::Id) {
        let r = self.rect(id).unwrap_or_else(|| panic!("no widget {id:?}"));
        self.click_at(r.center());
    }

    /// Let `secs` seconds pass, then draw a frame.
    fn later(&mut self, secs: f64) {
        self.time += secs;
        self.settle();
    }

    /// Turn the mouse wheel over `at` by `delta` (the way the content moves), and let the smooth
    /// scrolling run its course.
    fn wheel(&mut self, at: Pos2, delta: egui::Vec2) {
        self.frame(vec![Event::PointerMoved(at)]);
        let modifiers = Default::default();
        self.frame(vec![Event::MouseWheel { unit: egui::MouseWheelUnit::Point, delta, phase: egui::TouchPhase::Move, modifiers }]);
        for _ in 0..40 {
            self.frame(vec![]);
        }
    }

    /// Two clicks in a row on the widget `id`.
    fn double_click(&mut self, id: egui::Id) {
        let at = self.rect(id).unwrap_or_else(|| panic!("no widget {id:?}")).center();
        self.frame(vec![Event::PointerMoved(at)]);
        for _ in 0..2 {
            for pressed in [true, false] {
                self.frame(vec![Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Default::default() }]);
            }
        }
        self.settle();
    }

    /// A touchpad's tap at `at`: the press and the release in one frame.
    fn tap(&mut self, at: Pos2) {
        let (button, modifiers) = (PointerButton::Primary, egui::Modifiers::default());
        self.frame(vec![
            Event::PointerMoved(at),
            Event::PointerButton { pos: at, button, pressed: true, modifiers },
            Event::PointerButton { pos: at, button, pressed: false, modifiers },
        ]);
    }

    /// Press and let go of `key`.
    fn key(&mut self, key: egui::Key) {
        for pressed in [true, false] {
            self.frame(vec![Event::Key { key, physical_key: None, pressed, repeat: false, modifiers: Default::default() }]);
        }
        self.settle();
    }

    /// Click the text field `id`, select what it holds and type `text` over it, then Enter.
    fn type_into(&mut self, id: egui::Id, text: &str) {
        self.type_over(id, text);
        self.key(egui::Key::Enter);
    }

    /// Click the text field `id`, select what it holds and type `text` over it; it keeps the
    /// keyboard.
    fn type_over(&mut self, id: egui::Id, text: &str) {
        self.click(id);
        let all = egui::Modifiers::COMMAND;
        self.frame(vec![Event::Key { key: egui::Key::A, physical_key: None, pressed: true, repeat: false, modifiers: all }]);
        self.frame(vec![Event::Text(text.to_string())]);
    }

    fn hover(&mut self, id: egui::Id) {
        let at = self.rect(id).unwrap_or_else(|| panic!("no widget {id:?}")).center();
        self.frame(vec![Event::PointerMoved(at)]);
        self.settle();
    }

    /// Rest the pointer on `at` long enough for its hover text to show.
    fn rest_at(&mut self, at: Pos2) {
        self.frame(vec![Event::PointerMoved(at)]);
        self.later(1.0);
    }

    /// Whether the widget `id` took presses in the last frame: not when it was in a disabled part.
    fn enabled(&self, id: egui::Id) -> Option<bool> {
        self.ctx.viewport(|vp| vp.prev_pass.widgets.get(id).map(|w| w.enabled))
    }

    /// The texts painted inside `area`, left to right.
    fn texts_in(&self, area: Rect) -> Vec<String> {
        let mut v: Vec<&Painted> = self.texts.iter().filter(|p| area.expand(0.5).contains_rect(p.rect)).collect();
        v.sort_by(|a, b| a.rect.left().total_cmp(&b.rect.left()));
        v.into_iter().map(|p| p.text.clone()).collect()
    }

    /// The colour `text` was painted in inside `area`.
    fn color_of(&self, text: &str, area: Rect) -> Option<egui::Color32> {
        self.texts.iter().find(|p| p.text == text && area.expand(0.5).contains_rect(p.rect)).map(|p| p.color)
    }

    /// Pairs of texts showing in `area` that are drawn over each other.
    fn overlapping_texts(&self, area: Rect) -> Vec<(String, String)> {
        let shown: Vec<&Painted> = self.texts.iter().filter(|p| p.shown.is_positive() && p.shown.intersects(area)).collect();
        let mut v = vec![];
        for (i, a) in shown.iter().enumerate() {
            for b in shown.iter().skip(i + 1) {
                let both = a.shown.intersect(b.shown);
                if both.width() > 1.0 && both.height() > 1.0 {
                    v.push((a.text.clone(), b.text.clone()));
                }
            }
        }
        v
    }

    /// Everything painted inside `area` as one string, left to right.
    fn line_in(&self, area: Rect) -> String {
        self.texts_in(area).join(" ")
    }

    /// The fill of the rectangle painted exactly at `r` (the last one, topmost).
    fn fill_at(&self, r: Rect) -> Option<egui::Color32> {
        self.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Rect(s) if (s.rect.min - r.min).length() < 0.6 && (s.rect.max - r.max).length() < 0.6 => Some(s.fill),
                _ => None,
            })
            .next_back()
    }

    /// The rectangles painted inside `area`: where, their fill and their edge.
    fn rects_in(&self, area: Rect) -> Vec<(Rect, egui::Color32, egui::Stroke)> {
        self.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Rect(s) if area.expand(0.5).contains_rect(s.rect) => Some((s.rect, s.fill, s.stroke)),
                _ => None,
            })
            .collect()
    }

    /// The straight line segments painted inside `area`.
    fn segments_in(&self, area: Rect) -> Vec<([Pos2; 2], egui::Stroke)> {
        self.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::LineSegment { points, stroke } if points.iter().all(|p| area.expand(0.5).contains(*p)) => Some((*points, *stroke)),
                _ => None,
            })
            .collect()
    }

    /// Where colour gradients (a scroll area's fading edges) were painted over `area`.
    fn gradients_over(&self, area: Rect) -> Vec<Rect> {
        self.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Mesh(m) if m.vertices.len() == 4 && m.vertices.iter().any(|v| v.color != m.vertices[0].color) => {
                    Some(m.calc_bounds()).filter(|b| b.intersects(area))
                }
                _ => None,
            })
            .collect()
    }

    /// The font `text` was painted in inside `area`.
    fn font_of(&self, text: &str, area: Rect) -> Option<egui::FontId> {
        self.texts.iter().find(|p| p.text == text && area.expand(0.5).contains_rect(p.rect)).map(|p| p.font.clone())
    }

    /// What the status bar says.
    fn status_line(&self) -> String {
        self.line_in(self.panel(STATUS))
    }

    fn tokens(&self) -> Tokens {
        Tokens::get(&self.ctx)
    }

    fn layer_ids(&self) -> Vec<NodeId> {
        self.app.session.active().unwrap().doc.layers.iter().map(|l| l.id).collect()
    }

    fn current_layer(&self) -> Option<NodeId> {
        self.app.session.active().unwrap().current_layer()
    }

    /// The enabled and disabled clickable widgets of the open popup, top to bottom.
    fn popup_buttons(&self) -> Vec<(Rect, bool)> {
        let mut v: Vec<(Rect, bool)> = self.ctx.viewport(|vp| {
            vp.prev_pass
                .widgets
                .layers()
                .filter(|(l, _)| l.order == egui::Order::Foreground)
                // Not the popup's own area, which takes clicks to come to the front.
                .flat_map(|(l, ws)| ws.iter().filter(move |w| w.id != l.id.with("move")))
                .filter(|w| w.sense.senses_click())
                .map(|w| (w.rect, w.enabled))
                .collect()
        });
        v.sort_by(|a, b| a.0.top().total_cmp(&b.0.top()));
        v
    }
}

fn close(a: Rect, b: Rect) -> bool {
    (a.min - b.min).length() <= 1.0 && (a.max - b.max).length() <= 1.0
}

fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_min_max(pos2(x0, y0), pos2(x1, y1))
}

#[test]
fn regions_match_the_mockup_at_1600_by_988() {
    let mut h = Harness::new();
    let want = [
        (CAPTION, r(0.0, 0.0, 1600.0, 32.0)),
        (MENU, r(0.0, 32.0, 1600.0, 60.0)),
        (RIBBON, r(0.0, 60.0, 1600.0, 156.0)),
        (CONTROL, r(0.0, 156.0, 1600.0, 196.0)),
        (STATUS, r(0.0, 956.0, 1600.0, 988.0)),
        (TOOLS, r(0.0, 196.0, 56.0, 956.0)),
        (SET, r(56.0, 196.0, 272.0, 956.0)),
        (RIGHT, r(1296.0, 196.0, 1600.0, 956.0)),
    ];
    for (id, rect) in want {
        assert!(close(h.panel(id), rect), "{id}: {:?}, the mockup {rect:?}", h.panel(id));
    }
    // The canvas region is 1024 × 728 under the 32-point tabs; the rulers take 16 points of it.
    let canvas = h.app.canvas_rect.unwrap();
    assert!(close(canvas, r(288.0, 244.0, 1296.0, 956.0)), "{canvas:?}");
    // Full screen with menu bar off: no caption, menu, ribbon or Control bar.
    h.app.ui.screen_mode = 2;
    h.settle();
    assert!(close(h.app.canvas_rect.unwrap(), r(288.0, 48.0, 1296.0, 956.0)), "{:?}", h.app.canvas_rect);
    // Full screen: the canvas alone.
    h.app.ui.screen_mode = 3;
    h.settle();
    assert!(close(h.app.canvas_rect.unwrap(), r(0.0, 0.0, 1600.0, 988.0)), "{:?}", h.app.canvas_rect);
}

#[test]
fn caption_shows_the_document_and_unsaved_mark() {
    let mut h = Harness::new();
    let caption = h.panel(CAPTION);
    assert!(h.texts_in(caption).contains(&"Site 1 - Trowel".to_string()), "{:?}", h.texts_in(caption));
    // Trowel's mark at the left, the three window buttons at the right.
    let mark = crate::brand::texture_id(&h.ctx);
    let marks = h
        .shapes
        .iter()
        .filter(|c| matches!(&c.shape, egui::Shape::Mesh(m) if m.texture_id == mark && caption.contains_rect(m.calc_bounds())))
        .count();
    assert_eq!(marks, 1, "the mark in the caption");
    let buttons = h.ctx.viewport(|vp| {
        vp.prev_pass
            .widgets
            .layers()
            .flat_map(|(_, w)| w.iter())
            .filter(|w| w.rect.top() == 0.0 && w.rect.width() == crate::titlebar::BUTTON_WIDTH)
            .count()
    });
    assert_eq!(buttons, 3, "minimize, maximize, close");
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    h.settle();
    assert!(h.texts_in(caption).contains(&"Site 1 * - Trowel".to_string()), "{:?}", h.texts_in(caption));
}

#[test]
fn drawing_menu_sits_before_view_and_waits_for_its_jobs() {
    let mut h = Harness::new();
    let bar = h.panel(MENU);
    let titles = h.texts_in(bar);
    assert_eq!(titles, ["File", "Edit", "Object", "Type", "Select", "Effect", "Drawing", "View", "Window", "Help"]);
    let at = |h: &Harness, title: &str| h.texts.iter().find(|p| p.text == title && bar.contains_rect(p.rect)).unwrap().rect.center();
    h.click_at(at(&h, "Drawing"));
    let items = h.popup_buttons();
    // The first seven run (Weights, Revert, Cut Face, Scale and Coordinate System, Make Level Lines, Tablet & Pen Settings, Align Underlay); the others wait for their jobs.
    assert_eq!(items.iter().take(7).filter(|(_, enabled)| *enabled).count(), 7, "the seven live items: {items:?}");
    assert!(items.iter().skip(7).all(|(_, enabled)| !enabled), "they wait for their jobs");
    let shown: String = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("|");
    for item in
        ["Weights at Once…", "Make Level Lines", "Scale and Coordinate System…", "Tablet & Pen Settings…", "Align Underlay…", "Send to Strata…"]
    {
        assert!(shown.contains(item), "{item} in {shown}");
    }
    // A second click puts the menu away; View, after it, is the app's own View menu.
    h.click_at(at(&h, "Drawing"));
    assert!(h.popup_buttons().is_empty(), "the Drawing menu closed");
    h.click_at(at(&h, "View"));
    assert!(h.popup_buttons().iter().any(|(_, enabled)| *enabled), "View's items run");
}

#[test]
fn ribbon_follows_the_mockup_and_runs_commands() {
    let mut h = Harness::new();
    let keys = TILES;
    let ribbon = h.panel(RIBBON);
    let tiles: Vec<Rect> = keys.iter().map(|k| h.rect(wid("tile", k)).unwrap_or_else(|| panic!("no tile {k}"))).collect();
    for (k, t) in keys.iter().zip(&tiles) {
        assert_eq!(t.width(), 52.0, "{k}");
        assert!(ribbon.contains_rect(*t), "{k} in the ribbon");
    }
    assert!(tiles.windows(2).all(|w| w[0].right() <= w[1].left()), "left to right as in the mockup");
    let groups = h.line_in(ribbon);
    for g in ["File", "New Drawing", "Underlay", "Draw", "Drawing Elements", "View", "Export"] {
        assert!(groups.contains(g), "{g} in {groups}");
    }
    let t = h.tokens();
    let well = |h: &Harness, k: &str| {
        let tile = h.rect(wid("tile", k)).unwrap();
        h.fill_at(Rect::from_min_size(tile.min + vec2(4.0, 0.0), vec2(44.0, 44.0)))
    };
    // The Pen tile picks the Pen tool and shows it in hand.
    h.click(wid("tile", "pen"));
    assert_eq!(h.app.session.tool_id(), "pen");
    assert_eq!(well(&h, "pen"), Some(t.tool_active));
    // Grid shows the grid, and shows itself on.
    assert!(!h.app.ui.view.grid);
    h.click(wid("tile", "grid"));
    assert!(h.app.ui.view.grid);
    assert_eq!(well(&h, "grid"), Some(t.row_selected));
    // Save turns clay while the document has changes.
    assert_eq!(well(&h, "save"), Some(t.button));
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    h.settle();
    assert_eq!(well(&h, "save"), Some(t.action));
    // A tile a later job brings is dimmed and does nothing.
    assert_eq!(well(&h, "north"), Some(t.panel_darker));
    let docs = h.app.session.documents().len();
    h.click(wid("tile", "north"));
    assert_eq!((h.app.session.documents().len(), h.app.session.tool_id()), (docs, "pen"));
    assert!(h.app.ui.dialog.is_none());
    // Find a command opens the command palette.
    h.click(wid("find", ()));
    assert!(h.app.ui.palette_open);
}

#[test]
fn control_bar_shows_the_layer_and_next_weight() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    h.app.run("layer.new", json!({"name": "Pit outline"})).unwrap();
    h.app.select_tool("pen");
    h.settle();
    let bar = h.panel(CONTROL);
    let line = h.line_in(bar);
    for want in ["Pen", "Pit outline", "1 pt", "Snap", "Tolerance"] {
        assert!(line.contains(want), "{want} in {line}");
    }
    // The legend field lists the layers, in rows about as wide as the field so a padlock or eye at
    // the right stays by its name; picking one makes it current.
    h.click(wid("ctl-layer", ()));
    let row = h.rect(wid("ctl-layer-row", first.0)).expect("the list");
    assert!((220.0..=280.0).contains(&row.width()), "{row:?}");
    h.click(wid("ctl-layer-row", first.0));
    assert_eq!(h.current_layer(), Some(first));
    assert!(h.line_in(bar).contains("Layer 1"), "{}", h.line_in(bar));
    // The Snap toggle is View › Snap to Point.
    let snap = h.app.ui.view.snap_to_point;
    h.click(wid("ctl-snap", ()));
    assert_eq!(h.app.ui.view.snap_to_point, !snap);
    // Other tools keep the classic Control bar: with nothing selected, Document Setup. So do the
    // anchor tools, whose options are there.
    for tool in ["selection", "anchorPoint", "addAnchor", "smooth"] {
        h.app.select_tool(tool);
        h.settle();
        let line = h.line_in(bar);
        assert!(line.contains("Document Setup") && line.contains("Snap") && !line.contains("Legend"), "{tool}: {line}");
    }
}

/// Smaller windows keep every tile and tool and the fill and stroke in reach: a 1920 × 1080 screen
/// at 125 % less the task bar, a 1440 × 900 laptop's, a 1366 × 768 one's, the smallest window. The
/// ribbon narrows its tiles, then folds Find a Command to its magnifier, then scrolls (the mouse
/// wheel too); the tool column tightens, then scrolls above the fill and stroke, which stay at its
/// foot.
#[test]
fn small_windows_keep_every_tile_and_tool_in_reach() {
    for (w, height) in [(1536.0, 816.0), (1440.0, 900.0), (1366.0, 728.0), (800.0, 500.0)] {
        let mut h = Harness::sized(w, height);
        let (ribbon, column) = (h.panel(RIBBON), h.panel(TOOLS));
        let find = h.rect(wid("find", ())).unwrap_or_else(|| panic!("{w}: Find a Command"));
        assert!(ribbon.contains_rect(find), "{w}: {find:?} in {ribbon:?}");
        // Find a Command keeps its field while narrower tiles leave it room (from 1383 points).
        assert_eq!(find.width(), if w >= 1383.0 { 150.0 } else { 32.0 }, "{w}: Find a Command's width");
        let tile_shown = |h: &Harness, k: &str| {
            let r = h.rect(wid("tile", k)).unwrap_or_else(|| panic!("{w}: no tile {k}"));
            ribbon.contains_rect(r) && r.right() <= find.left()
        };
        if w >= 1366.0 {
            let hidden: Vec<_> = TILES.iter().filter(|k| !tile_shown(&h, k)).collect();
            assert!(hidden.is_empty(), "{w}: tiles cut off {hidden:?}");
        } else {
            assert!(tile_shown(&h, "open") && !tile_shown(&h, "strata"), "{w}: the ribbon scrolls");
            h.wheel(ribbon.center(), vec2(0.0, -4000.0));
            assert!(tile_shown(&h, "strata"), "{w}: the wheel brings the last tile in");
        }
        // The fill and stroke stay at the foot of the column, the tools above them.
        let fill = h.rect(wid("proxy", ()).with("fill-proxy")).unwrap_or_else(|| panic!("{w}: the fill and stroke"));
        assert!(column.contains_rect(fill), "{w}: {fill:?} in {column:?}");
        let tool_shown = |h: &Harness, id: egui::Id| {
            let r = h.rect(id).unwrap_or_else(|| panic!("{w}: no {id:?}"));
            column.contains_rect(r) && r.bottom() <= fill.top()
        };
        let all: Vec<egui::Id> = SLOTS.iter().map(|k| wid("tool", k)).chain(ARCH.iter().map(|k| wid("arch", k))).collect();
        if height >= 816.0 {
            let hidden: Vec<_> = all.iter().filter(|id| !tool_shown(&h, **id)).collect();
            assert!(hidden.is_empty(), "{w}: tools cut off {hidden:?}");
        } else {
            let last = wid("arch", "hachure");
            assert!(tool_shown(&h, wid("tool", "selection")) && !tool_shown(&h, last), "{w}: the column scrolls");
            h.wheel(h.rect(wid("tool", "selection")).unwrap().center(), vec2(0.0, -4000.0));
            assert!(tool_shown(&h, last) && tool_shown(&h, wid("tool", "zoom")), "{w}: the wheel brings the last tools in");
        }
        // Find a Command still opens the command palette.
        h.click(wid("find", ()));
        assert!(h.app.ui.palette_open, "{w}");
    }
}

/// A window too short for the right column scrolls it (the smallest, 800 × 500): the drawing
/// palette, the legend with room for two layers and the properties stay apart, and the wheel brings
/// the properties in. A taller one shows them all as they come.
#[test]
fn a_short_window_scrolls_the_right_column_without_overlap() {
    for (w, height) in [(1600.0, 988.0), (1366.0, 728.0), (800.0, 500.0)] {
        let h = Harness::sized(w, height);
        let right = h.panel(RIGHT);
        let clash = h.overlapping_texts(right);
        assert!(clash.is_empty(), "{w} × {height}: {clash:?}");
        assert!(right.contains_rect(h.rect(wid("weight", "0.1")).unwrap()), "{w} × {height}: the palette at the top");
    }
    let mut h = Harness::sized(800.0, 500.0);
    let right = h.panel(RIGHT);
    h.wheel(h.rect(wid("weight", "0.1")).unwrap().center(), vec2(0.0, -4000.0));
    let more = h.rect(wid("props-more", ())).expect("the properties' ⋯");
    assert!(right.contains_rect(more), "{more:?} in {right:?}");
    let clash = h.overlapping_texts(right);
    assert!(clash.is_empty(), "scrolled to the properties: {clash:?}");
    let layers = h.texts.iter().find(|p| p.text == "Legend · Layers").expect("the legend's head").shown;
    assert!(right.contains_rect(layers) && layers.bottom() < more.top(), "the legend above the properties");
    h.click(wid("props-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("properties"));
}

/// However short the window, the legend's list keeps to its own room: no layer's row runs over the
/// line and buttons of Weights at Once under it, and scrolled to its end the list shows the bottom
/// layer whole.
#[test]
fn the_legend_list_stays_above_its_footer() {
    for (w, height) in [(1280.0, 662.0), (800.0, 500.0)] {
        let mut h = Harness::sized(w, height);
        for i in 0..5 {
            h.app.run("layer.new", json!({ "name": format!("Pit {i}") })).unwrap();
        }
        h.settle();
        let names: Vec<String> = h.app.session.active().unwrap().doc.layers.iter().map(|l| l.display_name()).collect();
        // Down the column to the legend's foot (a short one scrolls), then down the list to its end.
        h.wheel(h.rect(wid("weight", "0.1")).unwrap().center(), vec2(0.0, -4000.0));
        let weights = h.rect(wid("legend-weights", ())).expect("Change All Weights…");
        assert!(h.panel(RIGHT).contains_rect(weights), "{w} × {height}: {weights:?}");
        // legend::footer: its line is 6 above its buttons.
        let line = weights.top() - 6.0;
        let head = h.texts.iter().find(|p| p.text == "Legend · Layers").expect("the legend's head").rect;
        h.wheel(pos2(head.center().x, (head.bottom() + line) / 2.0), vec2(0.0, -4000.0));
        let rows: Vec<&Painted> = h.texts.iter().filter(|p| names.contains(&p.text) && p.shown.is_positive()).collect();
        for p in &rows {
            assert!(p.shown.bottom() <= line, "{w} × {height}: {:?} over the footer: {:?}, its line at {line}", p.text, p.shown);
        }
        let bottom = rows.iter().find(|p| Some(&p.text) == names.first()).expect("the bottom layer in view");
        assert!(close(bottom.shown, bottom.rect), "{w} × {height}: the bottom layer cut: {:?} of {:?}", bottom.shown, bottom.rect);
    }
}

/// The tab strip in a narrow window, or with a long name, never draws a tab over the drawing's
/// name and "Not saved" at its right. The name there (its tab shows it too) gives way first when the
/// tabs don't all fit, then "Not saved" leaves its dot when the current tab doesn't; past that the
/// current tab's name is cut short with "…".
#[test]
fn the_tabs_and_the_name_at_their_right_never_overlap() {
    let long = "Excavation area 3, trench 12, north section, second season";
    // The window, the drawing opened after "Site 1" (the current one), and what shows at the right:
    // the name, "Not saved", whether the current tab's name is cut short.
    let cases = [
        (1600.0, 988.0, None, true, true, false),
        (1366.0, 728.0, Some(long), false, true, false),
        (800.0, 500.0, Some("Trench 12 north"), false, false, false),
        (800.0, 500.0, Some("Trench 12 north section"), false, false, true),
    ];
    for (w, height, other, name_shown, note_shown, cut) in cases {
        let mut h = Harness::sized(w, height);
        if let Some(name) = other {
            h.app.run("file.new", json!({"name": name, "width": 400, "height": 300})).unwrap();
        }
        let name = other.unwrap_or("Site 1");
        h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
        h.settle();
        let (set, right, control) = (h.panel(SET), h.panel(RIGHT), h.panel(CONTROL));
        let strip = r(set.right(), control.bottom(), right.left(), control.bottom() + 32.0);
        let clash = h.overlapping_texts(strip);
        assert!(clash.is_empty(), "{w} {name}: {clash:?}");
        // Nor does a tab (its ×, its underline) run under what is at the right.
        let docs = h.app.session.documents().len();
        let tabs: Vec<Rect> = std::iter::once(wid("tab-home", ())).chain((0..docs).map(|i| wid("tab", i))).filter_map(|id| h.rect(id)).collect();
        for p in h.texts.iter().filter(|p| p.shown.is_positive() && strip.contains_rect(p.shown)) {
            if !tabs.iter().any(|t| t.expand(0.5).contains_rect(p.rect)) {
                assert!(tabs.iter().all(|t| !t.intersects(p.shown)), "{w} {name}: a tab under {:?}: {tabs:?}", p.text);
            }
        }
        // The current tab shows its name, inside it; cut short only when even the dot leaves no room.
        let tab = h.rect(wid("tab", h.app.session.active_index().unwrap())).unwrap();
        let elided: Vec<bool> = h
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) if t.galley.text() == name && tab.expand(0.5).contains_rect(t.visual_bounding_rect()) => Some(t.galley.elided),
                _ => None,
            })
            .collect();
        assert_eq!(elided, [cut], "{w} {name}: the current tab's name");
        let shown = h.texts_in(strip);
        assert_eq!(shown.iter().filter(|s| *s == name).count(), if name_shown { 2 } else { 1 }, "{w} {name}: {shown:?}");
        assert_eq!(shown.iter().any(|s| s == "Not saved"), note_shown, "{w} {name}: {shown:?}");
        assert_eq!(h.rect(wid("tab-unsaved", ())).is_some(), !note_shown, "{w} {name}: the dot says it on hover");
    }
}

#[test]
fn tool_column_has_the_mockup_groups() {
    let mut h = Harness::new();
    let primaries = SLOTS;
    let column = h.panel(TOOLS);
    let rects: Vec<Rect> = primaries.iter().map(|p| h.rect(wid("tool", p)).unwrap_or_else(|| panic!("no tool {p}"))).collect();
    for (p, rect) in primaries.iter().zip(&rects) {
        assert_eq!(rect.size(), vec2(40.0, 27.0), "{p}");
        assert!(column.contains_rect(*rect), "{p} in the column");
    }
    assert!(rects.windows(2).all(|w| w[0].bottom() <= w[1].top()), "top to bottom as in the mockup");
    // The groups part after V A, after T, after C and after I.
    let gap = |a: usize| rects[a + 1].top() - rects[a].bottom();
    for a in [1, 6, 9, 11] {
        assert!(gap(a) > 4.0, "a separator after {}", primaries[a]);
    }
    assert!(gap(0) < 2.0 && gap(3) < 2.0);
    h.click(wid("tool", "pen"));
    assert_eq!(h.app.session.tool_id(), "pen");
    // The Pen slot keeps the tool last used in it.
    h.app.select_tool("curvature");
    h.settle();
    h.click(wid("tool", "selection"));
    assert_eq!(h.app.session.tool_id(), "selection");
    h.click(wid("tool", "pen"));
    assert_eq!(h.app.session.tool_id(), "curvature");
    // A right click opens the slot's flyout.
    let at = h.rect(wid("tool", "pen")).unwrap().center();
    h.press(at, PointerButton::Secondary);
    assert!(h.app.ui.flyout.is_some(), "the Pen slot's flyout");
    // The archaeology tools wait for J009.
    for a in ARCH {
        let rect = h.rect(wid("arch", a)).unwrap_or_else(|| panic!("no {a}"));
        assert!(column.contains_rect(rect) && rect.top() > rects[13].bottom(), "{a} under the tools");
    }
    h.app.ui.flyout = None;
    h.click(wid("arch", "section-line"));
    assert_eq!(h.app.session.tool_id(), "section-line");
}

#[test]
fn drawing_set_lists_open_documents_and_switches() {
    let mut h = Harness::new();
    h.app.run("file.new", json!({"name": "Site 2", "width": 200, "height": 100})).unwrap();
    h.settle();
    let set = h.panel(SET);
    let line = h.line_in(set);
    assert!(line.contains("Drawing Set") && line.contains("Site 1") && line.contains("Site 2"), "{line}");
    assert!(line.contains("Scale not set"), "{line}");
    let (a, b) = (h.rect(wid("set-row", 0usize)).unwrap(), h.rect(wid("set-row", 1usize)).unwrap());
    assert!(set.contains_rect(a) && a.bottom() <= b.top());
    h.click(wid("set-row", 0usize));
    assert_eq!(h.app.session.active_index(), Some(0));
    // From the Home screen too.
    h.app.run("app.home", json!({})).unwrap();
    h.settle();
    assert!(crate::menus::home_showing(&h.app));
    h.click(wid("set-row", 1usize));
    assert!(!crate::menus::home_showing(&h.app));
    assert_eq!(h.app.session.active_index(), Some(1));
    // ＋ is File › New.
    h.click(wid("set-new", ()));
    assert!(h.app.ui.dialog.is_some(), "the New Document dialog");
}

#[test]
fn tabs_switch_documents_and_show_unsaved() {
    let mut h = Harness::new();
    h.app.run("file.new", json!({"name": "Site 2", "width": 200, "height": 100})).unwrap();
    h.settle();
    let canvas = h.app.canvas_rect.unwrap();
    let strip = r(272.0, 196.0, 1296.0, canvas.top() - 16.0);
    let line = h.line_in(strip);
    assert!(line.contains("Home") && line.contains("Site 1") && line.contains("Site 2"), "{line}");
    assert!(!line.contains("Not saved"), "{line}");
    h.click(wid("tab", 0usize));
    assert_eq!(h.app.session.active_index(), Some(0));
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    h.settle();
    assert!(h.line_in(strip).contains("Not saved"), "{}", h.line_in(strip));
    h.click(wid("tab-home", ()));
    assert!(crate::menus::home_showing(&h.app));
    h.click(wid("tab", 1usize));
    assert_eq!(h.app.session.active_index(), Some(1));
    // The × of a tab closes it (Site 2 has no changes: no question).
    h.hover(wid("tab", 1usize));
    h.click(wid("tab-close", 1usize));
    assert_eq!(h.app.session.documents().len(), 1);
}

#[test]
fn canvas_overlays_zoom_and_name_the_tool() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let canvas = h.app.canvas_rect.unwrap();
    let line = h.line_in(canvas);
    assert!(line.contains("Pen · Layer 1"), "{line}");
    assert!(line.contains("to add a corner point"), "the Pen's hint: {line}");
    let zoom = |h: &Harness| h.app.view().unwrap().zoom;
    let z0 = zoom(&h);
    // The screen readout: a whole percentage.
    assert!(line.contains(&format!("{:.0} %", z0 * 100.0)) && !line.contains(&format!("{:.2}", z0 * 100.0)), "{line}");
    for k in ["in", "out", "fit"] {
        assert!(canvas.contains_rect(h.rect(wid("zoom", k)).unwrap()), "{k}");
    }
    // Presses on what floats over the canvas don't reach the Pen under it.
    let nodes = |h: &Harness| h.app.session.active().unwrap().doc.node_count();
    let n = nodes(&h);
    h.click(wid("zoom", "in"));
    assert!(zoom(&h) > z0);
    h.click(wid("zoom", "out"));
    h.click(wid("zoom", "out"));
    assert!(zoom(&h) < z0);
    h.click(wid("zoom", "fit"));
    assert!((zoom(&h) - z0).abs() < 1e-6, "back to the fitted view");
    h.click(wid("band", ()));
    h.click(wid("readout", ()));
    assert_eq!(nodes(&h), n, "no point drawn under the zoom buttons, the band or the readout");
}

/// The tool's hint reads plainly, as in the mockup: no bold keys, its parts apart by "·"; on the
/// canvas the first part, in the status bar all of them.
#[test]
fn hints_read_plainly_as_in_the_mockup() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let semibold = egui::FontFamily::Name(crate::theme::FONT_UI_SEMIBOLD.into());
    // The hint painted in `within`: its text and whether any of it is bold.
    let hint = |h: &Harness, within: Rect| -> Vec<(String, bool)> {
        h.shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) if within.contains_rect(t.visual_bounding_rect()) && t.galley.text().contains("to add a corner point") => {
                    let job = &t.galley.job;
                    Some((job.text.clone(), job.sections.iter().any(|s| s.format.font_id.family == semibold)))
                }
                _ => None,
            })
            .collect()
    };
    let band = hint(&h, h.app.canvas_rect.unwrap());
    assert_eq!(band, [("Click to add a corner point".to_string(), false)], "the first part only");
    let status = hint(&h, h.panel(STATUS));
    assert_eq!(status.len(), 1, "{status:?}");
    let (text, bold) = &status[0];
    assert!(!bold, "{status:?}");
    assert_eq!(text, "Click to add a corner point · Drag to add a smooth point · Click the first point to close · Enter to finish");
}

#[test]
fn palette_weight_chip_sets_the_next_stroke() {
    let mut h = Harness::new();
    let t = h.tokens();
    let right = h.panel(RIGHT);
    for w in ["0.1", "0.2", "0.3", "0.5", "0.8", "1.0"] {
        assert!(right.contains_rect(h.rect(wid("weight", w)).unwrap()), "{w}");
    }
    // Nothing selected: the weight of the next line drawn.
    h.click(wid("weight", "0.5"));
    assert_eq!(h.app.session.paint.stroke_width, 0.5);
    assert_eq!(h.fill_at(h.rect(wid("weight", "0.5")).unwrap()), Some(t.row_selected), "the chip shows on");
    // With a line selected: its weight.
    let id = h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap()["id"].clone();
    h.app.run("select.set", json!({ "ids": [id] })).unwrap();
    h.app.run("stroke.set", json!({"dash": [4, 2]})).unwrap();
    h.settle();
    h.click(wid("weight", "0.8"));
    let stroke = h.app.session.shown_stroke().unwrap();
    assert_eq!(stroke.width, 0.8);
    // Solid clears the dash.
    assert!(stroke.dash.is_some());
    h.click(wid("dash", "solid"));
    assert!(h.app.session.shown_stroke().unwrap().dash.is_none());
    // Adding a weight waits for J018.
    assert_eq!(h.fill_at(h.rect(wid("weight-add", ())).unwrap()), Some(t.panel_darker));
    // Hatch chips run with J014: clicking 45° sets the layer's hatch.
    h.click(wid("hatch", "45"));
    assert_eq!(h.fill_at(h.rect(wid("hatch", "45")).unwrap()), Some(t.row_selected));
    assert_eq!(h.app.session.shown_stroke().unwrap().width, 0.8);
    // ⋯ pops the Stroke panel out.
    h.click(wid("palette-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("stroke"));
}

#[test]
fn legend_rows_show_hide_lock_and_pick_the_layer() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    let pits = NodeId(h.app.run("layer.new", json!({"name": "Pits"})).unwrap()["id"].as_u64().unwrap());
    h.settle();
    let right = h.panel(RIGHT);
    let (top, below) = (h.rect(wid("legend-row", pits.0)).unwrap(), h.rect(wid("legend-row", first.0)).unwrap());
    assert!(right.contains_rect(top) && top.bottom() <= below.top(), "the top layer first");
    assert_eq!(top.height(), 24.0);
    let layer = |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().clone();
    h.click(wid("legend-eye", first.0));
    assert!(!layer(&h, first).visible);
    h.click(wid("legend-eye", first.0));
    assert!(layer(&h, first).visible);
    h.click(wid("legend-lock", first.0));
    assert!(layer(&h, first).locked);
    h.click(wid("legend-lock", first.0));
    assert!(!layer(&h, first).locked);
    assert_eq!(h.current_layer(), Some(pits));
    h.click(wid("legend-row", first.0));
    assert_eq!(h.current_layer(), Some(first));
    // ▸ lists the layer's objects.
    h.click(wid("legend-open", first.0));
    let objects = h.app.session.active().unwrap().doc.node(first).unwrap().children().unwrap().clone();
    let obj = h.rect(wid("legend-obj", objects[0].id.0)).expect("the rectangle's row");
    assert_eq!(obj.height(), 20.0);
    // ＋ adds a layer.
    let n = h.layer_ids().len();
    h.click(wid("legend-new", ()));
    assert_eq!(h.layer_ids().len(), n + 1);
    let line = h.line_in(right);
    assert!(line.contains("Legend · Layers") && line.contains("Pits"), "{line}");
}

/// A locked or hidden layer takes no new lines (the engine draws on the topmost layer that is shown
/// and unlocked instead), so when the current one is, the band, the Control bar and the legend say
/// where lines go rather than naming a layer they won't land in.
#[test]
fn a_locked_or_hidden_current_layer_shows_where_lines_go() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let pits = NodeId(h.app.run("layer.new", json!({"name": "Pits"})).unwrap()["id"].as_u64().unwrap());
    h.app.select_tool("pen");
    h.settle();
    let (canvas, bar, right) = (h.app.canvas_rect.unwrap(), h.panel(CONTROL), h.panel(RIGHT));
    let t = h.tokens();
    assert!(h.line_in(canvas).contains("Pen · Pits"), "{}", h.line_in(canvas));
    assert_eq!(h.color_of("Pits", right), Some(t.text));
    h.click(wid("legend-lock", pits.0));
    assert_eq!(h.current_layer(), Some(pits), "still the current layer");
    let line = h.line_in(canvas);
    assert!(line.contains("Pen · Layer 1") && line.contains("Pits is locked: drawing on Layer 1"), "{line}");
    assert!(h.line_in(bar).contains("Layer 1") && !h.line_in(bar).contains("Pits"), "{}", h.line_in(bar));
    assert_eq!(h.color_of("Pits", right), Some(t.clay_text), "the legend marks the current row");
    let id = h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap()["id"].as_u64().unwrap();
    assert_eq!(h.app.session.active().unwrap().doc.layer_containing(NodeId(id)), Some(first), "where the band said");
    // Hidden likewise.
    h.app.run("layer.setCurrent", json!({ "id": pits.0 })).unwrap();
    h.click(wid("legend-lock", pits.0));
    h.click(wid("legend-eye", pits.0));
    assert_eq!(h.current_layer(), Some(pits));
    assert!(h.line_in(canvas).contains("Pits is hidden: drawing on Layer 1"), "{}", h.line_in(canvas));
    // With every layer locked or hidden nothing is drawn, and the band says so; the legend field
    // too, from its left, with no layer colour or weight.
    h.click(wid("legend-lock", first.0));
    let line = h.line_in(canvas);
    assert!(line.contains("Pits is hidden: no layer to draw on") && !line.contains("Pen · Layer 1"), "{line}");
    let field = h.rect(wid("ctl-layer", ())).unwrap();
    assert_eq!(h.texts_in(field), ["No layer to draw on"]);
    let says = h.texts.iter().find(|p| p.text == "No layer to draw on" && field.contains_rect(p.rect)).unwrap().rect;
    assert!(says.left() - field.left() < 12.0, "from the field's left: {says:?} in {field:?}");
    assert!(h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).is_err());
}

/// A layer of thousands of objects opened in the legend lays out only the rows in view (the rest
/// scroll in), so it doesn't slow every frame down.
#[test]
fn a_big_open_layer_lays_out_only_the_rows_in_view() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let ids: Vec<u64> = (0..1500)
        .map(|i| h.app.run("shape.rectangle", json!({"x": i % 300, "y": 10, "width": 5, "height": 5})).unwrap()["id"].as_u64().unwrap())
        .collect();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.click(wid("legend-open", first.0));
    let rows = |h: &Harness| ids.iter().filter(|id| h.rect(wid("legend-obj", **id)).is_some()).count();
    let laid_out = rows(&h);
    assert!((5..60).contains(&laid_out), "{laid_out} of {} rows laid out", ids.len());
    // The top one (the last drawn) shows; the wheel brings the bottom one (the first drawn).
    let right = h.panel(RIGHT);
    let (top, bottom) = (*ids.last().unwrap(), ids[0]);
    assert!(right.contains_rect(h.rect(wid("legend-obj", top)).unwrap()));
    assert!(h.rect(wid("legend-obj", bottom)).is_none());
    let list = h.rect(wid("legend-obj", top)).unwrap().center();
    h.wheel(list, vec2(0.0, -1.0e6));
    let row = h.rect(wid("legend-obj", bottom)).expect("the bottom row, scrolled in");
    assert!(right.contains_rect(row), "{row:?}");
    // A click on it selects it.
    h.click(wid("legend-obj", bottom));
    assert_eq!(h.app.session.active().unwrap().selection.objects, [NodeId(bottom)]);
}

/// A message shows in the status bar for a while, then the hint again; a press on something a
/// later job brings says so there.
#[test]
fn status_bar_shows_messages_for_a_while() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let bar = h.panel(STATUS);
    assert!(h.line_in(bar).contains("to add a corner point"), "{}", h.line_in(bar));
    h.app.status("Saved Site 1");
    h.settle();
    let line = h.line_in(bar);
    assert!(line.contains("Saved Site 1") && !line.contains("to add a corner point"), "{line}");
    h.later(7.0);
    let line = h.line_in(bar);
    assert!(!line.contains("Saved Site 1") && line.contains("to add a corner point"), "{line}");
    // The same message again shows again: a second save says so too.
    h.app.status("Saved Site 1");
    h.settle();
    assert!(h.line_in(bar).contains("Saved Site 1"), "the same message again: {}", h.line_in(bar));
    h.later(7.0);
    // So does the same error twice: the status bar is where it reaches the user.
    for time in ["first", "second"] {
        assert!(h.app.run("layer.setCurrent", json!({ "id": 999_999 })).is_err());
        h.settle();
        let line = h.line_in(bar);
        assert!(!line.contains("to add a corner point"), "the error, the {time} time: {line}");
        h.later(7.0);
    }
    // North Arrow (J012): what it is and when.
    h.click(wid("tile", "north"));
    assert!(h.line_in(bar).contains("North Arrow: coming soon (J012)"), "{}", h.line_in(bar));
    h.later(5.0);
    // Cut Face runs with J014: with nothing selected, status explains what to select.
    h.click(wid("palette-cut", ()));
    assert!(h.line_in(bar).contains("select at least one closed shape"), "{}", h.line_in(bar));
    h.later(7.0);
    assert!(!h.line_in(bar).contains("select at least one closed shape"), "{}", h.line_in(bar));
}

/// The Underlay tile places a scan as a template (a locked, dimmed layer under the drawing); with
/// one placed it shows and hides the template layers.
#[test]
fn underlay_tile_places_a_template_then_shows_and_hides_it() {
    let mut h = Harness::new();
    let dir = std::env::temp_dir().join(format!("trowel-underlay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let scan = dir.join("scan.svg");
    std::fs::write(&scan, r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30"/></svg>"#).unwrap();
    let path = scan.to_string_lossy().to_string();
    h.app.services.pick_open = Some(Box::new(move |_| Some(path.clone())));
    h.click(wid("tile", "underlay"));
    let d = h.app.ui.dialog.as_ref().expect("the Place dialog");
    assert_eq!(d.kind, crate::dialogs::place::KIND);
    assert_eq!(d.fields.get("template"), Some(&json!(true)), "Template ticked");
    h.app.ui.dialog = None;
    // With a template layer holding a scan: the tile shows it on and hides it, then shows it again.
    let id = h.app.run("layer.new", json!({"name": "Scan"})).unwrap()["id"].as_u64().unwrap();
    draw_rect(&mut h, 10.0);
    h.app.run("layer.setProps", json!({"ids": [id], "template": true})).unwrap();
    h.settle();
    let t = h.tokens();
    let tile = h.rect(wid("tile", "underlay")).unwrap();
    let well = Rect::from_min_size(pos2(tile.center().x - 22.0, tile.top()), vec2(44.0, 44.0));
    assert_eq!(h.fill_at(well), Some(t.row_selected), "shown");
    let visible = |h: &Harness| h.app.session.active().unwrap().doc.node(NodeId(id)).unwrap().visible;
    h.click(wid("tile", "underlay"));
    assert!(!visible(&h) && h.app.ui.dialog.is_none());
    h.click(wid("tile", "underlay"));
    assert!(visible(&h));
    std::fs::remove_dir_all(&dir).ok();
}

/// The panels ⋯ opens pop out beside the right column, just under its top, however the classic
/// dock was left; with Auto-Collapse Iconic Panels on, a press on the right column puts them away.
#[test]
fn panels_pop_out_beside_the_right_column() {
    let mut h = Harness::new();
    h.app.ui.dock_collapsed = false;
    h.app.session.prefs.auto_collapse_icon_panels = true;
    h.click(wid("props-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("properties"));
    let right = h.panel(RIGHT);
    let flyout = h.ctx.memory(|m| m.area_rect(egui::Id::new("icon-panel"))).expect("the panel popped out");
    assert!((flyout.right() - (right.left() - 6.0)).abs() < 1.0 && (flyout.top() - (right.top() + 6.0)).abs() < 1.0, "{flyout:?} by {right:?}");
    h.click_at(pos2(right.left() + 12.0, right.top() + 8.0));
    assert!(h.app.ui.open_panel.is_none(), "put away");
}

/// The snapping tolerance is a field like the mockup's: a click lists 1 to 8 px, and a pick sets it.
#[test]
fn snap_tolerance_is_picked_from_a_list() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let field = h.rect(wid("ctl-tolerance", ())).unwrap();
    assert_eq!(h.texts_in(field), ["2", "px"]);
    h.click(wid("ctl-tolerance", ()));
    let items = h.popup_buttons();
    assert_eq!(items.len(), 8, "{items:?}");
    h.click_at(items[3].0.center());
    assert_eq!(h.app.session.prefs.snap_to_point_tolerance, 4.0);
    let field = h.rect(wid("ctl-tolerance", ())).unwrap();
    assert_eq!(h.texts_in(field), ["4", "px"]);
}

/// In a small window the drawing tools' Control bar is cut at Snap rather than running under it.
#[test]
fn control_bar_keeps_snap_in_reach_in_a_small_window() {
    let mut h = Harness::sized(800.0, 500.0);
    h.app.select_tool("pen");
    h.settle();
    // Nothing else shows over Snap and its tolerance…
    let toggle = h.rect(wid("ctl-snap", ())).unwrap();
    let bar = h.panel(CONTROL);
    let snap_area = Rect::from_min_max(pos2(toggle.left(), bar.top()), bar.max);
    let under: Vec<String> = h
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Text(t) if !["Snap", "Tolerance", "2", "px"].contains(&t.galley.text()) => {
                let shown = c.clip_rect.intersect(t.visual_bounding_rect());
                (shown.is_positive() && shown.intersects(snap_area)).then(|| t.galley.text().to_string())
            }
            _ => None,
        })
        .collect();
    assert!(under.is_empty(), "over Snap: {under:?}");
    // …and a click on it reaches it.
    let snap = h.app.ui.view.snap_to_point;
    h.click(wid("ctl-snap", ()));
    assert_eq!(h.app.ui.view.snap_to_point, !snap);
}

#[test]
fn properties_summarise_the_selection() {
    let mut h = Harness::new();
    let right = h.panel(RIGHT);
    assert!(h.line_in(right).contains("No selection"), "{}", h.line_in(right));
    let id = h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 100, "height": 50})).unwrap()["id"].clone();
    h.app.run("select.set", json!({ "ids": [id] })).unwrap();
    h.settle();
    let u = h.app.session.general_unit();
    let size = format!("{} × {}", u.format(100.0), u.format(50.0));
    assert!(h.line_in(right).contains(&size), "{size} in {}", h.line_in(right));
    h.click(wid("props-more", ()));
    assert_eq!(h.app.ui.open_panel.as_deref(), Some("properties"));
}

#[test]
fn status_bar_shows_tool_snap_unsaved_and_position() {
    let mut h = Harness::new();
    h.app.select_tool("pen");
    h.settle();
    let bar = h.panel(STATUS);
    let line = h.line_in(bar);
    let snap = if h.app.ui.view.snap_to_point { "Snap on" } else { "Snap off" };
    assert!(line.contains("Pen") && line.contains(snap) && line.contains("Scale and coordinate system not set"), "{line}");
    assert!(!line.contains("not saved"), "{line}");
    h.app.run("shape.rectangle", json!({"x": 10, "y": 10, "width": 50, "height": 40})).unwrap();
    // The pointer over the canvas: its position in the document.
    let at = h.app.canvas_rect.unwrap().center();
    h.frame(vec![Event::PointerMoved(at)]);
    h.settle();
    let p = h.app.hover_doc.expect("over the canvas");
    let u = h.app.session.general_unit();
    let line = h.line_in(bar);
    assert!(line.contains("1 not saved"), "{line}");
    assert!(line.contains(&u.readout(p.x)) && line.contains(&u.readout(p.y)), "{line}");
}

/// Trowel's colours come from the theme tokens, as everywhere in the app.
#[test]
fn trowel_code_takes_colours_from_tokens() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/trowel");
    let mut found = vec![];
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "tests.rs" {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).unwrap();
        for bad in ["Color32::from_", "Color32::WHITE", "Color32::BLACK", "Color32::GRAY", "Color32::LIGHT", "Color32::DARK", "Rgba::"] {
            if text.contains(bad) {
                found.push(format!("{name}: {bad}"));
            }
        }
    }
    assert!(found.is_empty(), "colours outside theme::Tokens: {found:?}");
}

/// Every Trowel icon the window names is bundled.
#[test]
fn every_trowel_icon_is_bundled() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/trowel");
    let mut names = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        if entry.file_name() == "tests.rs" {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).unwrap();
        for (at, _) in text.match_indices("\"tw-") {
            let rest = &text[at + 1..];
            names.insert(rest[..rest.find('"').unwrap()].to_string());
        }
    }
    assert!(names.len() >= 37, "{names:?}");
    let missing: Vec<_> = names.iter().filter(|n| !crate::icons::exists(n)).collect();
    assert!(missing.is_empty(), "icons not bundled: {missing:?}");
}

// ---------- J005: the legend's rows ----------

/// A rectangle drawn on the current layer with the pen's look; its id.
fn draw_rect(h: &mut Harness, x: f64) -> NodeId {
    NodeId(h.app.run("shape.rectangle", json!({"x": x, "y": 10, "width": 20, "height": 20})).unwrap()["id"].as_u64().unwrap())
}

/// A new top layer `name` (the current one) made a legend row with `legend`, `legend.set`'s params.
fn legend_row(h: &mut Harness, name: &str, legend: serde_json::Value) -> NodeId {
    let id = NodeId(h.app.run("layer.new", json!({ "name": name })).unwrap()["id"].as_u64().unwrap());
    let mut p = legend;
    p["id"] = json!(id.0);
    h.app.run("legend.set", p).unwrap();
    id
}

fn weight_of(h: &Harness, id: NodeId) -> f64 {
    h.app.session.active().unwrap().doc.node(id).unwrap().appearance.stroke().unwrap().width
}

/// The rows of `ids` laid out in the legend, top to bottom.
fn row_order(h: &Harness, ids: &[NodeId]) -> Vec<NodeId> {
    let mut v: Vec<(f32, NodeId)> = ids.iter().filter_map(|id| h.rect(wid("legend-row", id.0)).map(|r| (r.top(), *id))).collect();
    v.sort_by(|a, b| a.0.total_cmp(&b.0));
    v.into_iter().map(|(_, id)| id).collect()
}

/// A legend row shows its look (a sample drawn at its weight, the weight in points), ★ and a bold
/// clay name when it is a favourite, how many of its lines are set apart, and its number of
/// objects; a mark row shows its mark, and a layer without a row its colour as before (design 2.1
/// §7, criterion 11).
#[test]
fn legend_rows_show_the_look_star_and_overrides() {
    let mut h = Harness::new();
    let t = h.tokens();
    let first = h.layer_ids()[0];
    let pits = legend_row(&mut h, "Pit outline", json!({"weight": 0.8, "sample": "area", "favourite": true}));
    // Its lines: two in its look, one set apart at 0.5.
    h.app.run("stroke.set", json!({"weight": 0.8})).unwrap();
    for x in [10.0, 40.0] {
        draw_rect(&mut h, x);
    }
    let apart = draw_rect(&mut h, 70.0);
    h.app.run("select.set", json!({ "ids": [apart.0] })).unwrap();
    h.app.run("stroke.set", json!({"weight": 0.5})).unwrap();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    let section = legend_row(&mut h, "Section line", json!({"weight": 0.2, "line": "dashdot"}));
    let datum = legend_row(&mut h, "Datum", json!({"weight": null, "sample": {"mark": "△"}}));
    h.settle();
    let row = |h: &Harness, id: NodeId| h.rect(wid("legend-row", id.0)).unwrap();
    let pr = row(&h, pits);
    assert_eq!(h.texts_in(pr), ["★", "Pit outline", "Apart 1", "0.8", "3"]);
    assert_eq!(h.color_of("★", pr), Some(t.clay));
    assert_eq!(h.color_of("Pit outline", pr), Some(t.clay_text), "a favourite's name is clay");
    assert_eq!(h.font_of("Pit outline", pr).unwrap().family, crate::theme::semibold(12.0).family, "and bold");
    assert_eq!(h.color_of("Apart 1", pr), Some(t.caution));
    assert_eq!(h.color_of("0.8", pr), Some(t.text_dim));
    let badge = h.texts.iter().find(|p| p.text == "Apart 1").unwrap().rect;
    let rects = h.rects_in(pr);
    assert!(
        rects.iter().any(|(r, fill, edge)| *fill == t.caution_wash && edge.color == t.caution_edge && r.contains_rect(badge)),
        "the badge on the caution wash: {rects:?}"
    );
    // Its sample: an area 20 × 12 edged at 0.8 pt's 2 points.
    assert!(
        rects.iter().any(|(r, _, edge)| (r.width() - 20.0).abs() < 0.5
            && (r.height() - 12.0).abs() < 0.5
            && (edge.width - 2.0).abs() < 1e-3
            && edge.color == t.text),
        "the area sample: {rects:?}"
    );
    // A dash-dot line at 0.2 pt: thin dashes, 0.6 points wide.
    let sr = row(&h, section);
    assert_eq!(h.texts_in(sr), ["Section line", "0.2", "0"]);
    let sample = Rect::from_min_max(pos2(sr.left() + 56.0, sr.top()), pos2(sr.left() + 90.0, sr.bottom()));
    let dashes: Vec<_> = h.segments_in(sample).into_iter().filter(|(_, s)| (s.width - 0.6).abs() < 1e-3 && s.color == t.text).collect();
    assert!(dashes.len() >= 3, "dashes: {:?}", h.segments_in(sample));
    // A mark row: its mark (△ drawn as a shape: [`mark_samples_are_as_large_as_the_mockup`]) and
    // no weight. It is the current row: on oat, its name regular.
    let dr = row(&h, datum);
    assert_eq!(h.texts_in(dr), ["Datum", "0"]);
    assert_eq!(h.font_of("Datum", dr).unwrap().family, egui::FontFamily::Proportional);
    assert_eq!(h.color_of("Datum", dr), Some(t.text));
    // A layer without a row, as before: its colour, name and count.
    let lr = row(&h, first);
    assert_eq!(h.texts_in(lr), ["Layer 1", "0"]);
    assert!(h.rects_in(lr).iter().any(|(_, _, edge)| (edge.width - 1.5).abs() < 1e-3), "its colour patch");
    // ★ and ↺ are in the fonts the app bundles: no missing-glyph boxes on any system.
    let ctx = egui::Context::default();
    crate::theme::install_fonts(&ctx);
    ctx.run_ui(egui::RawInput::default(), |_| {}).textures_delta.clear();
    for c in ['★', '↺', '△', '→', '×'] {
        assert!(ctx.fonts_mut(|f| f.has_glyph(&egui::FontId::proportional(11.0), c)), "{c}");
    }
}

/// In the legend the favourites come first, then the other rows in layer order, top first
/// (criterion 12).
#[test]
fn favourites_come_first_in_each_section() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let a = legend_row(&mut h, "A", json!({"weight": 0.5}));
    let b = legend_row(&mut h, "B", json!({"weight": 0.5, "favourite": true}));
    let c = legend_row(&mut h, "C", json!({"weight": 0.2}));
    h.settle();
    assert_eq!(row_order(&h, &[first, a, b, c]), [b, c, a, first]);
    assert!(h.texts.iter().any(|p| p.text == "This drawing's layers"), "one section without Strata rows");
    // No longer a favourite, B goes back to its place.
    h.app.run("legend.set", json!({"id": b.0, "favourite": false})).unwrap();
    h.settle();
    assert_eq!(row_order(&h, &[first, a, b, c]), [c, b, a, first]);
}

/// Rows matching a Strata layer get a section of their own above the others, each section with its
/// favourites first (criterion 12).
#[test]
fn strata_rows_have_their_own_section() {
    let mut h = Harness::sized(1600.0, 1600.0);
    let first = h.layer_ids()[0];
    let added: Vec<NodeId> = h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap()["added"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| NodeId(v.as_u64().unwrap()))
        .collect();
    assert_eq!(added.len(), 15);
    h.settle();
    let doc = h.app.session.active().unwrap().doc.clone();
    let legend = |id: NodeId| doc.node(id).unwrap().legend().cloned().unwrap();
    let caption = |h: &Harness, s: &str| h.texts.iter().find(|p| p.text == s).map(|p| p.rect).unwrap_or_else(|| panic!("no caption {s}"));
    let survey = caption(&h, "Survey Data · Same Layers as Strata");
    let drawing = caption(&h, "Drawing · Trowel");
    assert!(!h.texts.iter().any(|p| p.text == "This drawing's layers"));
    let top = |id: NodeId| h.rect(wid("legend-row", id.0)).unwrap_or_else(|| panic!("row {id:?} laid out")).top();
    let (strata, other): (Vec<NodeId>, Vec<NodeId>) = added.iter().partition(|id| legend(**id).strata.is_some());
    for id in &strata {
        assert!(survey.bottom() < top(*id) && top(*id) < drawing.top(), "{id:?} in the Strata section");
    }
    for id in other.iter().chain([&first]) {
        assert!(top(*id) > drawing.bottom(), "{id:?} in the drawing's section");
    }
    // Each section: favourites first, then the rest, both in the group's order (its first row on
    // top); the layer without a row last.
    let want = |rows: Vec<(&str, vectorcraft_doc::Legend)>| -> Vec<String> {
        let (fav, rest): (Vec<_>, Vec<_>) = rows.into_iter().partition(|(_, l)| l.favourite);
        fav.into_iter().chain(rest).map(|(n, _)| n.to_string()).collect()
    };
    let plan = vectorcraft_engine::cmd::legend::group_rows(vectorcraft_doc::LegendGroup::Plan);
    let name = |id: &NodeId| doc.node(*id).unwrap().display_name();
    let shown = |mut ids: Vec<NodeId>| -> Vec<String> {
        ids.sort_by(|a, b| top(*a).total_cmp(&top(*b)));
        ids.iter().map(name).collect()
    };
    assert_eq!(shown(strata), want(plan.iter().filter(|(_, l)| l.strata.is_some()).cloned().collect()));
    let mut rest = want(plan.iter().filter(|(_, l)| l.strata.is_none()).cloned().collect());
    rest.push("Layer 1".to_string());
    assert_eq!(shown(other.into_iter().chain([first]).collect()), rest);
}

/// A click on a legend row makes it the layer drawn on and gives the pen its look; a layer without a
/// row leaves the pen as it is (criterion 13).
#[test]
fn clicking_a_row_picks_it_for_the_pen() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let section = legend_row(&mut h, "Section line", json!({"weight": 0.2, "line": "dashdot"}));
    h.settle();
    h.click(wid("legend-row", first.0));
    assert_eq!(h.current_layer(), Some(first));
    assert_eq!(h.app.session.new_art_stroke().width, 1.0, "no look: the pen as it was");
    h.click(wid("legend-row", section.0));
    assert_eq!(h.current_layer(), Some(section));
    let pen = h.app.session.new_art_stroke();
    assert_eq!(pen.width, 0.2);
    assert_eq!(pen.dash.map(|d| d.pattern), vectorcraft_doc::LineType::DashDot.dash_pt());
}

/// With a selection tool in hand, a click on a row moves the selected objects into it and gives
/// their lines its look, one undo step (criterion 13).
#[test]
fn clicking_a_row_with_the_selection_tool_moves_the_selection() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let line = draw_rect(&mut h, 10.0);
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.app.select_tool("selection");
    h.settle();
    h.click(wid("legend-row", pits.0));
    let layer_of = |h: &Harness| h.app.session.active().unwrap().doc.layer_containing(line);
    assert_eq!(layer_of(&h), Some(pits));
    assert_eq!(weight_of(&h, line), 0.8);
    assert_eq!(h.current_layer(), Some(pits));
    assert!(h.status_line().contains("Moved 1 to Pits"), "{}", h.status_line());
    h.app.run("edit.undo", json!({})).unwrap();
    assert_eq!(layer_of(&h), Some(first));
    assert_eq!(weight_of(&h, line), 1.0);
}

/// With a drawing tool in hand the line just drawn stays selected; a click on another row to draw
/// the next line with leaves it where it is (criterion 13, decision 1).
#[test]
fn clicking_a_row_while_drawing_does_not_move_the_new_line() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("layer.setCurrent", json!({ "id": first.0 })).unwrap();
    h.app.select_tool("pen");
    let line = draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.settle();
    h.click(wid("legend-row", pits.0));
    assert_eq!(h.current_layer(), Some(pits));
    assert_eq!(h.app.session.active().unwrap().doc.layer_containing(line), Some(first));
    assert_eq!(weight_of(&h, line), 1.0);
}

/// An open row lists its objects; a line set apart shows its weight and ↺ at the right, which puts
/// it back to the row's look (criterion 14).
#[test]
fn object_rows_show_overrides_and_revert() {
    let mut h = Harness::new();
    let t = h.tokens();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.8})).unwrap();
    let a = draw_rect(&mut h, 10.0);
    let b = draw_rect(&mut h, 40.0);
    h.app.run("select.set", json!({ "ids": [b.0] })).unwrap();
    h.app.run("stroke.set", json!({"weight": 0.5})).unwrap();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.click(wid("legend-open", pits.0));
    let (ra, rb) = (h.rect(wid("legend-obj", a.0)).unwrap(), h.rect(wid("legend-obj", b.0)).unwrap());
    assert!(!h.line_in(ra).contains("Apart"), "{}", h.line_in(ra));
    let badge = h.texts.iter().find(|p| p.text == "0.5 Apart ↺" && rb.contains_rect(p.rect)).expect("b's badge").rect;
    assert_eq!(h.color_of("0.5 Apart ↺", rb), Some(t.caution));
    assert!(badge.right() > rb.right() - 12.0, "at the row's right: {badge:?} in {rb:?}");
    h.click(wid("legend-revert", b.0));
    assert_eq!(weight_of(&h, b), 0.8);
    assert!(h.rect(wid("legend-revert", b.0)).is_none(), "no longer apart");
    assert!(!h.line_in(h.rect(wid("legend-row", pits.0)).unwrap()).contains("Apart"));
    h.app.run("edit.undo", json!({})).unwrap();
    assert_eq!(weight_of(&h, b), 0.5, "one undo step");
}

/// The head's group button adds a legend group's rows, then names the group most rows come from
/// (criterion 15).
#[test]
fn the_group_menu_adds_a_legend_group() {
    let mut h = Harness::new();
    let button = wid("legend-group", ());
    assert!(h.texts_in(h.rect(button).expect("the group button")).contains(&"Add a Legend".to_string()));
    h.click(button);
    let items = h.popup_buttons();
    assert_eq!(items.len(), 3, "{items:?}");
    let shown: String = h.texts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("|");
    for want in ["Add the Plan Legend", "Add the Section and Elevation Legend", "Add the Finds Legend"] {
        assert!(shown.contains(want), "{want} in {shown}");
    }
    h.click_at(items[0].0.center());
    assert_eq!(h.layer_ids().len(), 16);
    assert!(h.status_line().contains("Legend rows added: 15"), "{}", h.status_line());
    assert!(h.texts_in(h.rect(button).unwrap()).contains(&"For Plans".to_string()), "{:?}", h.texts_in(h.rect(button).unwrap()));
    // Again: every row is there already.
    h.click(button);
    let items = h.popup_buttons();
    h.click_at(items[0].0.center());
    assert_eq!(h.layer_ids().len(), 16);
    assert!(h.status_line().contains("No legend rows to add"), "{}", h.status_line());
    // The finds group's 7 rows are fewer than the plan's 15: still For Plans.
    h.click(button);
    let items = h.popup_buttons();
    h.click_at(items[2].0.center());
    assert_eq!(h.layer_ids().len(), 23);
    assert!(h.texts_in(h.rect(button).unwrap()).contains(&"For Plans".to_string()));
}

/// A group added from the group menu comes with its first ★ row as the one drawn with, so the
/// first lines go into the legend rather than the drawing's empty first layer; with nothing added
/// the row drawn with stays (UI review 2 U25).
#[test]
fn a_group_added_from_the_menu_draws_with_its_first_favourite() {
    let mut h = Harness::new();
    let current = |h: &Harness| h.current_layer().and_then(|id| h.app.session.active().unwrap().doc.node(id).map(|n| n.display_name()));
    for (item, want) in [(0, "굴광선 (유구 면)"), (1, "단면선 (잘린 땅)"), (2, "외형선")] {
        h.click(wid("legend-group", ()));
        let items = h.popup_buttons();
        h.click_at(items[item].0.center());
        assert_eq!(current(&h).as_deref(), Some(want));
    }
    let line = draw_rect(&mut h, 10.0);
    assert_eq!(weight_of(&h, line), 0.8, "the pen takes the row's look");
    h.click(wid("legend-group", ()));
    let items = h.popup_buttons();
    h.click_at(items[0].0.center());
    assert!(h.status_line().contains("No legend rows to add"), "{}", h.status_line());
    assert_eq!(current(&h).as_deref(), Some("외형선"));
}

/// Under the list, Weights at Once: "All Weights ×0.8 …" opens its card, and "Apart N → Legend"
/// puts every line set apart back to its row's look, dimmed when none is (criterion 16).
#[test]
fn the_footer_opens_the_weights_card_and_reverts_all_overrides() {
    let mut h = Harness::new();
    let t = h.tokens();
    let revert = wid("legend-overrides", ());
    let foot = |h: &Harness| h.rect(revert).expect("Apart N → Legend");
    assert_eq!(h.texts_in(foot(&h)), ["Apart 0 → Legend"]);
    assert_eq!(h.color_of("Apart 0 → Legend", foot(&h)), Some(t.text_disabled), "nothing to revert");
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.5})).unwrap();
    let (a, b) = (draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0));
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    assert_eq!(h.texts_in(foot(&h)), ["Apart 2 → Legend"]);
    assert_eq!(h.color_of("Apart 2 → Legend", foot(&h)), Some(t.text));
    h.click(revert);
    assert_eq!((weight_of(&h, a), weight_of(&h, b)), (0.8, 0.8));
    assert!(h.status_line().contains("Lines back to the legend: 2"), "{}", h.status_line());
    assert_eq!(h.texts_in(foot(&h)), ["Apart 0 → Legend"]);
    let weights = wid("legend-weights", ());
    assert_eq!(h.texts_in(h.rect(weights).unwrap()), ["All Weights ×0.8 …"]);
    h.click(weights);
    assert!(h.rect(wid("weights-card", ())).is_some(), "the card opens");
}

/// When the legend's rows overflow its list, the rows at its edges show as plainly as the others:
/// no fade (egui fades a scroll area's edges) that would read as a hidden layer (criterion 21, F2).
#[test]
fn the_legend_list_does_not_fade_its_last_row() {
    let mut h = Harness::new();
    for i in 0..30 {
        h.app.run("layer.new", json!({ "name": format!("Pit {i}") })).unwrap();
    }
    h.settle();
    let top_row = h.rect(wid("legend-row", h.layer_ids().last().unwrap().0)).unwrap();
    let line = h.rect(wid("legend-weights", ())).unwrap().top() - 6.0;
    let list = Rect::from_min_max(pos2(top_row.left(), top_row.top() - 40.0), pos2(top_row.right(), line));
    assert!(h.gradients_over(list).is_empty(), "at the top: {:?}", h.gradients_over(list));
    h.wheel(top_row.center(), vec2(0.0, -200.0));
    assert!(h.gradients_over(list).is_empty(), "scrolled: {:?}", h.gradients_over(list));
}

// ---------- J005: the row editor and Weights at Once ----------

/// Double-clicking a legend row opens its editor, a card beside the right column level with the
/// row; ✕, Esc and a press outside it close it (criterion 17).
#[test]
fn double_clicking_a_row_opens_its_editor() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pit outline", json!({"weight": 0.8, "strata": "feature_poly"}));
    h.settle();
    let (card, row) = (wid("row-card", ()), wid("legend-row", pits.0));
    assert!(h.rect(card).is_none());
    h.double_click(row);
    let c = h.rect(card).expect("the editor");
    let (right, r, status) = (h.panel(RIGHT), h.rect(row).unwrap(), h.panel(STATUS));
    assert!((c.width() - 304.0).abs() < 1.0 && c.right() <= right.left() && right.left() - c.right() <= 12.0, "{c:?} beside {right:?}");
    // Level with the row, or pushed up to stay above the status bar.
    assert!((c.top() - r.top()).abs() < 1.0 || (c.bottom() - status.top()).abs() < 1.0, "level with the row: {c:?}, {r:?}");
    assert!(c.bottom() <= status.top() + 0.5, "{c:?} over the status bar {status:?}");
    let line = h.line_in(c);
    for want in [
        "Legend Row",
        "Pit outline",
        "Name",
        "유구_면 · feature_poly",
        "This row is the layer",
        "Weight",
        "= 0.28 mm",
        "Solid",
        "Dash-dot",
        "Often Used",
        "This row's lines",
        "Select All",
        "Back to the Legend",
        "Saved in this drawing file for now",
        "Layer Options…",
    ] {
        assert!(line.contains(want), "{want} in {line}");
    }
    h.click(wid("card-close", ()));
    assert!(h.rect(card).is_none(), "✕ closes it");
    // Right away: egui alone would take the row's second click for a third one.
    h.double_click(row);
    assert!(h.rect(card).is_some(), "it opens again right after ✕");
    h.key(egui::Key::Escape);
    assert!(h.rect(card).is_none(), "Esc closes it");
    h.double_click(row);
    assert!(h.rect(card).is_some());
    h.click_at(h.app.canvas_rect.unwrap().center());
    assert!(h.rect(card).is_none(), "a press outside closes it");
}

/// The editor changes its row: a weight chip restyles the row's lines that followed it (one set
/// apart keeps its own), and so do the number field and the line types; ★, Select All, Back to the
/// Legend, the name and No Weight work on the row (criterion 17).
#[test]
fn the_row_editor_changes_the_row_and_its_lines() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.8})).unwrap();
    let (a, b, c) = (draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0), draw_rect(&mut h, 70.0));
    h.app.run("select.set", json!({ "ids": [c.0] })).unwrap();
    h.app.run("stroke.set", json!({"weight": 0.3})).unwrap();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    let legend = |h: &Harness| h.app.session.active().unwrap().doc.node(pits).unwrap().legend().cloned().unwrap();
    h.click(wid("card-weight", "0.5"));
    assert_eq!(legend(&h).weight, Some(0.5));
    assert_eq!((weight_of(&h, a), weight_of(&h, b), weight_of(&h, c)), (0.5, 0.5, 0.3));
    assert!(h.status_line().contains("Lines changed: 2"), "{}", h.status_line());
    h.type_into(wid("card-pt", ()), "0.6");
    assert_eq!(legend(&h).weight, Some(0.6));
    assert_eq!((weight_of(&h, a), weight_of(&h, c)), (0.6, 0.3));
    assert!(h.line_in(h.rect(wid("row-card", ())).unwrap()).contains("= 0.21 mm"));
    h.click(wid("card-line", "dashed"));
    assert_eq!(legend(&h).line, vectorcraft_doc::LineType::Dashed);
    let dash =
        |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().appearance.stroke().unwrap().dash.clone().map(|d| d.pattern);
    assert_eq!(dash(&h, a), vectorcraft_doc::LineType::Dashed.dash_pt());
    h.click(wid("card-favourite", ()));
    assert!(legend(&h).favourite);
    h.click(wid("card-select", ()));
    let mut selected = h.app.session.active().unwrap().selection.objects.clone();
    selected.sort();
    assert_eq!(selected, [a, b, c]);
    h.click(wid("card-revert", ()));
    assert_eq!((weight_of(&h, c), dash(&h, c)), (0.6, vectorcraft_doc::LineType::Dashed.dash_pt()));
    h.type_into(wid("card-name", ()), "Pit outlines");
    assert_eq!(h.app.session.active().unwrap().doc.node(pits).unwrap().display_name(), "Pit outlines");
    h.click(wid("card-weight", "none"));
    assert_eq!(legend(&h).weight, None);
    assert_eq!(weight_of(&h, a), 0.6, "no look: the lines stay");
}

/// The rows Weights at Once's card shows: those of `plan`, the ones changing alike together.
fn plan_table(h: &Harness, params: serde_json::Value) -> Vec<(String, String, String)> {
    let plan = vectorcraft_engine::cmd::legend::plan_weights(&h.app.session.active().unwrap().doc, &params).unwrap();
    let mut rows: Vec<(String, String, String)> = vec![];
    for row in &plan.rows {
        let (from, to) = (super::legend::pt(row.from), super::legend::pt(row.to));
        match rows.iter_mut().find(|r| r.1 == from && r.2 == to) {
            Some(r) => r.0 = format!("{} · {}", r.0, row.name),
            None => rows.push((row.name.clone(), from, to)),
        }
    }
    rows
}

/// Weights at Once's card previews the change as the engine plans it (the rows that change alike
/// together) and Change applies it, one undo step (criterion 18).
#[test]
fn weights_card_previews_and_applies_a_ratio() {
    let mut h = Harness::sized(1600.0, 1400.0);
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    let doc = h.app.session.active().unwrap().doc.clone();
    let named = |name: &str| doc.layers.iter().find(|l| l.display_name() == name).unwrap().id;
    let pit = named("굴광선 (유구 면)");
    h.app.run("legend.pick", json!({ "id": pit.0 })).unwrap();
    let line = draw_rect(&mut h, 10.0);
    assert_eq!(weight_of(&h, line), 0.8);
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    h.click(wid("legend-weights", ()));
    let card = h.rect(wid("weights-card", ())).expect("the card");
    let line_in = h.line_in(card);
    for want in ["Weights at Once", "This drawing", "Ratio", "Change Values", "One Step", "Lines set apart too", "Change", "Close"] {
        assert!(line_in.contains(want), "{want} in {line_in}");
    }
    assert_eq!(h.texts_in(h.rect(wid("weights-ratio", ())).unwrap()), ["×", "0.8"]);
    // The table, as the engine plans it.
    let want = plan_table(&h, json!({"how": "ratio", "ratio": 0.8}));
    assert!(want.len() >= 4, "{want:?}");
    for (names, from, to) in &want {
        let cell = h.texts.iter().find(|p| p.text == *names && card.contains_rect(p.rect)).unwrap_or_else(|| panic!("{names} in {line_in}")).rect;
        let row: Vec<String> = h
            .texts
            .iter()
            .filter(|p| card.contains_rect(p.rect) && (p.rect.center().y - cell.center().y).abs() < 2.0)
            .map(|p| p.text.clone())
            .collect();
        assert!(row.contains(from) && row.contains(to), "{names}: {from} → {to} in {row:?}");
    }
    let undo = if cfg!(target_os = "macos") { "⌘Z" } else { "Ctrl+Z" };
    assert!(line_in.contains(&format!("Lines that change: 1. One {undo} puts them all back.")), "{line_in}");
    h.click(wid("weights-apply", ()));
    let legend = |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().legend().cloned().unwrap();
    assert_eq!(legend(&h, pit).weight, Some(0.64));
    assert_eq!(weight_of(&h, line), 0.64);
    assert_eq!(legend(&h, named("조사구역 경계")).weight, Some(0.8));
    h.app.run("edit.undo", json!({})).unwrap();
    assert_eq!((legend(&h, pit).weight, weight_of(&h, line)), (Some(0.8), 0.8), "one undo step");
}

/// The card's other ways: Change Values (only rows at the first weight take the second) and One
/// Step (thinner by default, or thicker), with lines set apart left unless asked (criterion 18).
#[test]
fn weights_card_value_and_step_modes() {
    let mut h = Harness::sized(1600.0, 1400.0);
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    let doc = h.app.session.active().unwrap().doc.clone();
    let named = |name: &str| doc.layers.iter().find(|l| l.display_name() == name).unwrap().id;
    let (floor, pit, boundary) = (named("바닥선"), named("굴광선 (유구 면)"), named("조사구역 경계"));
    // A line on the floor row set apart at 0.2.
    h.app.run("legend.pick", json!({ "id": floor.0 })).unwrap();
    let apart = draw_rect(&mut h, 10.0);
    h.app.run("stroke.set", json!({"weight": 0.2})).unwrap();
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    let legend = |h: &Harness, id: NodeId| h.app.session.active().unwrap().doc.node(id).unwrap().legend().cloned().unwrap().weight;
    h.click(wid("legend-weights", ()));
    h.click(wid("weights-how", "value"));
    h.type_into(wid("weights-from", ()), "0.5");
    h.type_into(wid("weights-to", ()), "0.4");
    let card = h.rect(wid("weights-card", ())).unwrap();
    let want = plan_table(&h, json!({"how": "value", "from": 0.5, "to": 0.4}));
    assert_eq!(want.len(), 1, "{want:?}");
    assert!(h.texts.iter().any(|p| p.text == want[0].0 && card.contains_rect(p.rect)), "{want:?} in {}", h.line_in(card));
    h.click(wid("weights-apply", ()));
    assert_eq!((legend(&h, floor), legend(&h, pit)), (Some(0.4), Some(0.8)));
    assert_eq!(weight_of(&h, apart), 0.2, "set apart: left");
    // One step: thinner by default; thicker, with the lines set apart too. Change puts the card
    // away: it opens again for each.
    h.click(wid("legend-weights", ()));
    h.click(wid("weights-how", "step"));
    h.click(wid("weights-apply", ()));
    assert_eq!((legend(&h, floor), legend(&h, pit), legend(&h, boundary)), (Some(0.3), Some(0.5), Some(0.8)));
    assert_eq!(weight_of(&h, apart), 0.2, "set apart: left");
    h.click(wid("legend-weights", ()));
    h.click(wid("weights-how", "step"));
    h.click(wid("weights-step", "up"));
    h.click(wid("weights-overrides", ()));
    h.click(wid("weights-apply", ()));
    assert_eq!((legend(&h, floor), legend(&h, pit), legend(&h, boundary)), (Some(0.5), Some(0.8), Some(1.0)));
    assert_eq!(weight_of(&h, apart), 0.3, "set apart, with Lines set apart too");
}

// ---------- J005: the drawing palette, the Drawing menu and the Control bar ----------

/// The palette's line types run (criterion 19): with nothing selected they set the next line's
/// dashes, with lines selected theirs (Solid takes them off) and the pen stays as it was; the
/// first selected line's type shows on, none when the selected lines' dashes differ.
#[test]
fn palette_line_types_set_the_dash() {
    use vectorcraft_doc::LineType;
    let mut h = Harness::new();
    let t = h.tokens();
    let on = |h: &Harness, key: &str| h.fill_at(h.rect(wid("dash", key)).unwrap()) == Some(t.row_selected);
    assert!(on(&h, "solid") && !on(&h, "dashed"));
    for (key, want) in [("dashed", LineType::Dashed), ("dotted", LineType::Dotted), ("dashdot", LineType::DashDot), ("solid", LineType::Solid)] {
        h.click(wid("dash", key));
        assert_eq!(LineType::of(&h.app.session.new_art_stroke()), want, "the next line, {key}");
        assert!(on(&h, key), "{key} shows on");
    }
    let (a, b) = (draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0));
    h.app.run("select.set", json!({ "ids": [a.0, b.0] })).unwrap();
    h.settle();
    h.click(wid("dash", "dotted"));
    let line_of = |h: &Harness, id: NodeId| LineType::of(h.app.session.active().unwrap().doc.node(id).unwrap().appearance.stroke().unwrap());
    assert_eq!((line_of(&h, a), line_of(&h, b)), (LineType::Dotted, LineType::Dotted));
    assert_eq!(LineType::of(&h.app.session.new_art_stroke()), LineType::Solid, "the pen as it was");
    assert!(on(&h, "dotted") && !on(&h, "solid"));
    // Dashes that differ: none on.
    h.app.run("select.set", json!({ "ids": [b.0] })).unwrap();
    h.app.run("stroke.set", json!({ "dash": null })).unwrap();
    h.app.run("select.set", json!({ "ids": [a.0, b.0] })).unwrap();
    h.settle();
    assert!(["solid", "dashed", "dotted", "dashdot"].iter().all(|k| !on(&h, k)), "mixed: none on");
}

/// The chosen line type's ink edge shows on all four sides: nothing is drawn over it after it, not
/// the box's edge or the lines between the parts (criterion 19, F1 of J003's review).
#[test]
fn the_chosen_line_type_chip_keeps_its_border() {
    use vectorcraft_doc::LineType;
    let mut h = Harness::new();
    let t = h.tokens();
    let ink = egui::Stroke::new(1.0, t.text);
    for (key, kind) in [("dashed", LineType::Dashed), ("solid", LineType::Solid), ("dashdot", LineType::DashDot)] {
        h.app.run("stroke.set", json!({ "dash": kind.dash_pt() })).unwrap();
        h.settle();
        let part = h.rect(wid("dash", key)).unwrap();
        let shapes: Vec<&egui::Shape> = h.shapes.iter().map(|c| &c.shape).collect();
        let at = shapes.iter().rposition(|s| {
            matches!(s, egui::Shape::Rect(r) if (r.rect.min - part.min).length() < 0.6 && (r.rect.max - part.max).length() < 0.6 && r.stroke == ink)
        });
        let at = at.unwrap_or_else(|| panic!("{key}: no ink edge at {part:?}"));
        let edge = part.expand(0.5);
        let over: Vec<&&egui::Shape> = shapes
            .iter()
            .skip(at + 1)
            .filter(|s| match s {
                egui::Shape::Rect(r) => r.rect.intersects(edge),
                egui::Shape::LineSegment { points, .. } => Rect::from_two_pos(points[0], points[1]).intersects(edge),
                _ => false,
            })
            .collect();
        assert!(over.is_empty(), "{key}: drawn over its edge: {over:?}");
    }
}

/// Add to Legend makes a row of the selected line's look (the pen's when nothing is selected),
/// moves the line into it and opens the row's editor; Back to the Legend puts the selected lines
/// back to their row's look, or with nothing selected gives the pen the current row's look
/// (criterion 19).
#[test]
fn palette_legend_buttons_add_and_revert() {
    use vectorcraft_doc::LineType;
    let mut h = Harness::new();
    let t = h.tokens();
    let (add, back) = (wid("palette-to-legend", ()), wid("palette-from-legend", ()));
    for id in [add, back] {
        assert_ne!(h.fill_at(h.rect(id).unwrap()), Some(t.panel_darker), "{id:?} runs");
    }
    let first = h.layer_ids()[0];
    let line = draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.app.run("stroke.set", json!({ "weight": 0.35, "dash": LineType::Dashed.dash_pt() })).unwrap();
    h.settle();
    h.click(add);
    let row = h.current_layer().unwrap();
    assert_ne!(row, first);
    let doc = h.app.session.active().unwrap().doc.clone();
    let node = doc.node(row).unwrap();
    assert_eq!(node.display_name(), "Legend Row 1");
    let legend = node.legend().cloned().unwrap();
    assert_eq!((legend.weight, legend.line), (Some(0.35), LineType::Dashed));
    assert_eq!(doc.layer_containing(line), Some(row), "the line moves into its row");
    let card = h.rect(wid("row-card", ())).expect("the row's editor opens");
    assert!(h.line_in(card).contains("Legend Row 1"), "{}", h.line_in(card));
    // The row's look changes, the line is set apart, then put back.
    h.app.run("legend.set", json!({ "id": row.0, "weight": 0.5 })).unwrap();
    assert_eq!(weight_of(&h, line), 0.5);
    h.app.run("stroke.set", json!({ "weight": 0.2 })).unwrap();
    h.settle();
    h.click(back);
    assert_eq!(weight_of(&h, line), 0.5);
    assert!(h.status_line().contains("Lines back to the legend: 1"), "{}", h.status_line());
    // Nothing selected: the pen takes the current row's look.
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.app.run("stroke.set", json!({ "weight": 0.1, "dash": null })).unwrap();
    h.settle();
    h.click(back);
    let pen = h.app.session.new_art_stroke();
    assert_eq!((pen.width, LineType::of(&pen)), (0.5, LineType::Dashed));
    assert!(h.status_line().contains("The next line takes Legend Row 1's look"), "{}", h.status_line());
    // The next row takes the next number.
    h.click(add);
    let name = h.app.session.active().unwrap().doc.node(h.current_layer().unwrap()).unwrap().display_name();
    assert_eq!(name, "Legend Row 2");
}

/// The Drawing menu's first two items run (criterion 20): Change Weights at Once… opens its card,
/// Revert to the Legend puts the selected lines back to their row's look, or every line set apart
/// when nothing is selected. The others still wait for their jobs.
#[test]
fn drawing_menu_opens_the_weights_card_and_reverts() {
    let mut h = Harness::new();
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.5})).unwrap();
    let (a, b) = (draw_rect(&mut h, 10.0), draw_rect(&mut h, 40.0));
    h.settle();
    let bar = h.panel(MENU);
    let open = |h: &mut Harness| {
        let at = h.texts.iter().find(|p| p.text == "Drawing" && bar.contains_rect(p.rect)).unwrap().rect.center();
        h.click_at(at);
        h.popup_buttons()
    };
    let items = open(&mut h);
    let enabled: Vec<bool> = items.iter().map(|(_, e)| *e).collect();
    assert_eq!(enabled, [true, true, true, true, true, true, true, false, false, false]);
    h.click_at(h.texts.iter().find(|p| p.text == "Drawing" && bar.contains_rect(p.rect)).unwrap().rect.center());
    h.app.run("select.set", json!({ "ids": [b.0] })).unwrap();
    h.settle();
    let items = open(&mut h);
    h.click_at(items[1].0.center());
    assert_eq!((weight_of(&h, a), weight_of(&h, b)), (0.5, 0.8), "the selected line");
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    let items = open(&mut h);
    h.click_at(items[1].0.center());
    assert_eq!(weight_of(&h, a), 0.8, "nothing selected: every line set apart");
    assert!(h.status_line().contains("Lines back to the legend: 1"), "{}", h.status_line());
    let items = open(&mut h);
    h.click_at(items[0].0.center());
    assert!(h.rect(wid("weights-card", ())).is_some(), "Weights at Once opens");
}

/// The Control bar's legend field lists the rows with their look (a layer without a row, its
/// colour) and shows the chosen one's; picking one there makes it the layer drawn on with its
/// look and moves nothing (criterion 13).
#[test]
fn control_bar_lists_rows_with_their_look() {
    use vectorcraft_doc::LineType;
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8, "line": "dashed"}));
    h.app.run("layer.setCurrent", json!({ "id": first.0 })).unwrap();
    h.app.select_tool("pen");
    let line = draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.settle();
    let colour = |h: &Harness, id: NodeId| crate::panels::layers::layer_colour(&h.app.session.active().unwrap().doc, id);
    let patch_in = |h: &Harness, area: Rect, id: NodeId| h.rects_in(area).iter().any(|(_, _, s)| s.color == colour(h, id));
    h.click(wid("ctl-layer", ()));
    assert!(patch_in(&h, h.rect(wid("ctl-layer-row", first.0)).unwrap(), first), "a layer without a row: its colour");
    assert!(!patch_in(&h, h.rect(wid("ctl-layer-row", pits.0)).unwrap(), pits), "a row: its look");
    h.click(wid("ctl-layer-row", pits.0));
    assert_eq!(h.current_layer(), Some(pits));
    let pen = h.app.session.new_art_stroke();
    assert_eq!((pen.width, LineType::of(&pen)), (0.8, LineType::Dashed));
    assert_eq!(h.app.session.active().unwrap().doc.layer_containing(line), Some(first), "nothing moves");
    assert!(!patch_in(&h, h.rect(wid("ctl-layer", ())).unwrap(), pits), "the field shows the row's look");
}

// ---------- J005 rework 1: what the code and UI reviews found ----------

/// The legend weight of row `id`.
fn row_weight(h: &Harness, id: NodeId) -> Option<f64> {
    h.app.session.active().unwrap().doc.node(id).unwrap().legend().and_then(|l| l.weight)
}

/// A ratio typed into Weights at Once and Change pressed straight after (no Enter) changes the
/// weights by what was typed; a ratio out of range is refused, says so and changes nothing
/// (code review C1).
#[test]
fn weights_card_changes_by_the_ratio_just_typed() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.click(wid("legend-weights", ()));
    h.type_over(wid("weights-ratio", ()), "0.5");
    h.click(wid("weights-apply", ()));
    assert_eq!(row_weight(&h, pits), Some(0.4), "×0.5: {}", h.status_line());
    h.click(wid("legend-weights", ()));
    h.type_over(wid("weights-ratio", ()), "20");
    h.click(wid("weights-apply", ()));
    assert_eq!(row_weight(&h, pits), Some(0.4), "refused: nothing changes");
    assert!(h.status_line().contains("The ratio is between 0.1 and 10"), "{}", h.status_line());
}

/// The texts on the line of Weights at Once's table that names `name`.
fn table_line(h: &Harness, name: &str) -> Vec<String> {
    let card = h.rect(wid("weights-card", ())).expect("the card");
    let cell = h.texts.iter().find(|p| p.text == name && card.contains_rect(p.rect)).unwrap_or_else(|| panic!("{name} in {}", h.line_in(card))).rect;
    h.texts.iter().filter(|p| card.contains_rect(p.rect) && (p.rect.center().y - cell.center().y).abs() < 2.0).map(|p| p.text.clone()).collect()
}

/// The table follows a ratio being typed, before Enter: what it shows is what Change does (UI
/// review 2, the rest of code review C1).
#[test]
fn the_weights_table_follows_the_ratio_being_typed() {
    let mut h = Harness::new();
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.click(wid("legend-weights", ()));
    assert!(table_line(&h, "Pits").contains(&"0.64".to_string()), "{:?}", table_line(&h, "Pits"));
    h.type_over(wid("weights-ratio", ()), "0.5");
    h.settle();
    assert!(table_line(&h, "Pits").contains(&"0.4".to_string()), "×0.5 as typed: {:?}", table_line(&h, "Pits"));
}

/// Change does the change and puts the card away, the status bar saying what changed: a second
/// press can't change the weights again unseen (UI review 2 U26).
#[test]
fn the_weights_card_closes_after_change() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.click(wid("legend-weights", ()));
    h.click(wid("weights-apply", ()));
    assert_eq!(row_weight(&h, pits), Some(0.64));
    assert!(h.rect(wid("weights-card", ())).is_none(), "the card is put away");
    assert!(h.status_line().contains("Rows changed: 1 · lines: 0"), "{}", h.status_line());
}

/// Rows changing with no line following them: the card's note counts the rows rather than saying
/// that no line changes (UI review 2 U29).
#[test]
fn the_weights_note_counts_rows_when_no_line_follows() {
    let mut h = Harness::new();
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.click(wid("legend-weights", ()));
    let note = h.line_in(h.rect(wid("weights-card", ())).unwrap());
    let undo = if cfg!(target_os = "macos") { "⌘Z" } else { "Ctrl+Z" };
    assert!(note.contains(&format!("Rows that change: 1. No line follows them. One {undo} puts them back.")), "{note}");
    assert!(!note.contains("Lines that change"), "{note}");
}

/// A weight chip pressed while points are being typed in the row's editor wins, one undo step
/// (code review C6).
#[test]
fn a_weight_chip_pressed_while_typing_wins() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    h.type_over(wid("card-pt", ()), "0.4");
    h.click(wid("card-weight", "0.5"));
    assert_eq!(row_weight(&h, pits), Some(0.5));
    h.app.run("edit.undo", json!({})).unwrap();
    assert_eq!(row_weight(&h, pits), Some(0.8), "one undo step");
}

/// Esc while a card's field has the keyboard puts back what was typed and keeps the card; the next
/// Esc puts the card away (code review C7, UI review U5).
#[test]
fn esc_in_a_card_field_keeps_the_card() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    h.type_over(wid("card-name", ()), "Postholes");
    h.key(egui::Key::Escape);
    assert!(h.rect(wid("row-card", ())).is_some(), "the card stays");
    assert_eq!(h.app.session.active().unwrap().doc.node(pits).unwrap().display_name(), "Pits", "what was typed is put back");
    h.key(egui::Key::Escape);
    assert!(h.rect(wid("row-card", ())).is_none(), "the next Esc puts it away");
}

/// Weights at Once opens as it starts (by ratio, ×0.8) however it was left, even when another
/// drawing came to the front while it was open (code review C8).
#[test]
fn weights_card_opens_afresh() {
    let mut h = Harness::new();
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.click(wid("legend-weights", ()));
    h.click(wid("weights-how", "value"));
    assert!(h.rect(wid("weights-from", ())).is_some());
    h.app.run("file.new", json!({"name": "Site 2", "width": 400, "height": 300})).unwrap();
    h.settle();
    assert!(h.rect(wid("weights-card", ())).is_none(), "it goes with its drawing");
    h.click(wid("legend-weights", ()));
    let ratio = h.rect(wid("weights-ratio", ())).expect("by ratio");
    assert_eq!(h.texts_in(ratio), ["×", "0.8"]);
    assert!(h.rect(wid("weights-from", ())).is_none());
}

/// A press on the canvas while a card is open only puts the card away: the tool in hand doesn't
/// get it (UI review U1).
#[test]
fn a_press_on_the_canvas_only_puts_a_card_away() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.select_tool("pen");
    h.settle();
    let nodes = |h: &Harness| h.app.session.active().unwrap().doc.node_count();
    let n = nodes(&h);
    let canvas = h.app.canvas_rect.unwrap();
    h.double_click(wid("legend-row", pits.0));
    assert!(h.rect(wid("row-card", ())).is_some());
    h.click_at(canvas.center());
    assert!(h.rect(wid("row-card", ())).is_none(), "the card is put away");
    assert_eq!(nodes(&h), n, "the Pen got nothing");
    h.click(wid("legend-weights", ()));
    assert!(h.rect(wid("weights-card", ())).is_some());
    h.click_at(canvas.center());
    assert!(h.rect(wid("weights-card", ())).is_none());
    assert_eq!(nodes(&h), n, "the Pen got nothing");
    h.click_at(canvas.center());
    assert!(nodes(&h) > n, "with no card the Pen draws");
}

/// The same with a touchpad's tap (a press and a release in one frame): it puts the card away and
/// the Pen gets nothing.
#[test]
fn a_tap_on_the_canvas_only_puts_a_card_away() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.select_tool("pen");
    h.settle();
    let nodes = |h: &Harness| h.app.session.active().unwrap().doc.node_count();
    let n = nodes(&h);
    let canvas = h.app.canvas_rect.unwrap();
    h.double_click(wid("legend-row", pits.0));
    assert!(h.rect(wid("row-card", ())).is_some());
    h.tap(canvas.center());
    h.settle();
    assert!(h.rect(wid("row-card", ())).is_none(), "the card is put away");
    assert_eq!(nodes(&h), n, "the Pen got nothing");
}

/// A double click on a row with a selection tool in hand and lines selected opens the row's editor
/// and leaves the lines where they were, as they were: the first click's move is taken back (code
/// review C2 option b, UI review U2).
#[test]
fn double_clicking_a_row_with_lines_selected_moves_nothing() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let line = draw_rect(&mut h, 10.0);
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
    h.app.select_tool("selection");
    h.settle();
    h.double_click(wid("legend-row", pits.0));
    assert!(h.rect(wid("row-card", ())).is_some(), "the editor opens");
    assert_eq!(h.app.session.active().unwrap().doc.layer_containing(line), Some(first), "the line stays");
    assert_eq!(weight_of(&h, line), 1.0, "with its own look");
    assert_eq!(h.app.session.active().unwrap().selection.objects, [line], "still selected");
    assert_eq!(h.current_layer(), Some(pits), "the row is the one drawn with");
    assert!(h.status_line().contains("The selected lines stay where they were"), "{}", h.status_line());
}

/// The first click of a double click moves the selected line out of an open row above the one
/// clicked, so the rows under the pointer go up before the second click: the double click still
/// opens the clicked row's editor and moves nothing, whether another row or nothing comes under
/// the pointer (code review 2 C16).
#[test]
fn double_clicking_a_row_moves_nothing_when_its_first_click_shifts_the_rows() {
    for bottom in [false, true] {
        let mut h = Harness::new();
        let first = h.layer_ids()[0];
        // The row clicked: one above Layer 1, or Layer 1 itself, the bottom row.
        let row = if bottom {
            h.app.run("legend.set", json!({"id": first.0, "weight": 0.8})).unwrap();
            first
        } else {
            legend_row(&mut h, "Pits", json!({"weight": 0.8}))
        };
        let walls = legend_row(&mut h, "Walls", json!({"weight": 0.5}));
        let line = draw_rect(&mut h, 10.0);
        h.settle();
        h.click(wid("legend-open", walls.0));
        h.app.run("select.set", json!({ "ids": [line.0] })).unwrap();
        h.app.select_tool("selection");
        h.settle();
        let name = h.app.session.active().unwrap().doc.node(row).unwrap().display_name();
        let undo = |h: &Harness| h.app.session.active().unwrap().history.undo.iter().map(|e| e.label.clone()).collect::<Vec<_>>();
        let before = undo(&h);
        let was = h.rect(wid("legend-row", row.0)).unwrap();
        // In the row's lower part, which the line's own row above it leaving brings 20 higher.
        let at = pos2(was.center().x, was.bottom() - 3.0);
        let click = |h: &mut Harness| {
            for pressed in [true, false] {
                h.frame(vec![Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Default::default() }]);
            }
        };
        h.frame(vec![Event::PointerMoved(at)]);
        click(&mut h);
        h.settle();
        assert!(!h.rect(wid("legend-row", row.0)).unwrap().contains(at), "bottom {bottom}: the rows went up under the pointer");
        click(&mut h);
        h.settle();
        let card = h.rect(wid("row-card", ())).unwrap_or_else(|| panic!("bottom {bottom}: no editor"));
        assert!(h.line_in(card).contains(&name), "bottom {bottom}: {name}'s editor: {}", h.line_in(card));
        assert_eq!(h.app.session.active().unwrap().doc.layer_containing(line), Some(walls), "bottom {bottom}: the line stays");
        assert_eq!(undo(&h), before, "bottom {bottom}: nothing done");
        assert_eq!(h.current_layer(), Some(row), "bottom {bottom}: the row is the one drawn with");
    }
}

/// Two clicks on a row up to half a second apart open its editor (a system's usual double click);
/// further apart they are two picks (UI review U10).
#[test]
fn a_slow_double_click_opens_the_editor() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    let row = wid("legend-row", pits.0);
    // The clicks' releases 0.35 s apart plus the wait.
    h.click(row);
    h.later(0.1);
    h.click(row);
    assert!(h.rect(wid("row-card", ())).is_some(), "0.45 s apart: the editor");
    h.click(wid("card-close", ()));
    h.later(1.0);
    h.click(row);
    h.later(0.25);
    h.click(row);
    assert!(h.rect(wid("row-card", ())).is_none(), "0.6 s apart: two picks");
}

/// A touchpad's tap comes as a press and a release in one frame. Two taps on a row open its
/// editor, and do so again after a card was shown and put away: the press of the tap that opens a
/// card doesn't put it away.
#[test]
fn two_taps_open_the_editor_again() {
    let mut h = Harness::new();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    let at = h.rect(wid("legend-row", pits.0)).unwrap().center();
    for round in 0..2 {
        h.tap(at);
        h.tap(at);
        h.settle();
        assert!(h.rect(wid("row-card", ())).is_some(), "round {round}: the editor opens");
        h.click(wid("card-close", ()));
        assert!(h.rect(wid("row-card", ())).is_none(), "round {round}: ✕ puts it away");
    }
    // With the editor open, Add to Legend's tap opens the new row's editor.
    h.tap(at);
    h.tap(at);
    h.settle();
    h.tap(h.rect(wid("palette-to-legend", ())).unwrap().center());
    h.settle();
    let card = h.rect(wid("row-card", ())).expect("the new row's editor");
    assert!(h.texts_in(card).iter().any(|s| s.starts_with("Legend Row ")), "{:?}", h.texts_in(card));
}

/// After the plan legend is added its Underlay row is there but empty: the Underlay tile still
/// places a scan (UI review U3, J003's tile).
#[test]
fn the_underlay_tile_places_a_scan_beside_the_empty_underlay_row() {
    let mut h = Harness::new();
    let dir = std::env::temp_dir().join(format!("trowel-underlay-row-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let scan = dir.join("scan.svg");
    std::fs::write(&scan, r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30"/></svg>"#).unwrap();
    let path = scan.to_string_lossy().to_string();
    h.app.services.pick_open = Some(Box::new(move |_| Some(path.clone())));
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    h.settle();
    let t = h.tokens();
    let tile = h.rect(wid("tile", "underlay")).unwrap();
    let well = Rect::from_min_size(pos2(tile.center().x - 22.0, tile.top()), vec2(44.0, 44.0));
    assert_ne!(h.fill_at(well), Some(t.row_selected), "no scan to show");
    h.click(wid("tile", "underlay"));
    let d = h.app.ui.dialog.as_ref().expect("the Place dialog");
    assert_eq!(d.kind, crate::dialogs::place::KIND);
    assert_eq!(d.fields.get("template"), Some(&json!(true)), "Template ticked");
    std::fs::remove_dir_all(&dir).ok();
}

/// The Underlay tile's scan goes under every line: on a locked template layer at the bottom of the
/// stack, named after the tile and the file, while the row drawn with stays (UI review 2 U24).
#[test]
fn the_underlay_tiles_scan_goes_under_every_line() {
    let mut h = Harness::new();
    let dir = std::env::temp_dir().join(format!("trowel-underlay-bottom-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let scan = dir.join("scan.svg");
    std::fs::write(&scan, r#"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="30"><rect width="40" height="30"/></svg>"#).unwrap();
    let path = scan.to_string_lossy().to_string();
    h.app.services.pick_open = Some(Box::new(move |_| Some(path.clone())));
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    let pit = h.app.session.active().unwrap().doc.layers.iter().find(|l| l.display_name() == "굴광선 (유구 면)").unwrap().id;
    h.app.run("legend.pick", json!({ "id": pit.0 })).unwrap();
    h.settle();
    h.click(wid("tile", "underlay"));
    crate::dialogs::confirm(&mut h.app).unwrap();
    h.settle();
    let doc = h.app.session.active().unwrap().doc.clone();
    let bottom = doc.layers.first().unwrap();
    assert_eq!(bottom.display_name(), "Underlay · scan.svg");
    assert!(bottom.is_template() && bottom.locked && bottom.children().is_some_and(|c| !c.is_empty()), "the scan, locked, on a template layer");
    assert_eq!(h.current_layer(), Some(pit), "the row drawn with stays");
    std::fs::remove_dir_all(&dir).ok();
}

/// The closed outlines painted inside `area` with three corners: where they are and their edge.
fn triangles_in(h: &Harness, area: Rect) -> Vec<(Rect, egui::epaint::PathStroke)> {
    h.shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Path(p) if p.closed && p.points.len() == 3 && p.points.iter().all(|q| area.contains(*q)) => {
                Some((Rect::from_points(&p.points), p.stroke.clone()))
            }
            _ => None,
        })
        .collect()
}

/// The mark samples are as large as the mockup's: △ is its triangle (10 × 9 inside 12 × 11, one
/// point of ink), other marks are 12 points (UI review U4).
#[test]
fn mark_samples_are_as_large_as_the_mockup() {
    let mut h = Harness::new();
    let t = h.tokens();
    let datum = legend_row(&mut h, "Datum", json!({"weight": null, "sample": {"mark": "△"}}));
    let finds = legend_row(&mut h, "Finds", json!({"weight": 0.2, "sample": {"mark": "③"}}));
    h.settle();
    let dr = h.rect(wid("legend-row", datum.0)).unwrap();
    let slot = Rect::from_min_size(pos2(dr.left() + 4.0 + 60.0, dr.center().y - 6.0), vec2(20.0, 12.0));
    let found = triangles_in(&h, slot.expand(1.0));
    let [(b, stroke)] = found.as_slice() else { panic!("one triangle: {found:?}") };
    assert!((b.width() - 10.0).abs() < 0.6 && (b.height() - 9.0).abs() < 0.6, "{b:?}");
    assert!((b.center().x - slot.center().x).abs() < 0.6, "centred: {b:?} in {slot:?}");
    assert_eq!((stroke.width, stroke.color.clone()), (1.0, egui::epaint::ColorMode::Solid(t.text)));
    let fr = h.rect(wid("legend-row", finds.0)).unwrap();
    assert_eq!(h.font_of("③", fr).map(|f| f.size), Some(12.0));
}

/// What is cut short says itself in full on hover, and 「Apart N」 says what it means: a long row
/// name, the badge, a long card head (cut with "…"), a long line of Weights at Once's table (UI
/// review U6, U11).
#[test]
fn cut_names_and_apart_badges_say_themselves_on_hover() {
    let mut h = Harness::sized(1600.0, 1400.0);
    let long = "Pit outline of the northern dwelling, second phase";
    let pits = legend_row(&mut h, long, json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.5})).unwrap();
    draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    let row = h.rect(wid("legend-row", pits.0)).unwrap();
    let tip = |h: &Harness, s: &str| h.texts.iter().any(|p| p.text == s);
    h.rest_at(pos2(row.left() + 140.0, row.center().y));
    assert!(tip(&h, &format!("{long}\nDraw with this row · double-click to change it")), "the name in full");
    let badge = h.texts.iter().find(|p| p.text == "Apart 1" && row.contains_rect(p.rect)).unwrap().rect;
    h.rest_at(badge.center());
    assert!(tip(&h, "Lines of this row with a weight or line type of their own: 1"), "what Apart means");
    h.double_click(wid("legend-row", pits.0));
    let (card, close) = (h.rect(wid("row-card", ())).unwrap(), h.rect(wid("card-close", ())).unwrap());
    let head = h.texts.iter().find(|p| p.text == long && card.contains(p.rect.left_center()) && p.rect.top() < close.bottom()).unwrap().rect;
    assert!(head.right() <= close.left(), "the card's head cut short with …: {head:?}, ✕ {close:?}");
    h.click(wid("card-close", ()));
    // Weights at Once's table: the longest line of rows changing alike.
    h.app.run("legend.addGroup", json!({"group": "plan"})).unwrap();
    h.settle();
    h.click(wid("legend-weights", ()));
    let card = h.rect(wid("weights-card", ())).unwrap();
    let longest = plan_table(&h, json!({"how": "ratio", "ratio": 0.8})).into_iter().map(|r| r.0).max_by_key(|n| n.chars().count()).unwrap();
    let cell = h.texts.iter().find(|p| p.text == longest && card.contains_rect(p.rect)).unwrap().rect;
    h.rest_at(pos2(cell.left() + 20.0, cell.center().y));
    assert!(h.texts.iter().filter(|p| p.text == longest).count() >= 2, "{longest} on hover");
}

/// With no drawing open Weights at Once's button is dimmed: there is nothing to change (UI review
/// U7).
#[test]
fn the_weights_button_waits_for_a_drawing() {
    let mut h = Harness::new();
    let t = h.tokens();
    h.app.run("file.close", json!({})).unwrap();
    h.settle();
    assert!(h.app.session.active().is_none());
    let b = h.rect(wid("legend-weights", ())).expect("the button");
    assert_eq!(h.color_of("All Weights ×0.8 …", b), Some(t.text_disabled));
}

/// Dimmed controls say on hover why they wait: Weights at Once for a drawing, a weightless row's
/// line types for a weight (UI review 2 U28).
#[test]
fn dimmed_controls_say_why_on_hover() {
    let mut h = Harness::new();
    let datum = legend_row(&mut h, "Datum", json!({"weight": null, "sample": {"mark": "△"}}));
    h.settle();
    h.double_click(wid("legend-row", datum.0));
    h.rest_at(h.rect(wid("card-line", "dashed")).unwrap().center());
    let shown: Vec<&str> = h.texts.iter().map(|p| p.text.as_str()).collect();
    assert!(shown.iter().any(|s| s.contains("A row without a weight has no line type")), "{shown:?}");
    // No drawing open: a new one closes without asking to save it.
    let mut h = Harness::new();
    h.app.run("file.close", json!({})).unwrap();
    h.settle();
    assert!(h.app.session.active().is_none());
    h.rest_at(h.rect(wid("legend-weights", ())).unwrap().center());
    let shown: Vec<&str> = h.texts.iter().map(|p| p.text.as_str()).collect();
    assert!(shown.iter().any(|s| s.contains("Open a drawing to change its weights")), "{shown:?}");
    assert!(!shown.iter().any(|s| s.contains("Change every line's weight at once")), "{shown:?}");
}

/// A click on a locked or hidden row says why it isn't drawn with and leaves the current layer as
/// it was; so does picking it in the Control bar's list (UI review U8).
#[test]
fn a_locked_or_hidden_row_is_not_picked() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("layer.setCurrent", json!({ "id": first.0 })).unwrap();
    h.app.run("layer.setProps", json!({ "ids": [pits.0], "locked": true })).unwrap();
    h.settle();
    h.click(wid("legend-row", pits.0));
    assert_eq!(h.current_layer(), Some(first));
    assert!(h.status_line().contains("Pits is locked: unlock it to draw with it"), "{}", h.status_line());
    h.app.run("layer.setProps", json!({ "ids": [pits.0], "locked": false, "visible": false })).unwrap();
    h.later(1.0);
    h.click(wid("legend-row", pits.0));
    assert_eq!(h.current_layer(), Some(first));
    assert!(h.status_line().contains("Pits is hidden: show it to draw with it"), "{}", h.status_line());
    h.app.select_tool("pen");
    h.app.status("");
    h.settle();
    h.click(wid("ctl-layer", ()));
    h.click(wid("ctl-layer-row", pits.0));
    assert_eq!(h.current_layer(), Some(first));
    assert!(h.status_line().contains("Pits is hidden: show it to draw with it"), "{}", h.status_line());
}

/// A row without a weight has no look: its line types are dimmed and change nothing (UI review U9).
#[test]
fn a_row_without_a_weight_has_its_line_types_dimmed() {
    let mut h = Harness::new();
    let datum = legend_row(&mut h, "Datum", json!({"weight": null, "sample": {"mark": "△"}}));
    h.settle();
    h.double_click(wid("legend-row", datum.0));
    assert_eq!(h.enabled(wid("card-line", "dashed")), Some(false));
    h.click(wid("card-line", "dashed"));
    let line = h.app.session.active().unwrap().doc.node(datum).unwrap().legend().unwrap().line.clone();
    assert_eq!(line, vectorcraft_doc::LineType::Solid);
    // With a weight they run.
    h.click(wid("card-weight", "0.5"));
    assert_eq!(h.enabled(wid("card-line", "dashed")), Some(true));
}

/// The cards' sizes and type as in the mockup (figures ② and ③, UI review nits U12–U17): the
/// legend footer's buttons 24 points, chip rows 24, the line under the Strata field 18, "= mm" in
/// 12 points, "pt" right after the number, the row's line count in mono, ×0.8 in mono ink, the
/// table's lines 34 and its head 30.
#[test]
fn cards_match_the_mockup_sizes() {
    let mut h = Harness::sized(1600.0, 1400.0);
    let t = h.tokens();
    let pits = legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.app.run("stroke.set", json!({"weight": 0.8})).unwrap();
    draw_rect(&mut h, 10.0);
    h.app.run("select.set", json!({ "ids": [] })).unwrap();
    h.settle();
    assert_eq!(h.rect(wid("legend-weights", ())).unwrap().height(), 24.0, "the legend footer's button");
    h.double_click(wid("legend-row", pits.0));
    let card = h.rect(wid("row-card", ())).unwrap();
    assert_eq!(h.rect(wid("card-weight", "0.5")).unwrap().height(), 24.0);
    assert_eq!(h.rect(wid("card-line", "solid")).unwrap().height(), 24.0);
    let strata = h.rect(wid("card-strata", ())).unwrap();
    let note = h.texts.iter().find(|p| p.text == "This row is the layer" && card.contains_rect(p.rect)).unwrap().rect;
    let chip = h.rect(wid("card-weight", "0.5")).unwrap();
    assert!((chip.top() - strata.bottom() - (5.0 + 18.0 + 5.0)).abs() < 0.6, "18 for the line between: {strata:?} {note:?} {chip:?}");
    assert_eq!(h.font_of("= 0.28 mm", card).map(|f| f.size), Some(12.0));
    let field = h.rect(wid("card-pt", ())).unwrap();
    let number = h.texts.iter().find(|p| p.text == "0.8" && field.contains_rect(p.rect)).unwrap().rect;
    let unit = h.texts.iter().find(|p| p.text == "pt" && field.contains_rect(p.rect)).unwrap().rect;
    assert!(unit.left() - number.right() < 8.0, "pt right after the number: {number:?} {unit:?}");
    assert_eq!(h.font_of("Lines: 1", card).map(|f| f.family), Some(crate::theme::mono(12.0).family));
    h.click(wid("card-close", ()));
    h.click(wid("legend-weights", ()));
    let card = h.rect(wid("weights-card", ())).unwrap();
    assert_eq!(h.rect(wid("weights-how", "ratio")).unwrap().height(), 24.0);
    let ratio = h.rect(wid("weights-ratio", ())).unwrap();
    assert_eq!(h.font_of("×", ratio).map(|f| f.family), Some(crate::theme::mono(11.0).family));
    assert_eq!(h.color_of("×", ratio), Some(t.text));
    let head = h.texts.iter().find(|p| p.text == "Legend Row" && card.contains_rect(p.rect)).unwrap().rect;
    let first = h.texts.iter().find(|p| p.text == "Pits" && card.contains_rect(p.rect)).unwrap().rect;
    assert!((first.center().y - head.center().y - (30.0 / 2.0 + 34.0 / 2.0)).abs() < 1.0, "head 30, lines 34: {head:?} {first:?}");
}

/// The Control bar's legend list shows the rows in the legend's order: the Strata rows' section,
/// then the drawing's, favourites first in each (UI review U22).
#[test]
fn control_bar_lists_rows_in_the_legend_order() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];
    let a = legend_row(&mut h, "A", json!({"weight": 0.5, "strata": "feature_line"}));
    let b = legend_row(&mut h, "B", json!({"weight": 0.5, "favourite": true}));
    let c = legend_row(&mut h, "C", json!({"weight": 0.2}));
    h.app.select_tool("pen");
    h.settle();
    let all = [first, a, b, c];
    assert_eq!(row_order(&h, &all), [a, b, c, first]);
    h.click(wid("ctl-layer", ()));
    let mut listed: Vec<(f32, NodeId)> = all.iter().map(|id| (h.rect(wid("ctl-layer-row", id.0)).unwrap().top(), *id)).collect();
    listed.sort_by(|x, y| x.0.total_cmp(&y.0));
    assert_eq!(listed.into_iter().map(|(_, id)| id).collect::<Vec<_>>(), [a, b, c, first]);
}

/// In a small window a card pushed up to clear the status bar stops under the tab row and scrolls
/// instead (UI review U23).
#[test]
fn a_card_stays_under_the_tab_row() {
    let mut h = Harness::sized(1366.0, 728.0);
    let first = h.layer_ids()[0];
    legend_row(&mut h, "Pits", json!({"weight": 0.8}));
    h.settle();
    h.double_click(wid("legend-row", first.0));
    let c = h.rect(wid("row-card", ())).expect("the editor");
    let canvas = h.app.canvas_rect.unwrap();
    assert!(c.top() >= canvas.top() - 0.5, "{c:?} under the tab row, canvas {canvas:?}");
    assert!(c.bottom() <= h.panel(STATUS).top() + 0.5, "{c:?}");
}

/// A press on something a later job brings that opens a dialog names it without its "…" (code
/// review C14: nothing on screen reaches this now that Change Weights at Once… runs).
#[test]
fn a_soon_note_drops_a_dialogs_ellipsis() {
    let ctx = egui::Context::default();
    super::note_soon(&ctx, "Change All Weights…", "J005");
    assert_eq!(super::soon_note(&ctx).as_deref(), Some("Change All Weights: coming soon (J005)"));
}

#[test]
fn j014_hatch_and_cut_face_and_estimate_controls() {
    let mut h = Harness::new();
    let first = h.layer_ids()[0];

    // 1. Test ctl-estimate toggle in control bar (with a drawing tool active)
    h.app.select_tool("pen");
    h.settle();
    h.click(wid("ctl-estimate", ()));
    let stroke = h.app.session.shown_stroke().unwrap();
    assert!(stroke.dash.is_some(), "clicking ctl-estimate toggles on dashed stroke");
    h.click(wid("ctl-estimate", ()));
    let stroke2 = h.app.session.shown_stroke().unwrap();
    assert!(stroke2.dash.is_none(), "clicking ctl-estimate toggles off dashed stroke");

    // 2. Test Cut Face button with a rectangle selected
    let rect_id = NodeId(h.app.run("shape.rectangle", json!({"x": 10.0, "y": 10.0, "width": 50.0, "height": 30.0})).unwrap()["id"].as_u64().unwrap());
    h.app.run("select.set", json!({"ids": [rect_id.0]})).unwrap();
    h.settle();

    h.click(wid("palette-cut", ()));
    h.settle();
    let bar = h.panel(STATUS);
    assert!(h.line_in(bar).contains("1"), "status reports created cut face");

    // 3. Test card-hatch in row editor
    h.double_click(wid("legend-row", first.0));
    h.settle();
    assert!(h.rect(wid("row-card", ())).is_some());
    h.click(wid("card-hatch", "45"));
    h.settle();
    let layer = h.app.session.active().unwrap().doc.node(first).unwrap().clone();
    assert_eq!(layer.legend().unwrap().hatch, vectorcraft_doc::Hatch::Deg45);
}

#[test]
fn trowel_home_screen_displays_dashboard_and_creates_templates() {
    let mut h = Harness::new();
    h.app.run("app.home", json!({})).unwrap();
    h.settle();

    assert!(crate::menus::home_showing(&h.app));
    let has = |s: &str| h.texts.iter().any(|p| p.text.contains(s));
    assert!(has("Trowel"), "Home shows Trowel brand");
    assert!(has("조사 연동") || has("Survey Link"), "shows survey link");
    assert!(has("최근 도면집") || has("Recent Drawing Set"), "shows recent drawings section");
    assert!(has("도면 작성 3단계 순서") || has("3-Step Drawing Workflow"), "shows workflow");
    assert!(has("새 도면 견본 (템플릿 4종)") || has("New Drawing Templates (4 Types)"), "shows template cards");

    // Creating plan template switches away from Home and populates plan layers
    h.app.run("drawing.newPlan", json!({})).unwrap();
    h.settle();
    assert!(!crate::menus::home_showing(&h.app));
    assert_eq!(h.app.session.documents().len(), 2);
    let doc = &h.app.session.active().unwrap().doc;
    assert_eq!(doc.layers.len(), 15);
    assert!(doc.layers.iter().any(|l| l.display_name() == "굴광선 (유구 면)"));
}

#[test]
fn ribbon_new_drawing_tiles_are_active() {
    let mut h = Harness::new();
    let _r_plan = h.rect(wid("tile", "plan")).expect("plan tile");
    let r_section = h.rect(wid("tile", "section")).expect("section tile");
    let r_finds = h.rect(wid("tile", "finds")).expect("finds tile");
    let _r_sheet = h.rect(wid("tile", "sheet")).expect("sheet tile");

    // Click section tile
    h.click_at(r_section.center());
    h.settle();
    assert_eq!(h.app.session.documents().len(), 2);
    let doc = &h.app.session.active().unwrap().doc;
    assert_eq!(doc.layers.len(), 12);
    assert!(doc.layers.iter().any(|l| l.display_name() == "단면선 (잘린 땅)"));

    // Click finds tile
    h.click_at(r_finds.center());
    h.settle();
    assert_eq!(h.app.session.documents().len(), 3);
    let doc2 = &h.app.session.active().unwrap().doc;
    assert_eq!(doc2.layers.len(), 7);
    assert!(doc2.layers.iter().any(|l| l.display_name() == "외형선"));
}

#[test]
fn scale_and_coord_card_and_status_bar_display() {
    let mut h = Harness::new();
    let bar = h.panel(STATUS);
    assert!(h.line_in(bar).contains("Scale and coordinate system not set"));

    // Click status-scale opens the card
    h.click(wid("status-scale", ()));
    assert!(h.rect(wid("coord-card", ())).is_some(), "the coord card opens");

    // Click 1:10 scale segment
    h.click(wid("coord-scale", "10"));

    // Apply
    h.click(wid("coord-apply", ()));
    assert!(h.rect(wid("coord-card", ())).is_none(), "the card is closed");

    // Status bar shows 1:10 and 5186
    let line = h.line_in(h.panel(STATUS));
    assert!(line.contains("1:10") && line.contains("5186"), "{line}");

    // Hover over canvas shows formatted TM coordinates in status bar
    let at = h.app.canvas_rect.unwrap().center();
    h.frame(vec![Event::PointerMoved(at)]);
    h.settle();
    let line = h.line_in(h.panel(STATUS));
    assert!(line.contains('X') && line.contains('Y'), "{line}");

    // Drawings set foot also shows Scale 1:10 and 5186
    let foot = h.panel(SET);
    let set_line = h.line_in(foot);
    assert!(set_line.contains("1:10") && set_line.contains("5186"), "{set_line}");
}

#[test]
fn drawing_menu_make_level_lines_runs_and_reports_status() {
    let mut h = Harness::new();
    h.app.run("drawing.newSection", json!({})).unwrap();
    h.settle();

    let bar = h.panel(MENU);
    let at = h.texts.iter().find(|p| p.text == "Drawing" && bar.contains_rect(p.rect)).unwrap().rect.center();
    h.click_at(at);
    let items = h.popup_buttons();
    // 5th item is "Make Level Lines"
    h.click_at(items[4].0.center());
    h.settle();

    let status = h.status_line();
    assert!(status.contains("Level Lines:") || status.contains("해발선"), "{status}");
}

/// J016: Pressure curve calculations and boundary clamping.
#[test]
fn pen_pressure_curve_math_and_clamping() {
    use crate::canvas::apply_pressure_curve;

    // Normal curve is linear
    assert_eq!(apply_pressure_curve(0.0, "normal"), 0.0);
    assert_eq!(apply_pressure_curve(1.0, "normal"), 1.0);
    assert!((apply_pressure_curve(0.5, "normal") - 0.5).abs() < 1e-6);

    // Soft curve is responsive (higher value for medium pressure)
    let soft_mid = apply_pressure_curve(0.5, "soft");
    assert!(soft_mid > 0.5, "soft curve should be more responsive at mid-point: {soft_mid}");

    // Firm curve requires deliberate pressure (lower value for medium pressure)
    let firm_mid = apply_pressure_curve(0.5, "firm");
    assert!(firm_mid < 0.5, "firm curve should be firmer at mid-point: {firm_mid}");

    // Out of bounds values clamp properly
    assert_eq!(apply_pressure_curve(-0.5, "normal"), 0.0);
    assert_eq!(apply_pressure_curve(1.5, "normal"), 1.0);
    assert_eq!(apply_pressure_curve(-0.5, "soft"), 0.0);
    assert_eq!(apply_pressure_curve(1.5, "soft"), 1.0);
}

/// J016: Control bar Pen Pressure segmented button toggles uniform vs follow pressure.
#[test]
fn control_bar_pen_pressure_segmented_button() {
    let mut h = Harness::new();
    h.app.run("drawing.newPlan", json!({})).unwrap();
    h.app.select_tool("pen");
    h.settle();

    assert!(!h.app.session.prefs.pen_pressure_follow, "default is uniform weight");

    // Click Follow Pressure (segment 1)
    let seg1_id = wid("ctl-pressure-seg", 1);
    let r1 = h.rect(seg1_id).expect("ctl-pressure-seg 1 should be laid out");
    h.click_at(r1.center());
    h.settle();
    assert!(h.app.session.prefs.pen_pressure_follow, "should toggle to follow pressure");

    // Click Same Weight (segment 0)
    let seg0_id = wid("ctl-pressure-seg", 0);
    let r0 = h.rect(seg0_id).expect("ctl-pressure-seg 0 should be laid out");
    h.click_at(r0.center());
    h.settle();
    assert!(!h.app.session.prefs.pen_pressure_follow, "should toggle back to uniform weight");
}

/// J016: Canvas button row renders and tablet settings card can be opened.
#[test]
fn canvas_button_row_and_tablet_card() {
    let mut h = Harness::new();
    h.app.run("drawing.newPlan", json!({})).unwrap();
    h.settle();

    // Canvas button row is visible
    let row_id = wid("canvas-btn-row", ());
    assert!(h.rect(row_id).is_some(), "canvas button row should be present");

    // Click the tablet settings button on the canvas bar
    let tablet_btn_id = wid("cbtn", "tablet");
    let btn_r = h.rect(tablet_btn_id).expect("tablet settings button should be present");
    h.click_at(btn_r.center());
    h.settle();

    // Tablet card should be open
    let card_id = wid("tablet-card", ());
    assert!(h.rect(card_id).is_some(), "tablet settings card should open");

    // Click Save Settings button
    let apply_id = wid("tablet-apply", ());
    let apply_r = h.rect(apply_id).expect("save settings button should be present");
    h.click_at(apply_r.center());
    h.settle();

    let status = h.status_line();
    assert!(status.contains("Tablet & Pen") || status.contains("타블렛"), "{status}");
}

/// J008: Underlay alignment card opens from ribbon Align tile, toggles methods, and closes.
#[test]
fn underlay_align_card_opens_and_toggles() {
    let mut h = Harness::new();
    h.app.run("drawing.newPlan", json!({})).unwrap();
    h.settle();

    // Click the Align tile on the ribbon
    let align_tile_id = wid("tile", "align");
    let tile_r = h.rect(align_tile_id).expect("align tile should be present on ribbon");
    h.click_at(tile_r.center());
    h.settle();

    // Underlay alignment card should be open
    let card_id = wid("underlay-card", ());
    assert!(h.rect(card_id).is_some(), "underlay alignment card should open");

    // Toggle 2-Point Similarity
    let btn_2pt_id = wid("underlay-2pt", ());
    let r_2pt = h.rect(btn_2pt_id).expect("2-point segment button should be present");
    h.click_at(r_2pt.center());
    h.settle();
    assert!(h.rect(card_id).is_some(), "card stays open in 2-point mode");

    // Toggle 4-Point Affine
    let btn_affine_id = wid("underlay-affine", ());
    let r_aff = h.rect(btn_affine_id).expect("affine segment button should be present");
    h.click_at(r_aff.center());
    h.settle();
    assert!(h.rect(card_id).is_some(), "card stays open in affine mode");

    // Verify Apply button is present
    let apply_id = wid("underlay-apply", ());
    assert!(h.rect(apply_id).is_some(), "apply button should be present");

    // Click Cancel button to close card
    let close_id = wid("underlay-close", ());
    let close_r = h.rect(close_id).expect("cancel button should be present");
    h.click_at(close_r.center());
    h.settle();
    assert!(h.rect(card_id).is_none(), "underlay alignment card should close on cancel");
}

#[test]
fn j009_plan_tools_and_metrics_integration() {
    let mut h = Harness::new();

    // 1. Tool selection for all 5 archaeological plan tools
    for tool_name in ["section-line", "spot-height", "find-number", "posthole", "hachure"] {
        h.click(wid("arch", tool_name));
        assert_eq!(h.app.session.tool_id(), tool_name, "tool switched to {tool_name}");
    }

    // 2. Execution of the 5 archaeological plan drawing commands
    let sl_res = h.app.run("drawing.addSectionLine", json!({"x1": 10.0, "y1": 10.0, "x2": 100.0, "y2": 10.0, "label": "A"})).unwrap();
    assert_eq!(sl_res["label"], "A");

    let sh_res = h.app.run("drawing.addSpotHeight", json!({"x": 50.0, "y": 50.0, "elevation": 152.36})).unwrap();
    assert_eq!(sh_res["elevation"], 152.36);

    let fn_res = h.app.run("drawing.addFindNumber", json!({"x": 60.0, "y": 60.0})).unwrap();
    assert_eq!(fn_res["number"], 1);

    let ph_res = h.app.run("drawing.addPosthole", json!({"x": 70.0, "y": 70.0, "radius": 13.0, "depth_cm": 25})).unwrap();
    assert_eq!(ph_res["number"], 1);
    assert_eq!(ph_res["depth_cm"], 25);

    let ht_res = h.app.run("drawing.addSlopeTicks", json!({"points": [[10.0, 10.0], [50.0, 10.0], [100.0, 10.0]], "spacing": 6.0})).unwrap();
    assert!(ht_res["ticks"].as_u64().unwrap() > 0);

    // 3. When Closed dropdown interaction in control bar
    h.app.select_tool("pen");
    h.settle();
    let when_closed_id = wid("ctl-closed", ());
    let when_closed_rect = h.rect(when_closed_id).expect("ctl-closed widget present");
    assert!(when_closed_rect.is_positive());

    // 4. Closed polygon metrics calculation and properties panel display
    // Scale 1:20 => 141.73228 pt per meter
    h.app.run("drawing.setCoord", json!({"scale": 20.0})).unwrap();
    let rect_id = NodeId(
        h.app.run("shape.rectangle", json!({"x": 200.0, "y": 200.0, "width": 141.73228, "height": 141.73228})).unwrap()["id"].as_u64().unwrap(),
    );
    h.app.run("select.set", json!({"ids": [rect_id.0]})).unwrap();
    h.settle();

    let metrics_res = h.app.run("drawing.calculateMetrics", json!({})).unwrap();
    assert!((metrics_res["area_m2"].as_f64().unwrap() - 1.0).abs() < 0.05);
    assert!((metrics_res["perimeter_m"].as_f64().unwrap() - 4.0).abs() < 0.05);

    h.app.run("feature.setProps", json!({"kind": "주거지", "number": 1, "period": "청동기"})).unwrap();
    h.settle();

    let right = h.panel(RIGHT);
    let right_line = h.line_in(right);
    assert!(right_line.contains("주거지"), "{}", right_line);
    assert!(right_line.contains("1호"), "{}", right_line);
    assert!(right_line.contains("청동기"), "{}", right_line);
    assert!(right_line.contains("Plan Area"), "{}", right_line);
    assert!(right_line.contains("1.00 ㎡"), "{}", right_line);
    assert!(right_line.contains("Perimeter"), "{}", right_line);
    assert!(right_line.contains("4.00 m"), "{}", right_line);
    assert!(right_line.contains("Major/Minor"), "{}", right_line);
}
