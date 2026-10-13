//! The tool column: fourteen slots in five groups (design 2.1 §3), the archaeology tools under
//! them and the fill and stroke at the bottom. A slot shows the tool in hand when it holds it, else
//! the one last used from it; a long press or a right click opens its other tools (the classic
//! toolbar's flyout).

use serde_json::json;

use super::*;

/// The slots, top to bottom: their tools (first = the slot's own, with Trowel's icon) and whether
/// a group ends after it.
const SLOTS: &[(&[&str], &str, bool)] = &[
    (&["selection"], "tw-select", false),
    (&["directSelection", "groupSelection", "lasso", "magicWand"], "tw-direct", true),
    (&["pen", "curvature", "addAnchor", "deleteAnchor", "anchorPoint", "wall-cut"], "tw-pen", false),
    (&["pencil", "smooth", "pathEraser", "join", "paintbrush", "blobBrush"], "tw-pencil", false),
    (&["lineSegment", "arc", "spiral", "stratum-line", "centerline"], "tw-line", false),
    (&["rectangle", "roundedRectangle", "ellipse", "polygon", "star", "rim-arc"], "tw-rect", false),
    (&["type", "areaType", "typeOnPath", "verticalType", "stratum-badge"], "tw-type", true),
    (&["rotate", "reflect"], "tw-rotate", false),
    (&["scale", "shear", "reshape", "freeTransform"], "tw-scale", false),
    (&["scissors", "knife", "eraser"], "tw-scissors", true),
    (&["livePaintBucket", "livePaintSelection", "shapeBuilder", "hearth-cut"], "tw-bucket", false),
    (&["eyedropper", "measure"], "tw-dropper", true),
    (&["hand", "rotateView"], "tw-hand", false),
    (&["zoom"], "tw-zoom", false),
];

/// A tool's button as in the mockup; a shorter window tightens it.
const BUTTON: egui::Vec2 = vec2(40.0, 27.0);
const BUTTON_TIGHT: f32 = 23.0;
/// The fill and stroke under the tools: the gap above them and their height.
const PROXY_GAP: f32 = 8.0;
const PROXY_H: f32 = 34.0;

/// The archaeology tools (J009): their keys, icons and names.
fn arch() -> [(&'static str, &'static str, &'static str); 5] {
    [
        ("section-line", "tw-section-line", tw!("Section Line")),
        ("spot-height", "tw-spot-height", tw!("Spot Height")),
        ("find-number", "tw-find-number", tw!("Find Number")),
        ("posthole", "tw-posthole", tw!("Posthole Depth")),
        ("hachure", "tw-hachure", tw!("Slope Hachure")),
    ]
}

/// How tall the tools are with buttons `bh` tall: the slots and the lines between their groups,
/// then the archaeology tools' title and buttons, from the padding at the top.
fn tools_height(bh: f32) -> f32 {
    let lines = SLOTS.iter().filter(|s| s.2).count() as f32;
    6.0 + SLOTS.len() as f32 * (bh + 1.0) + lines * 7.0 + 4.0 + 12.0 + arch().len() as f32 * (bh + 1.0)
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::left(TOOLS).resizable(false).exact_size(TOOLS_W).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_right(ui, r, t.border);
        let ctx = ui.ctx().clone();
        // The slot whose long press opened its flyout, until the next press.
        if ui.input(|i| i.pointer.any_pressed()) {
            ui.data_mut(|d| d.remove::<egui::Id>(held_id()));
        }
        // The tools as in the mockup with the fill and stroke under them while they fit; then the
        // buttons tighten; then the tools scroll above the fill and stroke at the column's foot.
        let room = r.height() - PROXY_GAP - PROXY_H - 6.0;
        let bh = if tools_height(BUTTON.y) <= room { BUTTON.y } else { BUTTON_TIGHT };
        let height = tools_height(bh);
        let (chosen, proxy_top) = if height <= room {
            (tools(app, ui, r, r.top(), bh, &t), r.top() + height + PROXY_GAP)
        } else {
            let view = Rect::from_min_max(r.min, pos2(r.right() - 1.0, r.top() + room));
            let mut list = ui.new_child(egui::UiBuilder::new().max_rect(view).layout(egui::Layout::top_down(egui::Align::Min)));
            list.set_clip_rect(view);
            let mut chosen = None;
            egui::ScrollArea::vertical().id_salt(wid("tools-scroll", ())).auto_shrink([false, false]).show(&mut list, |ui| {
                let (_, content) = ui.allocate_space(vec2(view.width(), height));
                chosen = tools(app, ui, r, content.top(), bh, &t);
            });
            (chosen, view.bottom() + PROXY_GAP)
        };
        // Fill and stroke.
        let proxy = Rect::from_min_size(pos2(r.left(), proxy_top), vec2(r.width(), PROXY_H));
        let mut cell = ui.new_child(egui::UiBuilder::new().id(wid("proxy", ())).max_rect(proxy).layout(egui::Layout::top_down(egui::Align::Center)));
        crate::panels::proxy(app, &mut cell, 30.0);
        match chosen {
            Some(Pick::Tool(id)) => app.select_tool(&id),
            Some(Pick::Options(id)) => {
                app.select_tool(&id);
                app.run("tool.options", json!({ "tool": id })).ok();
            }
            None => {}
        }
        // The classic toolbar's flyouts, open or torn off.
        crate::toolbar::floating(app, &ctx);
        crate::toolbar::flyout(app, &ctx);
    });
}

/// The slots and the archaeology tools in the column `r`, from `top`, their buttons `bh` tall.
/// Returns what a press picked.
fn tools(app: &mut VectorcraftApp, ui: &Ui, r: Rect, top: f32, bh: f32, t: &Tokens) -> Option<Pick> {
    let active = app.session.tool_id().to_string();
    let x = r.center().x - BUTTON.x / 2.0;
    let mut y = top + 6.0;
    let mut chosen = None;
    for (tools, icon, group_ends) in SLOTS {
        let key = tools.first().copied().unwrap_or_default();
        // The tool last used from the slot, kept with the classic toolbar's (apart from them).
        let memory = format!("trowel/{key}");
        let on = tools.contains(&active.as_str());
        if on {
            app.ui.slot_tool.insert(memory.clone(), active.clone());
        }
        let shown = if on {
            active.clone()
        } else {
            app.ui.slot_tool.get(&memory).filter(|s| tools.contains(&s.as_str())).cloned().unwrap_or_else(|| key.to_string())
        };
        let rect = Rect::from_min_size(pos2(x, y), vec2(BUTTON.x, bh));
        if let Some(pick) = slot(app, ui, tools, icon, &shown, rect, on, t) {
            chosen = Some(pick);
        }
        y = rect.bottom() + 1.0;
        if *group_ends {
            ui.painter().hline((r.center().x - 14.0)..=(r.center().x + 14.0), y + 3.5, Stroke::new(1.0, t.border));
            y += 7.0;
        }
    }
    // The archaeology tools (J009): their group title, then dimmed buttons.
    y += 4.0;
    text(ui, pos2(r.center().x, y + 5.0), Align2::CENTER_CENTER, tw!("Archaeology"), font(9.0), t.text_dim);
    y += 12.0;
    for (key, icon, label) in arch() {
        let rect = Rect::from_min_size(pos2(x, y), vec2(BUTTON.x, bh));
        let on = app.session.tool_id() == key;
        let resp = ui.interact(rect, wid("arch", key), Sense::click());
        let fill = if on {
            t.tool_active
        } else if resp.hovered() {
            t.hover
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, CornerRadius::same(8), fill);
        crate::icons::paint(ui, icon, Rect::from_center_size(rect.center(), vec2(19.0, 19.0)), if on { t.icon_on } else { t.icon });
        if let Some(sc) = crate::shortcut_editor::tool_shortcut(key) {
            text(ui, rect.right_bottom() - vec2(2.0, 1.0), Align2::RIGHT_BOTTOM, sc, mono(8.0), if on { t.icon_on } else { t.text_disabled });
        }
        if resp.on_hover_text(label).clicked() {
            chosen = Some(Pick::Tool(key.to_string()));
        }
        y = rect.bottom() + 1.0;
    }
    chosen
}

/// Where the slot whose long press opened its flyout is kept.
fn held_id() -> egui::Id {
    wid("tool-held", ())
}

/// What a press on a slot picked.
enum Pick {
    Tool(String),
    /// A double click: the tool and its options.
    Options(String),
}

/// One slot's button at `rect`, showing tool `shown` (the slot's own `icon` when it is the first).
#[allow(clippy::too_many_arguments)]
fn slot(app: &mut VectorcraftApp, ui: &Ui, tools: &[&'static str], icon: &str, shown: &str, rect: Rect, on: bool, t: &Tokens) -> Option<Pick> {
    let key = tools.first().copied().unwrap_or_default();
    let resp = ui.interact(rect, wid("tool", key), Sense::click());
    let fill = if on {
        t.tool_active
    } else if resp.hovered() {
        t.hover
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, CornerRadius::same(8), fill);
    let info = vectorcraft_tools::tool_info(shown);
    let glyph = if shown == key { icon } else { info.map_or(icon, |i| crate::icons::tool_icon(i.icon)) };
    crate::icons::paint(ui, glyph, Rect::from_center_size(rect.center(), vec2(19.0, 19.0)), if on { t.icon_on } else { t.icon });
    if let Some(sc) = crate::shortcut_editor::tool_shortcut(shown) {
        text(ui, rect.right_bottom() - vec2(2.0, 1.0), Align2::RIGHT_BOTTOM, sc, mono(8.0), if on { t.icon_on } else { t.text_disabled });
    }
    // A right press or a long one opens the slot's other tools; a slot torn off into its own strip
    // comes to the front instead.
    let press = if tools.len() > 1 { crate::toolbar::flyout_press(ui, &resp, None) } else { None };
    if let Some(press) = press {
        if crate::toolbar::is_floating(app, key) {
            crate::toolbar::raise_floating(ui.ctx(), key);
        } else {
            crate::toolbar::open_flyout(app, ui.ctx(), tools, rect);
        }
        if press == crate::toolbar::FlyoutPress::Primary {
            ui.data_mut(|d| d.insert_temp(held_id(), resp.id));
        }
    }
    let held = ui.data(|d| d.get_temp::<egui::Id>(held_id())) == Some(resp.id);
    let tip = info.map(crate::toolbar::tip).unwrap_or_default();
    // Its other tools, for those who don't know to look.
    let tip = if tools.len() > 1 { format!("{tip}\n{}", tw!("Press and hold for more tools")) } else { tip };
    let resp = resp.on_hover_text(tip);
    if press.is_some() || held && resp.clicked() {
        // Releasing the long press that opened the flyout leaves it open.
        None
    } else if resp.double_clicked() {
        Some(Pick::Options(shown.to_string()))
    } else if resp.clicked() {
        Some(Pick::Tool(shown.to_string()))
    } else {
        None
    }
}
