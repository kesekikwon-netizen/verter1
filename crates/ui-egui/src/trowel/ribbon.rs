//! The ribbon: one row of tiles in seven groups (design 2.1 §3), Find a Command at the right.
//! Each tile runs a command or picks a tool; what a later job brings is dimmed.

use serde_json::json;

use super::*;

/// Inside the ribbon: its padding, a group's caption and padding, a tile and its icon well.
const PAD: f32 = 8.0;
const CAPTION: f32 = 14.0;
const GROUP_PAD: f32 = 9.0;
const TILE_W: f32 = 52.0;
const GAP: f32 = 6.0;
const WELL: f32 = 44.0;
const FIND: egui::Vec2 = vec2(150.0, 32.0);

/// What a tile does.
#[derive(Clone, Copy)]
enum Act {
    /// Run a menu command.
    Cmd(&'static str),
    /// Pick a tool.
    Tool(&'static str),
    /// The underlay: show or hide the template layers, or place a scan as one.
    Underlay,
    /// Align scanned underlay with 2 or 4 GCPs.
    Align,
    /// Comes with a later job.
    Soon(&'static str),
}

struct Tile {
    key: &'static str,
    icon: &'static str,
    label: &'static str,
    act: Act,
}

/// How a tile looks: at rest, its tool in hand, its view on, Save with changes to save, or off.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    Rest,
    On,
    Shown,
    Save,
    Off,
}

fn tile(key: &'static str, icon: &'static str, label: &'static str, act: Act) -> Tile {
    Tile { key, icon, label, act }
}

/// The groups and their tiles, left to right.
fn groups() -> [(&'static str, Vec<Tile>); 7] {
    use Act::*;
    [
        (
            tw!("File"),
            vec![
                tile("open", "tw-open", tw!("Open"), Cmd("file.open")),
                tile("save", "tw-save", tw!("Save"), Cmd("file.save")),
                tile("import", "tw-import", tw!("Import"), Cmd("file.place")),
            ],
        ),
        (
            tw!("New Drawing"),
            vec![
                tile("plan", "tw-plan", tw!("Plan"), Cmd("drawing.newPlan")),
                tile("section", "tw-section", tw!("Section"), Cmd("drawing.newSection")),
                tile("finds", "tw-finds", tw!("Finds"), Cmd("drawing.newFinds")),
                tile("sheet", "tw-sheet", tw!("Plate"), Cmd("drawing.newSheet")),
            ],
        ),
        (tw!("Underlay"), vec![tile("align", "tw-align", tw!("Align"), Align), tile("underlay", "tw-underlay", tw!("Underlay"), Underlay)]),
        (
            tw!("Draw"),
            vec![
                tile("pen", "tw-pen", tw!("Pen"), Tool("pen")),
                tile("pencil", "tw-pencil", tw!("Pencil"), Tool("pencil")),
                tile("section-line", "tw-section-line", tw!("Section Line"), Soon("J009")),
                tile("spot-height", "tw-spot-height", tw!("Spot Height"), Soon("J009")),
                tile("bucket", "tw-bucket", tw!("Bucket"), Tool("livePaintBucket")),
            ],
        ),
        (
            tw!("Drawing Elements"),
            vec![
                tile("north", "tw-north", tw!("North Arrow"), Soon("J012")),
                tile("scale-bar", "tw-scale-bar", tw!("Scale Bar"), Soon("J012")),
                tile("legend", "tw-legend", tw!("Legend Table"), Soon("J012")),
                tile("title-block", "tw-title-block", tw!("Title Block"), Soon("J012")),
            ],
        ),
        (
            tw!("View"),
            vec![tile("grid", "tw-grid", tw!("Grid"), Cmd("view.grid")), tile("magnet", "tw-magnet", tw!("Snap"), Cmd("view.snapToPoint"))],
        ),
        (
            tw!("Export"),
            vec![
                tile("pdf", "tw-pdf", tw!("PDF"), Cmd("file.export.pdf")),
                tile("dxf", "tw-dxf", tw!("DXF"), Cmd("ui.dxfOptionsDialog")),
                tile("strata", "tw-strata", tw!("To Strata"), Soon("J019")),
            ],
        ),
    ]
}

fn look(app: &VectorcraftApp, act: Act) -> Look {
    match act {
        Act::Soon(_) => Look::Off,
        Act::Align => {
            if app.session.active().is_some() {
                Look::Rest
            } else {
                Look::Off
            }
        }
        Act::Underlay => match templates(app) {
            (ids, true) if !ids.is_empty() => Look::Shown,
            (ids, _) if !ids.is_empty() => Look::Rest,
            _ => look(app, Act::Cmd("file.place")),
        },
        Act::Tool(id) if app.session.tool_id() == id => Look::On,
        Act::Tool(_) => Look::Rest,
        Act::Cmd(id) if !crate::menus::enabled(app, id) => Look::Off,
        Act::Cmd("file.save") if app.session.active().is_some_and(|d| d.is_dirty()) => Look::Save,
        Act::Cmd(id) if crate::menus::checked(app, id, &json!({})) == Some(true) => Look::Shown,
        Act::Cmd(_) => Look::Rest,
    }
}

/// How the tiles are laid out: their width and the gap between them.
#[derive(Clone, Copy)]
struct Fit {
    tile_w: f32,
    gap: f32,
}

/// As in the mockup.
const FULL: Fit = Fit { tile_w: TILE_W, gap: GAP };
/// For a narrower window: tiles as wide as their wells.
const NARROW: Fit = Fit { tile_w: WELL, gap: 4.0 };
/// Find a Command folded to its magnifier, for a narrower window.
const FIND_FOLDED: f32 = 32.0;

/// How wide the groups are, laid out with `fit` (the ribbon's padding at the left included).
fn tiles_width(groups: &[(&str, Vec<Tile>)], fit: Fit) -> f32 {
    PAD + groups
        .iter()
        .map(|(_, tiles)| {
            let n = tiles.len() as f32;
            GROUP_PAD * 2.0 + n * fit.tile_w + (n - 1.0).max(0.0) * fit.gap + 1.0
        })
        .sum::<f32>()
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top(RIBBON).exact_size(RIBBON_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_below(ui, r, t.border);
        let groups = groups();
        // The tiles as in the mockup with Find a Command beside them while they fit; then the
        // tiles narrow, then Find a Command folds to its magnifier, then the tiles scroll. Each
        // narrower window gives up more, so Find a Command doesn't fold and open again.
        let room = r.width();
        let (fit, find_w) = if tiles_width(&groups, FULL) + PAD + FIND.x + PAD <= room {
            (FULL, FIND.x)
        } else if tiles_width(&groups, NARROW) + PAD + FIND.x + PAD <= room {
            (NARROW, FIND.x)
        } else {
            (NARROW, FIND_FOLDED)
        };
        let well_top = r.top() + PAD + CAPTION + 5.0;
        let find = Rect::from_center_size(pos2(r.right() - PAD - find_w / 2.0, well_top + WELL / 2.0), vec2(find_w, FIND.y));
        let view = Rect::from_min_max(r.min, pos2(find.left() - PAD, r.bottom() - 1.0));
        let width = tiles_width(&groups, fit);
        let mut run = None;
        if width <= view.width() {
            run = tiles(app, ui, &groups, fit, r.left(), r.top(), r.bottom(), &t);
        } else {
            let mut strip = ui.new_child(egui::UiBuilder::new().max_rect(view).layout(egui::Layout::left_to_right(egui::Align::Min)));
            strip.set_clip_rect(view);
            // A mouse's wheel turns it too.
            strip.style_mut().always_scroll_the_only_direction = true;
            egui::ScrollArea::horizontal().id_salt(wid("ribbon-scroll", ())).auto_shrink([false, false]).show(&mut strip, |ui| {
                let (_, content) = ui.allocate_space(vec2(width, view.height()));
                run = tiles(app, ui, &groups, fit, content.left(), content.top(), r.bottom(), &t);
            });
        }
        find_button(app, ui, find, find_w < FIND.x, &t);
        match run {
            Some(Act::Cmd(id)) => crate::menus::invoke(app, id, json!({})),
            Some(Act::Tool(id)) => app.select_tool(id),
            Some(Act::Underlay) => underlay(app),
            Some(Act::Align) => super::legend::open_underlay_align(app, ui.ctx()),
            Some(Act::Soon(_)) | None => {}
        }
    });
}

/// The drawing's underlays, template layers holding something (a scan placed as a template: locked
/// and dimmed; the plan legend's Underlay row, empty, is not one yet), and whether any of them
/// shows.
fn templates(app: &VectorcraftApp) -> (Vec<u64>, bool) {
    let Some(st) = app.session.active() else { return (vec![], false) };
    let layers: Vec<_> = st.doc.layers.iter().filter(|l| l.is_template() && l.children().is_some_and(|c| !c.is_empty())).collect();
    (layers.iter().map(|l| l.id.0).collect(), layers.iter().any(|l| l.visible))
}

/// The Underlay tile: with underlays ([`templates`]), hide them all or show them all; else place a
/// scan as one (File › Place with Template ticked).
fn underlay(app: &mut VectorcraftApp) {
    let (ids, shown) = templates(app);
    if ids.is_empty() {
        // Under every line, its layer named in the user's words (`file.place`'s `underlay`).
        crate::menus::invoke(app, "file.place", json!({ "template": true, "underlay": tw!("Underlay") }));
    } else if let Err(e) = app.run("layer.setProps", json!({ "ids": ids, "visible": !shown })) {
        app.status(e);
    }
}

/// The groups from `left`, their captions at `top`, the tiles down to `bottom`, a line after each
/// group. Returns what a clicked tile does.
#[allow(clippy::too_many_arguments)]
fn tiles(app: &VectorcraftApp, ui: &Ui, groups: &[(&str, Vec<Tile>)], fit: Fit, left: f32, top: f32, bottom: f32, t: &Tokens) -> Option<Act> {
    let well_top = top + PAD + CAPTION + 5.0;
    let mut run = None;
    let mut x = left + PAD;
    for (caption, tiles) in groups {
        text(ui, pos2(x + GROUP_PAD, top + PAD + CAPTION / 2.0), Align2::LEFT_CENTER, *caption, bold(11.0), t.text_dim);
        let mut tx = x + GROUP_PAD;
        for tl in tiles {
            let rect = Rect::from_min_max(pos2(tx, well_top), pos2(tx + fit.tile_w, bottom - 6.0));
            if let Some(act) = tile_ui(app, ui, tl, rect, t) {
                run = Some(act);
            }
            tx += fit.tile_w + fit.gap;
        }
        x = tx - fit.gap + GROUP_PAD;
        ui.painter().vline(x + 0.5, (top + PAD)..=(bottom - 6.0), Stroke::new(1.0, t.border));
        x += 1.0;
    }
    run
}

/// Find a Command at `find` (`folded`: its magnifier alone): it opens the command palette.
fn find_button(app: &mut VectorcraftApp, ui: &Ui, find: Rect, folded: bool, t: &Tokens) {
    let resp = ui.interact(find, wid("find", ()), Sense::click());
    let fill = if resp.hovered() { t.panel } else { t.input };
    field_box(ui, find, fill, t.input_border);
    if folded {
        crate::icons::paint(ui, "search", Rect::from_center_size(find.center(), vec2(13.0, 13.0)), t.text_dim);
    } else {
        crate::icons::paint(ui, "search", Rect::from_center_size(find.left_center() + vec2(16.0, 0.0), vec2(13.0, 13.0)), t.text_disabled);
        text(ui, find.left_center() + vec2(29.0, 0.0), Align2::LEFT_CENTER, tw!("Find a command"), font(12.0), t.text_disabled);
    }
    let label = if folded { tw!("Find a command") } else { tl!("Search commands and tools") };
    let tip = match crate::menus::shortcut_of("help.commandPalette") {
        Some(sc) => format!("{label} ({})", crate::menus::pretty_shortcut(sc)),
        None => label.to_string(),
    };
    if resp.on_hover_text(tip).clicked() {
        app.ui.palette_open = true;
        app.ui.palette_query.clear();
    }
}

/// One tile in `rect`: its well, icon and label. Returns what it does when clicked.
fn tile_ui(app: &VectorcraftApp, ui: &Ui, tile: &Tile, rect: Rect, t: &Tokens) -> Option<Act> {
    let look = look(app, tile.act);
    let id = wid("tile", tile.key);
    let resp = ui.interact(rect, id, Sense::click());
    let well = Rect::from_min_size(pos2(rect.center().x - WELL / 2.0, rect.top()), vec2(WELL, WELL));
    let fill = match look {
        Look::On => t.tool_active,
        Look::Shown => t.row_selected,
        Look::Save => t.action,
        Look::Off => t.panel_darker,
        Look::Rest if resp.hovered() => t.hover,
        Look::Rest => t.button,
    };
    let edge = if look == Look::Shown { Stroke::new(1.0, t.ring) } else { Stroke::NONE };
    ui.painter().rect(well, CornerRadius::same(12), fill, edge, StrokeKind::Inside);
    let tint = match look {
        Look::On => t.icon_on,
        Look::Save => t.on_action,
        Look::Off => t.icon.gamma_multiply(0.45),
        Look::Rest | Look::Shown => t.icon,
    };
    crate::icons::paint(ui, tile.icon, Rect::from_center_size(well.center(), vec2(24.0, 24.0)), tint);
    let (label_font, label_color) = match look {
        Look::On | Look::Shown | Look::Save => (bold(11.0), t.text),
        Look::Off => (font(11.0), t.text_dim),
        Look::Rest => (font(11.0), t.text),
    };
    let label = pos2(rect.center().x, well.bottom() + 11.0);
    ui.painter().with_clip_rect(rect.expand2(vec2(3.0, 0.0))).text(label, Align2::CENTER_CENTER, tile.label, label_font, label_color);
    let tip = match tile.act {
        Act::Soon(job) => soon(tile.label, job),
        Act::Align => tw!("Align scanned drawing to real-world scale and TM coordinates").to_string(),
        Act::Underlay if templates(app).0.is_empty() => tw!("Place a scan under the drawing").to_string(),
        Act::Underlay => tw!("Show or hide the underlay").to_string(),
        Act::Tool(id) => vectorcraft_tools::tool_info(id).map(crate::toolbar::tip).unwrap_or_default(),
        Act::Cmd(id) => match crate::menus::shortcut_of(id) {
            Some(sc) => format!("{} ({})", tile.label, crate::menus::pretty_shortcut(sc)),
            None => tile.label.to_string(),
        },
    };
    let resp = resp.on_hover_text(tip);
    match tile.act {
        // Off: it says what it waits for, and does nothing.
        Act::Soon(job) if resp.clicked() => {
            note_soon(ui.ctx(), tile.label, job);
            None
        }
        _ => (look != Look::Off && resp.clicked()).then_some(tile.act),
    }
}
