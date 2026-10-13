//! The status bar (design 2.1 §3): at the left what is going on (a save running, a note, the last
//! message for a while, else how to use the tool in hand); at the right the tool, Snap, how many
//! drawings have changes not saved, where the pointer is in the drawing, and the drawing's scale
//! and coordinate system (J006).

use super::*;

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom(STATUS).exact_size(STATUS_H).show_separator_line(false).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        line_above(ui, r, t.border);
        let coord_opt = app.session.active().and_then(|st| st.doc.coord.as_ref());
        let (scale_label, scale_color) = match coord_opt {
            Some(c) => (c.summary_label(), t.text),
            None => (tw!("Scale and coordinate system not set").to_string(), t.text_dim),
        };
        let scale = cell(ui, r, r.right(), false, &[(scale_label, font(12.0), scale_color)], &t);
        let resp = ui.interact(scale, wid("status-scale", ()), Sense::click());
        if resp.on_hover_text(tw!("Scale and coordinate system")).clicked() {
            super::legend::open_coord(app, ui.ctx());
        }

        let (x, y) = match (coord_opt, app.hover_doc) {
            (Some(c), Some(p)) => {
                let (tm_x, tm_y) = c.pt_to_tm(p.x, p.y);
                (vectorcraft_doc::format_tm_coord(tm_x), vectorcraft_doc::format_tm_coord(tm_y))
            }
            (None, Some(p)) => {
                let u = app.session.general_unit();
                (u.readout(p.x), u.readout(p.y))
            }
            (_, None) => ("–".to_string(), "–".to_string()),
        };
        let at = cell(
            ui,
            r,
            scale.left(),
            false,
            &[("X".into(), font(12.0), t.text_dim), (x, mono(12.0), t.text), ("Y".into(), font(12.0), t.text_dim), (y, mono(12.0), t.text)],
            &t,
        );
        let unsaved = app.session.documents().iter().filter(|d| d.is_dirty()).count();
        let saved = if unsaved > 0 {
            cell(ui, r, at.left(), true, &[(crate::i18n::fmt(tw!("{n} not saved"), &[("n", &unsaved.to_string())]), font(12.0), t.text)], &t)
        } else {
            cell(ui, r, at.left(), false, &[(tw!("All saved").into(), font(12.0), t.text_dim)], &t)
        };
        let snap = if app.ui.view.snap_to_point { tw!("Snap on") } else { tw!("Snap off") };
        let snap = cell(ui, r, saved.left(), false, &[(snap.into(), font(12.0), t.text)], &t);
        let tool = cell(ui, r, snap.left(), false, &[(tool_name(app.session.tool_id()), font(12.0), t.text)], &t);
        // What is going on, in the room left: a message, else how to use the tool in hand.
        let room = Rect::from_min_max(pos2(r.left() + 16.0, r.top()), pos2(tool.left() - 12.0, r.bottom()));
        let p = ui.painter().with_clip_rect(room);
        match message(app, ui.ctx()) {
            Some(m) => {
                p.text(pos2(room.left(), r.center().y), Align2::LEFT_CENTER, m, font(12.0), t.text);
            }
            None => {
                let hint = p.layout_job(hint_job(app, false, 12.0, t.text_dim));
                p.galley(pos2(room.left(), r.center().y - hint.size().y / 2.0), hint, t.text_dim);
            }
        }
    });
}

/// A save or export running; else a note from something a later job brings ([`soon_note`]); else
/// the last message, for [`MESSAGE_SECS`] after it is put there (the same one again too), so the
/// hint comes back.
fn message(app: &VectorcraftApp, ctx: &egui::Context) -> Option<String> {
    let status = crate::chrome::status_message(app);
    if let Some((running, true)) = status {
        return Some(running);
    }
    if let Some(note) = soon_note(ctx) {
        return Some(note);
    }
    let (message, _) = status?;
    let now = ctx.input(|i| i.time);
    let seen = wid("status-seen", ());
    let since = match ctx.data(|d| d.get_temp::<(u64, String, f64)>(seen)) {
        Some((seq, s, at)) if seq == app.ui.status_seq && s == app.ui.status => at,
        _ => {
            ctx.data_mut(|d| d.insert_temp(seen, (app.ui.status_seq, app.ui.status.clone(), now)));
            now
        }
    };
    let left = MESSAGE_SECS - (now - since);
    // Drawn again when it is due to go.
    (left > 0.0).then(|| {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(left));
        message
    })
}

/// A cell of the status bar `r` ending at `right`: its parts left to right (text, font, colour),
/// after a clay dot when `dot`, and a line at its left. Returns its rect.
fn cell(ui: &Ui, r: Rect, right: f32, dot: bool, parts: &[(String, FontId, Color32)], t: &Tokens) -> Rect {
    let widths: Vec<f32> = parts.iter().map(|(s, f, _)| width_of(ui, s, f.clone())).collect();
    let gaps = 6.0 * parts.len().saturating_sub(1) as f32;
    let inner = widths.iter().sum::<f32>() + gaps + if dot { 12.0 } else { 0.0 };
    let c = Rect::from_min_max(pos2(right - 12.0 - inner - 12.0, r.top()), pos2(right, r.bottom()));
    let cy = r.center().y;
    ui.painter().vline(c.left() + 0.5, (cy - 9.0)..=(cy + 9.0), Stroke::new(1.0, t.border));
    let mut x = c.left() + 12.0;
    if dot {
        ui.painter().circle_filled(pos2(x + 3.0, cy), 3.0, t.clay);
        x += 12.0;
    }
    for ((s, f, color), w) in parts.iter().zip(&widths) {
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, s, f.clone(), *color);
        x += w + 6.0;
    }
    c
}
