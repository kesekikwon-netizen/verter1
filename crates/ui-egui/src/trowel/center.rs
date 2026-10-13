//! The middle of the window: the document tabs over the canvas (Home first, the current drawing
//! underlined, a tab's × on hover, the drawing's name and whether it is saved at the right) and
//! what floats on the canvas: the tool band at the top left, the zoom buttons and the screen
//! readout at the bottom right.

use serde_json::json;

use super::*;

/// The ×'s size.
const CLOSE: f32 = 10.0;
/// The narrowest the current tab gets: a letter or two and its ×.
const MIN_TAB: f32 = 14.0 + 16.0 + 8.0 + CLOSE + 8.0;

/// The canvas under the tabs (screen mode 3: the canvas alone) and what floats on it.
pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, mode: u8) {
    let full = ui.max_rect();
    let below = if mode < 3 { Rect::from_min_max(pos2(full.left(), tabs(app, ui, full)), full.max) } else { full };
    let mut canvas = ui.new_child(egui::UiBuilder::new().max_rect(below).layout(*ui.layout()));
    crate::canvas::show(app, &mut canvas);
    if mode < 3 {
        overlays(app, &mut canvas);
    }
}

/// The tab strip at the top of `full`. Returns its bottom.
fn tabs(app: &mut VectorcraftApp, ui: &mut Ui, full: Rect) -> f32 {
    let t = Tokens::get(ui.ctx());
    let strip = Rect::from_min_size(full.min, vec2(full.width(), TABS_H));
    ui.painter().rect_filled(strip, 0.0, t.panel);
    line_below(ui, strip, t.border);
    let home = crate::menus::home_showing(app);
    let active = app.session.active_index().filter(|_| !home);
    let titles: Vec<String> = app.session.documents().iter().map(|d| d.title()).collect();
    let home_w = width_of(ui, tw!("Home"), font(12.0)) + 28.0;
    let widths: Vec<f32> = titles.iter().map(|n| 14.0 + width_of(ui, n, font(12.0)) + 8.0 + CLOSE + 8.0).collect();
    let left = strip.left() + 8.0;
    // What is at the right leaves the tabs their room, the current one's first.
    let current_w = active.and_then(|i| widths.get(i)).copied().unwrap_or(0.0);
    let need = [left + home_w + widths.iter().sum::<f32>() + 8.0, left + home_w + current_w + 8.0];
    let corner_left = corner(app, ui, strip, home, need, &t);
    let room = corner_left - 8.0 - (left + home_w);
    // The current drawing's tab stays in view; the drawing set lists them all.
    let first = crate::chrome::first_tab_shown(&widths, active, room);
    let h = TABS_H - 1.0;
    let mut go_home = false;
    let mut activate = None;
    let mut close = None;
    let home_tab = Rect::from_min_size(pos2(left, strip.top()), vec2(home_w, h));
    if tab(ui, wid("tab-home", ()), home_tab, tw!("Home"), home, false, &t).clicked() {
        go_home = true;
    }
    let end = left + home_w + room;
    let mut x = home_tab.right();
    for (i, (title, w)) in titles.iter().zip(&widths).enumerate().skip(first) {
        if x + w > end && Some(i) != active {
            break;
        }
        // The current tab takes the room there is, its name cut short.
        let w = if x + w > end { (end - x).max(MIN_TAB) } else { *w };
        let r = Rect::from_min_size(pos2(x, strip.top()), vec2(w, h));
        let resp = tab(ui, wid("tab", i), r, title, Some(i) == active, true, &t);
        let x_rect = Rect::from_center_size(pos2(r.right() - 8.0 - CLOSE / 2.0, r.center().y), vec2(CLOSE, CLOSE));
        let x_resp = ui.interact(x_rect.expand(3.0), wid("tab-close", i), Sense::click());
        if resp.hovered() || x_resp.hovered() {
            crate::icons::paint(ui, "x", x_rect, if x_resp.hovered() { t.text } else { t.text_dim });
        }
        if x_resp.on_hover_text(tw!("Close")).clicked() {
            close = Some(i);
        } else if resp.clicked() {
            activate = Some(i);
        }
        x = r.right();
    }
    if let Some(i) = close {
        if let Err(e) = crate::unsaved::close(app, i) {
            app.status(e);
        }
    } else if let Some(i) = activate {
        app.ui.home = None;
        if let Err(e) = app.run("document.activate", json!({ "index": i })) {
            app.status(e);
        }
    } else if go_home {
        crate::menus::invoke(app, "app.home", json!({}));
    }
    strip.bottom()
}

/// One tab in `r`: `label`, cut short with "…" when it doesn't fit before the × (`close`) or the
/// tab's end, and underlined when it is the `current` one.
fn tab(ui: &Ui, id: egui::Id, r: Rect, label: &str, current: bool, close: bool, t: &Tokens) -> egui::Response {
    let resp = ui.interact(r, id, Sense::click());
    let color = if current || resp.hovered() { t.text } else { t.text_dim };
    let x = r.left() + 14.0;
    let room = r.right() - if close { 8.0 + CLOSE + 8.0 } else { 14.0 } - x;
    let mut job = egui::text::LayoutJob::simple_singleline(label.to_string(), font(12.0), color);
    if width_of(ui, label, font(12.0)) > room {
        job.wrap = egui::text::TextWrapping::truncate_at_width(room.max(0.0));
    }
    let name = ui.painter().layout_job(job);
    ui.painter().with_clip_rect(r).galley(pos2(x, r.center().y - name.size().y / 2.0), name, color);
    if current {
        ui.painter().rect_filled(Rect::from_min_max(pos2(r.left(), r.bottom() - 2.0), r.max), 0.0, t.text);
    }
    resp
}

/// The current drawing's name at the right end of the strip, and whether it is saved. The tabs
/// want the strip up to `need`: all of them, the current one. The name, which its tab shows too,
/// shows when all of them fit; "Not saved" when the current one does, else its dot, which says it
/// on hover. Returns where it begins.
fn corner(app: &VectorcraftApp, ui: &Ui, strip: Rect, home: bool, need: [f32; 2], t: &Tokens) -> f32 {
    let Some(d) = app.session.active().filter(|_| !home) else { return strip.right() };
    let cy = strip.center().y;
    let right = strip.right() - 12.0;
    let dirty = d.is_dirty();
    let (dot, note) = if dirty { (6.0, width_of(ui, tw!("Not saved"), font(12.0)) + 5.0) } else { (0.0, 0.0) };
    let name = width_of(ui, &d.title(), font(12.0)) + if dirty { 8.0 } else { 0.0 };
    let show_name = right - (name + dot + note) >= need[0];
    let show_note = right - (dot + note) >= need[1];
    let mut x = right;
    if dirty {
        if show_note {
            x = text(ui, pos2(x, cy), Align2::RIGHT_CENTER, tw!("Not saved"), font(12.0), t.text).left() - 5.0;
        } else {
            ui.interact(Rect::from_center_size(pos2(x - 3.0, cy), vec2(14.0, 14.0)), wid("tab-unsaved", ()), Sense::hover())
                .on_hover_text(tw!("Not saved"));
        }
        ui.painter().circle_filled(pos2(x - 3.0, cy), 3.0, t.clay);
        x -= 6.0;
    }
    if show_name {
        x = text(ui, pos2(x - if dirty { 8.0 } else { 0.0 }, cy), Align2::RIGHT_CENTER, d.title(), font(12.0), t.text).left();
    }
    x
}

/// The tool band, the zoom buttons and the screen readout, over the canvas.
fn overlays(app: &mut VectorcraftApp, ui: &mut Ui) {
    if crate::menus::home_showing(app) || app.session.active().is_none() {
        return;
    }
    let Some(canvas) = app.canvas_rect else { return };
    let t = Tokens::get(ui.ctx());
    band(app, ui, canvas, &t);
    zoom_buttons(app, ui, canvas, &t);
    readout(app, ui, canvas, &t);
    if app.session.prefs.canvas_button_row {
        canvas_button_row(app, ui, canvas, &t);
    }
}

/// A card floating over the canvas: its shadow, `surface` fill and edge.
fn card(ui: &Ui, r: Rect, t: &Tokens) {
    // Dark themes have no shadow to show.
    let color = if ui.visuals().dark_mode { Color32::TRANSPARENT } else { t.text.gamma_multiply(0.12) };
    ui.painter().add(egui::epaint::Shadow { offset: [0, 2], blur: 12, spread: 0, color }.as_shape(r, CornerRadius::same(8)));
    field_box(ui, r, t.input, t.input_border);
}

/// Over a card: presses there don't reach the canvas under it.
fn shield(ui: &Ui, id: egui::Id, r: Rect) -> egui::Response {
    ui.interact(r, id, Sense::click_and_drag())
}

/// The band at the top left: the tool in hand, the layer it draws on (and why, when that is not the
/// current layer) and how to use it.
fn band(app: &VectorcraftApp, ui: &Ui, canvas: Rect, t: &Tokens) {
    let tool = app.session.tool_id();
    let (layer, note) = match app.session.active() {
        Some(st) => {
            let target = draw_target(st);
            (target.into.and_then(|id| st.doc.node(id)).map(|n| n.display_name()), target.note)
        }
        None => (None, None),
    };
    let what = match layer {
        Some(l) => format!("{} · {l}", tool_name(tool)),
        None => tool_name(tool),
    };
    let hint = ui.painter().layout_job(hint_job(app, true, 12.0, t.text_dim));
    let what_w = width_of(ui, &what, font(12.0)) + note.as_ref().map_or(0.0, |n| 8.0 + width_of(ui, n, font(12.0)));
    let at = canvas.min + vec2(14.0, 14.0);
    let most = (canvas.width() - 28.0 - 36.0 - 16.0).max(0.0);
    let hint_w = hint.size().x.min((most - (10.0 + 16.0 + 8.0 + what_w + 8.0 + 1.0 + 8.0 + 10.0)).max(0.0));
    let w = 10.0 + 16.0 + 8.0 + what_w + if hint.is_empty() { 10.0 } else { 8.0 + 1.0 + 8.0 + hint_w + 10.0 };
    let r = Rect::from_min_size(at, vec2(w.min(most), 36.0));
    card(ui, r, t);
    shield(ui, wid("band", ()), r);
    let cy = r.center().y;
    if let Some(info) = vectorcraft_tools::tool_info(tool) {
        crate::icons::paint(
            ui,
            crate::icons::tool_icon(info.icon),
            Rect::from_center_size(pos2(r.left() + 10.0 + 8.0, cy), vec2(16.0, 16.0)),
            t.icon,
        );
    }
    let p = ui.painter().with_clip_rect(r.shrink2(vec2(10.0, 0.0)));
    let mut x = p.text(pos2(r.left() + 34.0, cy), Align2::LEFT_CENTER, what, font(12.0), t.text).right() + 8.0;
    if let Some(note) = note {
        x = p.text(pos2(x, cy), Align2::LEFT_CENTER, note, font(12.0), t.clay_text).right() + 8.0;
    }
    if !hint.is_empty() {
        p.vline(x + 0.5, (cy - 8.0)..=(cy + 8.0), Stroke::new(1.0, t.border));
        p.galley(pos2(x + 9.0, cy - hint.size().y / 2.0), hint, t.text_dim);
    }
}

/// Zoom in, zoom out and fit, one above the other at the bottom right.
fn zoom_buttons(app: &mut VectorcraftApp, ui: &Ui, canvas: Rect, t: &Tokens) {
    const CELL: f32 = 36.0;
    let r = Rect::from_min_max(
        pos2(canvas.right() - 16.0 - CELL, canvas.bottom() - 60.0 - 3.0 * CELL),
        pos2(canvas.right() - 16.0, canvas.bottom() - 60.0),
    );
    card(ui, r, t);
    let mut run = None;
    let buttons =
        [("in", "view.zoomIn", tw!("Zoom In")), ("out", "view.zoomOut", tw!("Zoom Out")), ("fit", "view.fitArtboard", tw!("Fit Artboard in Window"))];
    for (i, (key, cmd, label)) in buttons.into_iter().enumerate() {
        let cell = Rect::from_min_size(pos2(r.left(), r.top() + i as f32 * CELL), vec2(CELL, CELL));
        // Presses here don't reach the canvas either.
        let resp = ui.interact(cell, wid("zoom", key), Sense::click_and_drag());
        if resp.hovered() {
            let radius = match i {
                0 => CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 },
                2 => CornerRadius { nw: 0, ne: 0, sw: 8, se: 8 },
                _ => CornerRadius::ZERO,
            };
            ui.painter().rect_filled(cell.shrink(1.0), radius, t.hover);
        }
        if i > 0 {
            ui.painter().hline(cell.x_range().shrink(1.0), cell.top() + 0.5, Stroke::new(1.0, t.border));
        }
        match key {
            "in" => crate::icons::paint(ui, "plus", sharp_square(ui, cell, 15.0), t.text),
            "out" => crate::icons::paint(ui, "minus", sharp_square(ui, cell, 15.0), t.text),
            _ => {
                text(ui, cell.center(), Align2::CENTER_CENTER, tw!("Fit"), font(10.0), t.text);
            }
        }
        let tip = match crate::menus::shortcut_of(cmd) {
            Some(sc) => format!("{label} ({})", crate::menus::pretty_shortcut(sc)),
            None => label.to_string(),
        };
        if resp.on_hover_text(tip).clicked() {
            run = Some(cmd);
        }
    }
    if let Some(cmd) = run {
        crate::menus::invoke(app, cmd, json!({}));
    }
}

/// A `size` square in the middle of `cell`, on whole pixels so a thin icon stays sharp.
fn sharp_square(ui: &Ui, cell: Rect, size: f32) -> Rect {
    let ppp = ui.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    Rect::from_min_size(pos2(snap(cell.center().x - size / 2.0), snap(cell.center().y - size / 2.0)), vec2(size, size))
}

/// The zoom as the screen readout shows it: a whole percentage, with a decimal below 10 %.
fn percent(zoom: f64) -> String {
    let p = zoom * 100.0;
    if p < 9.95 { format!("{p:.1} %") } else { format!("{p:.0} %") }
}

/// The screen readout at the bottom right: the zoom as a percentage and drawing scale.
fn readout(app: &VectorcraftApp, ui: &Ui, canvas: Rect, t: &Tokens) {
    let zoom = percent(app.view().map_or(1.0, |v| v.zoom));
    let label = tw!("Screen");
    let scale_opt = app.session.active().and_then(|st| st.doc.coord.as_ref()).map(|c| c.scale_label());

    let mut w = 10.0 + width_of(ui, label, font(12.0)) + 4.0 + width_of(ui, &zoom, mono(12.0)) + 10.0;
    if let Some(scale_str) = &scale_opt {
        w += 8.0 + width_of(ui, "·", font(12.0)) + 8.0 + width_of(ui, tw!("Drawing"), font(12.0)) + 4.0 + width_of(ui, scale_str, mono(12.0));
    }

    let r = Rect::from_min_max(pos2(canvas.right() - 16.0 - w, canvas.bottom() - 16.0 - 30.0), pos2(canvas.right() - 16.0, canvas.bottom() - 16.0));
    card(ui, r, t);
    shield(ui, wid("readout", ()), r);
    let mut x = text(ui, pos2(r.left() + 10.0, r.center().y), Align2::LEFT_CENTER, label, font(12.0), t.text_dim).right() + 4.0;
    x = text(ui, pos2(x, r.center().y), Align2::LEFT_CENTER, zoom, mono(12.0), t.text).right();
    if let Some(scale_str) = scale_opt {
        x = text(ui, pos2(x + 8.0, r.center().y), Align2::LEFT_CENTER, "·", font(12.0), t.text_dim).right();
        x = text(ui, pos2(x + 8.0, r.center().y), Align2::LEFT_CENTER, tw!("Drawing"), font(12.0), t.text_dim).right() + 4.0;
        text(ui, pos2(x, r.center().y), Align2::LEFT_CENTER, scale_str, mono(12.0), t.text);
    }
}

/// J016 (A5): Canvas Quick Action Button Row for stylus & tablet users.
/// Vertically floating on the left side: Undo, Redo, Pan (Hand Tool toggle), and Tablet Settings.
fn canvas_button_row(app: &mut VectorcraftApp, ui: &Ui, canvas: Rect, t: &Tokens) {
    const CELL: f32 = 32.0;
    let buttons = [
        ("undo", "edit.undo", tw!("Undo"), "undo-2"),
        ("redo", "edit.redo", tw!("Redo"), "redo-2"),
        ("hand", "tool.set hand", tw!("Hand Tool (Pan)"), "hand"),
        ("tablet", "tablet.settings", tw!("Tablet & Pen Settings"), "settings"),
    ];
    let total_h = buttons.len() as f32 * CELL;
    let r = Rect::from_min_size(pos2(canvas.left() + 14.0, (canvas.center().y - total_h / 2.0).max(canvas.top() + 60.0)), vec2(CELL, total_h));
    card(ui, r, t);
    shield(ui, wid("canvas-btn-row", ()), r);

    let mut run_cmd = None;
    let mut open_tablet_settings = false;

    for (i, (key, cmd, label, icon)) in buttons.into_iter().enumerate() {
        let cell = Rect::from_min_size(pos2(r.left(), r.top() + i as f32 * CELL), vec2(CELL, CELL));
        let resp = ui.interact(cell, wid("cbtn", key), Sense::click());
        if resp.hovered() {
            let radius = match i {
                0 => CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 },
                n if n == buttons.len() - 1 => CornerRadius { nw: 0, ne: 0, sw: 8, se: 8 },
                _ => CornerRadius::ZERO,
            };
            ui.painter().rect_filled(cell.shrink(1.0), radius, t.hover);
        }
        if i > 0 {
            ui.painter().hline(cell.x_range().shrink(1.0), cell.top() + 0.5, Stroke::new(1.0, t.border));
        }

        if key == "hand" && app.session.tool_id() == "hand" {
            ui.painter().rect_filled(cell.shrink(1.0), CornerRadius::ZERO, t.row_selected);
        }

        crate::icons::paint(ui, icon, sharp_square(ui, cell, 15.0), t.text);

        if resp.on_hover_text(label).clicked() {
            if key == "tablet" {
                open_tablet_settings = true;
            } else if key == "hand" {
                if app.session.tool_id() == "hand" {
                    run_cmd = Some("tool.set pen");
                } else {
                    run_cmd = Some("tool.set hand");
                }
            } else {
                run_cmd = Some(cmd);
            }
        }
    }

    if let Some(cmd) = run_cmd {
        if let Some(tool_name) = cmd.strip_prefix("tool.set ") {
            app.select_tool(tool_name);
        } else {
            crate::menus::invoke(app, cmd, json!({}));
        }
    }
    if open_tablet_settings {
        super::legend::open_tablet(app, ui.ctx());
    }
}
