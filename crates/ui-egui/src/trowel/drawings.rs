//! The drawing set: the open drawings, the current one marked; a click shows one (from the Home
//! screen too) and ＋ starts a new one. Its folders, templates and the scale and coordinate system
//! at the bottom come with J004 and J006.

use serde_json::json;

use super::*;

const HEAD_H: f32 = 32.0;
const ROW_H: f32 = 26.0;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::left(SET).resizable(false).exact_size(SET_W).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_right(ui, r, t.border);
        // The head: its title and ＋ (File › New).
        let head = Rect::from_min_size(r.min, vec2(r.width(), HEAD_H));
        panel_head(ui, head, tw!("Drawing Set"), "", 1, &t);
        if head_button(ui, wid("set-new", ()), head, 0, "plus", tw!("New drawing"), &t).clicked() {
            crate::menus::invoke(app, "file.newDialog", json!({}));
        }
        // The open drawings.
        let cap = head.bottom() + 10.0 + 7.0;
        list_caption(ui, r.left() + 12.0, r.right() - 12.0, cap, tw!("Open Drawings"), &t);
        let foot = footer(app, ui, r, &t);
        let rows = Rect::from_min_max(pos2(r.left(), cap + 9.0), pos2(r.right() - 1.0, foot.top() - 4.0));
        let current = app.session.active_index().filter(|_| !crate::menus::home_showing(app));
        let names: Vec<String> = app.session.documents().iter().map(|d| d.title()).collect();
        let mut open = None;
        let mut list = ui.new_child(egui::UiBuilder::new().max_rect(rows).layout(egui::Layout::top_down(egui::Align::Min)));
        egui::ScrollArea::vertical().id_salt(wid("set-scroll", ())).auto_shrink([false, false]).show(&mut list, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for (i, name) in names.iter().enumerate() {
                let (_, row) = ui.allocate_space(vec2(ui.available_width(), ROW_H));
                let row = Rect::from_min_max(pos2(row.left() + 6.0, row.top()), pos2(row.right() - 6.0, row.bottom()));
                let resp = ui.interact(row, wid("set-row", i), Sense::click());
                let on = current == Some(i);
                if on {
                    ui.painter().rect_filled(row, CornerRadius::same(4), t.row_selected);
                } else if resp.hovered() {
                    ui.painter().rect_filled(row, CornerRadius::same(4), t.hover);
                }
                crate::icons::paint(ui, "tw-plan", Rect::from_center_size(pos2(row.left() + 6.0 + 7.0, row.center().y), vec2(14.0, 14.0)), t.icon);
                ui.painter().with_clip_rect(row).text(pos2(row.left() + 26.0, row.center().y), Align2::LEFT_CENTER, name, font(12.0), t.text);
                if resp.on_hover_text(name).clicked() {
                    open = Some(i);
                }
            }
        });
        if let Some(i) = open {
            app.ui.home = None;
            if let Err(e) = app.run("document.activate", json!({ "index": i })) {
                app.status(e);
            }
        }
    });
}

/// The set's footer at the bottom of `r`: the current drawing's size, its scale and coordinate
/// system (J006). Returns its rect.
fn footer(app: &mut VectorcraftApp, ui: &Ui, r: Rect, t: &Tokens) -> Rect {
    let line = 11.0 * 1.6;
    let foot = Rect::from_min_max(pos2(r.left(), r.bottom() - 8.0 - 3.0 * line - 8.0), r.max);
    line_above(ui, foot, t.border);
    let size = app.session.active().and_then(|d| {
        let ab = d.doc.artboards.get(app.view().map_or(0, |v| v.artboard)).or_else(|| d.doc.artboards.first())?;
        let u = app.session.general_unit();
        Some(format!("{} × {}", u.format(ab.rect.width()), u.format(ab.rect.height())))
    });
    // "This drawing · 400 × 300 mm", the size in ink; the scale and coordinate system with J006.
    let mut y = foot.top() + 8.0 + line / 2.0;
    if let Some(size) = size {
        let x = text(ui, pos2(foot.left() + 12.0, y), Align2::LEFT_CENTER, format!("{} · ", tw!("This drawing")), font(11.0), t.text_dim).right();
        text(ui, pos2(x, y), Align2::LEFT_CENTER, size, mono(11.0), t.text);
    }
    y += line;
    let soon_top = y - line / 2.0;
    let coord_opt = app.session.active().and_then(|st| st.doc.coord.as_ref());
    let (l1, l2) = match coord_opt {
        Some(c) => (format!("{} {}", tw!("Scale"), c.scale_label()), format!("{} {}", tw!("Working"), c.crs_label())),
        None => (tw!("Scale not set").to_string(), tw!("Coordinate system not set").to_string()),
    };
    let color = if coord_opt.is_some() { t.text } else { t.text_dim };
    text(ui, pos2(foot.left() + 12.0, y), Align2::LEFT_CENTER, &l1, font(11.0), color);
    y += line;
    text(ui, pos2(foot.left() + 12.0, y), Align2::LEFT_CENTER, &l2, font(11.0), color);
    y += line;

    let lines = Rect::from_min_max(pos2(foot.left(), soon_top), pos2(foot.right() - 1.0, y - line / 2.0));
    let resp = ui.interact(lines, wid("set-foot", ()), Sense::click());
    if resp.on_hover_text(tw!("Scale and coordinate system")).clicked() {
        super::legend::open_coord(app, ui.ctx());
    }
    foot
}
