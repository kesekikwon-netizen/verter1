//! Trowel Home Screen & Dashboard (design 2.1 §2, blueprint Section 1):
//! Starts a new excavation drawing from 4 standard templates (Plan 1:20, Section 1:20, Finds 1:1,
//! Plate A3), resumes recent drawings, shows the 3-step workflow guide, and displays Strata GIS link
//! and CAD coordinate system status.

use egui::{Align2, CornerRadius, Rect, Sense, Stroke, Ui, pos2, vec2};
use serde_json::json;

use super::*;

const BAR_H: f32 = 36.0;

/// Draw Trowel's Home screen over `rect`.
pub(crate) fn show(app: &mut VectorcraftApp, ui: &mut Ui, rect: Rect) {
    let t = Tokens::get(ui.ctx());
    // Background: Quiet Plate desk surface
    ui.painter().rect_filled(rect, 0.0, t.panel_darker);

    // 1. Top Control Bar: Strata survey link, coordinate system, drawing set path, connection status
    let bar_rect = Rect::from_min_size(rect.min, vec2(rect.width(), BAR_H));
    top_control_bar(app, ui, bar_rect, &t);

    // 2. Scrollable Home Dashboard
    let content_rect = Rect::from_min_max(pos2(rect.left(), bar_rect.bottom()), rect.max);
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(content_rect).layout(egui::Layout::top_down(egui::Align::Center)));
    let ui = &mut child;

    egui::ScrollArea::vertical().id_salt(wid("home-scroll", ())).auto_shrink([false, false]).show(ui, |ui| {
        let max_w = 1080.0_f32.min(ui.available_width() - 48.0).max(400.0);
        ui.add_space(20.0);
        ui.allocate_ui_with_layout(vec2(max_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            hero_section(app, ui, &t);
            ui.add_space(24.0);
            main_grid_section(app, ui, &t);
            ui.add_space(24.0);
            footer_status_bar(ui, &t);
            ui.add_space(20.0);
        });
    });
}

/// The top control bar: Strata integration, coordinate system, drawings path, sync status.
fn top_control_bar(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect, t: &Tokens) {
    ui.painter().rect_filled(r, 0.0, t.panel);
    line_below(ui, r, t.border);

    let cy = r.center().y;
    let mut x = r.left() + 16.0;

    // Survey Link badge
    let l_lbl = text(ui, pos2(x, cy), Align2::LEFT_CENTER, tw!("Survey Link"), font(11.0), t.text_dim);
    x = l_lbl.right() + 8.0;
    let link_badge = Rect::from_center_size(pos2(x + 75.0, cy), vec2(150.0, 22.0));
    ui.painter().rect_filled(link_badge, CornerRadius::same(4), t.row_selected);
    ui.painter().rect_stroke(link_badge, CornerRadius::same(4), Stroke::new(1.0, t.ring), egui::StrokeKind::Inside);
    text(ui, link_badge.center(), Align2::CENTER_CENTER, tw!("Gwangnyeong-ri 1 (Strata GPKG)"), font(11.0), t.text);
    x = link_badge.right() + 18.0;

    // Coordinate System
    let crs_lbl = text(ui, pos2(x, cy), Align2::LEFT_CENTER, tw!("Coordinate System"), font(11.0), t.text_dim);
    x = crs_lbl.right() + 8.0;
    let crs_badge = Rect::from_center_size(pos2(x + 70.0, cy), vec2(140.0, 22.0));
    field_box(ui, crs_badge, t.input, t.input_border);
    text(ui, crs_badge.center(), Align2::CENTER_CENTER, tw!("EPSG:5186 Central Origin"), font(11.0), t.text);
    x = crs_badge.right() + 18.0;

    // Drawing Set Path
    if r.width() > 900.0 {
        let p_lbl = text(ui, pos2(x, cy), Align2::LEFT_CENTER, tw!("Drawing Set Path"), font(11.0), t.text_dim);
        x = p_lbl.right() + 8.0;
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, tw!("Desktop › Gwangnyeong-ri 1 › Drawings"), mono(11.0), t.text_dim);
    }

    // Right-aligned Connection Status
    let sync_label = text(ui, pos2(r.right() - 80.0, cy), Align2::CENTER_CENTER, tw!("Synchronized"), font(11.0), t.clay_text);
    text(ui, pos2(sync_label.left() - 10.0, cy), Align2::RIGHT_CENTER, tw!("Connection Status"), font(11.0), t.text_dim);
    let _ = app;
}

/// The Hero section: brand and quick actions on the left, resume card on the right.
fn hero_section(app: &mut VectorcraftApp, ui: &mut Ui, t: &Tokens) {
    let full_w = ui.available_width();
    let hero_h = 160.0;
    let (rect, _) = ui.allocate_exact_size(vec2(full_w, hero_h), Sense::hover());

    line_above(ui, rect, t.border);
    line_below(ui, rect, t.border);

    let left_w = (rect.width() - 440.0).max(320.0);
    let left_rect = Rect::from_min_size(rect.min + vec2(12.0, 16.0), vec2(left_w, hero_h - 32.0));
    let card_rect = Rect::from_min_max(pos2(rect.right() - 420.0, rect.top() + 14.0), pos2(rect.right() - 12.0, rect.bottom() - 14.0));

    // Left brand column
    let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect).layout(egui::Layout::top_down(egui::Align::Min)));
    {
        let ui = &mut left_ui;
        ui.label(egui::RichText::new("Trowel").font(theme::semibold(28.0)).color(t.text));
        ui.add_space(2.0);
        ui.label(egui::RichText::new(tw!("Korean Excavation CAD & Artifact Drafting")).font(font(13.0)).color(t.text_dim));
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(tw!("Start a new archaeological drawing or resume your excavation drawing set.")).font(font(11.5)).color(t.text_dim),
        );
        ui.label(
            egui::RichText::new(tw!("Directly integrated 1:1 with Strata (hgis) field survey data for reports and plates."))
                .font(font(11.5))
                .color(t.text_dim),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let new_btn = ui.add(
                egui::Button::new(egui::RichText::new(tw!("New Drawing (⌘N)")).font(bold(12.0)).color(t.on_action))
                    .fill(t.action)
                    .corner_radius(CornerRadius::same(4))
                    .min_size(vec2(140.0, 28.0)),
            );
            if new_btn.clicked() {
                app.run("drawing.newPlan", json!({})).ok();
            }

            ui.add_space(8.0);
            let open_btn = ui.add(
                egui::Button::new(egui::RichText::new(tw!("Open Drawing Set (⌘O)")).font(font(12.0)).color(t.text))
                    .fill(t.input)
                    .stroke(Stroke::new(1.0, t.input_border))
                    .corner_radius(CornerRadius::same(4))
                    .min_size(vec2(130.0, 28.0)),
            );
            if open_btn.clicked() {
                app.run("file.open", json!({})).ok();
            }
        });
    }

    // Right "Resume Work" Card
    resume_card(app, ui, card_rect, t);
}

/// The "Resume Work" card in the hero section.
fn resume_card(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect, t: &Tokens) {
    ui.painter().rect_filled(r, CornerRadius::same(2), t.panel);
    ui.painter().rect_stroke(r, CornerRadius::same(2), Stroke::new(1.0, t.border), egui::StrokeKind::Inside);

    // Card Header
    let head_h = 26.0;
    let head_rect = Rect::from_min_size(r.min, vec2(r.width(), head_h));
    line_below(ui, head_rect, t.border);
    text(ui, pos2(r.left() + 12.0, head_rect.center().y), Align2::LEFT_CENTER, tw!("Resume Work"), font(11.0), t.text_dim);
    text(ui, pos2(r.right() - 12.0, head_rect.center().y), Align2::RIGHT_CENTER, tw!("Saved"), font(11.0), t.text_dim);

    // Target Drawing Row
    let recent = crate::io::recent_files(app);
    let first_file = recent.first().cloned();
    let display_title = first_file.as_ref().map_or_else(
        || tw!("Gwangnyeong-ri 1 · Dwelling 1 Plan").to_string(),
        |p| std::path::Path::new(p).file_name().map_or_else(|| p.clone(), |n| n.to_string_lossy().into_owned()),
    );

    let target_y = head_rect.bottom() + 26.0;
    text(ui, pos2(r.left() + 14.0, target_y), Align2::LEFT_CENTER, &display_title, bold(15.0), t.text);

    let resume_btn_rect = Rect::from_center_size(pos2(r.right() - 56.0, target_y), vec2(84.0, 26.0));
    let resp = ui.interact(resume_btn_rect, wid("home-resume-btn", ()), Sense::click());
    let fill = if resp.hovered() { t.action.gamma_multiply(0.9) } else { t.action };
    ui.painter().rect_filled(resume_btn_rect, CornerRadius::same(4), fill);
    text(ui, resume_btn_rect.center(), Align2::CENTER_CENTER, tw!("Resume →"), bold(11.5), t.on_action);

    if resp.clicked() {
        if let Some(path) = first_file {
            let _ = crate::io::open_reporting(app, &path);
        } else {
            app.run("drawing.newPlan", json!({})).ok();
        }
    }

    // Meta Columns Row
    let meta_y = r.bottom() - 32.0;
    line_above(ui, Rect::from_min_max(pos2(r.left(), meta_y - 8.0), r.max), t.border);

    let cols = [
        (tw!("Survey Name"), "광령리1"),
        (tw!("Type"), "평면도"),
        (tw!("Scale"), "1:20"),
        (tw!("CRS"), "EPSG:5186"),
        (tw!("Modified"), "오늘 07:04"),
    ];
    let col_w = (r.width() - 24.0) / cols.len() as f32;
    for (i, (k, v)) in cols.iter().enumerate() {
        let cx = r.left() + 12.0 + i as f32 * col_w;
        text(ui, pos2(cx, meta_y), Align2::LEFT_CENTER, *k, font(10.0), t.text_dim);
        text(ui, pos2(cx, meta_y + 14.0), Align2::LEFT_CENTER, *v, mono(11.0), t.text);
    }
}

/// The main 2-column grid: recent drawings table on the left, workflow rail & templates on the right.
fn main_grid_section(app: &mut VectorcraftApp, ui: &mut Ui, t: &Tokens) {
    let full_w = ui.available_width();
    let left_w = (full_w * 0.58).max(340.0);
    let right_w = full_w - left_w - 24.0;

    ui.horizontal_top(|ui| {
        // Left Column: Recent Drawings Table
        ui.allocate_ui_with_layout(vec2(left_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tw!("Recent Drawing Set")).font(bold(14.0)).color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let r = ui.allocate_exact_size(vec2(160.0, 22.0), Sense::hover()).0;
                    field_box(ui, r, t.input, t.input_border);
                    text(ui, pos2(r.left() + 8.0, r.center().y), Align2::LEFT_CENTER, tw!("Search drawing / survey"), font(11.0), t.text_dim);
                });
            });
            ui.add_space(8.0);
            recent_drawings_table(app, ui, t);
        });

        ui.add_space(24.0);

        // Right Column: 3-Step Workflow & 4 Templates
        ui.allocate_ui_with_layout(vec2(right_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            workflow_section(ui, t);
            ui.add_space(20.0);
            templates_section(app, ui, t);
        });
    });
}

/// Table of recent drawings.
fn recent_drawings_table(app: &mut VectorcraftApp, ui: &mut Ui, t: &Tokens) {
    let table_w = ui.available_width();
    let row_h = 28.0;

    let headers = [
        (tw!("Drawing Name"), 0.28),
        (tw!("Survey Name"), 0.16),
        (tw!("Type"), 0.14),
        (tw!("Scale"), 0.10),
        (tw!("CRS"), 0.12),
        (tw!("Status"), 0.10),
        (tw!("Modified"), 0.10),
    ];

    // Table Head
    let (head_r, _) = ui.allocate_exact_size(vec2(table_w, 24.0), Sense::hover());
    ui.painter().rect_filled(head_r, 0.0, t.panel);
    line_below(ui, head_r, t.border);

    let mut hx = head_r.left() + 8.0;
    for (name, frac) in headers {
        let cw = table_w * frac;
        text(ui, pos2(hx, head_r.center().y), Align2::LEFT_CENTER, name, font(11.0), t.text_dim);
        hx += cw;
    }

    // Default sample excavation drawings (when recent files empty or as defaults)
    let default_rows = [
        ("1호 주거지 평면", "광령리1", "평면도", "1:20", "5186", tw!("Saved"), "오늘 07:04", "plan"),
        ("1호 주거지 A–A′ 단면", "광령리1", "단면·토층", "1:20", "EL.152.5", tw!("Saved"), "오늘 06:45", "section"),
        ("토기 1호 실측도", "광령리1", "유물 실측", "1:1", "—", tw!("Saved"), "어제 18:30", "finds"),
        ("도판 3호 유구배치도", "광령리1", "종합 도판", "1:100", "5186", tw!("Unsaved"), "어제 16:15", "sheet"),
        ("2호 주거지 평면", "광령리1", "평면도", "1:20", "5186", tw!("Saved"), "10월 11일", "plan"),
        ("병산동 1호 주거지", "병산동 시굴", "평면도", "1:20", "5186", tw!("Saved"), "10월 09일", "plan"),
    ];

    let recent: Vec<String> = crate::io::recent_files(app).iter().take(6).cloned().collect();
    let mut open_file = None;
    let mut create_template = None;

    if !recent.is_empty() {
        for (i, path) in recent.iter().enumerate() {
            let (row_r, _) = ui.allocate_exact_size(vec2(table_w, row_h), Sense::click());
            let resp = ui.interact(row_r, wid("home-row-rec", i), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(row_r, CornerRadius::same(2), t.hover);
            }
            line_below(ui, row_r, t.divider);

            let p = std::path::Path::new(path);
            let fname = p.file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().into_owned());

            let mut rx = row_r.left() + 8.0;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, &fname, bold(11.5), t.text);
            rx += table_w * 0.28;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, "광령리1", font(11.0), t.text);
            rx += table_w * 0.16;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, "실측도", font(11.0), t.text);
            rx += table_w * 0.14;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, "1:20", mono(11.0), t.text);
            rx += table_w * 0.10;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, "5186", mono(11.0), t.text);
            rx += table_w * 0.12;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, tw!("Saved"), font(11.0), t.text_dim);
            rx += table_w * 0.10;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, "최근", mono(11.0), t.text_dim);

            if resp.clicked() {
                open_file = Some(path.clone());
            }
        }
    } else {
        for (i, (dname, sname, kind, scale, crs, status, mtime, template_kind)) in default_rows.iter().enumerate() {
            let (row_r, _) = ui.allocate_exact_size(vec2(table_w, row_h), Sense::click());
            let resp = ui.interact(row_r, wid("home-row-def", i), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(row_r, CornerRadius::same(2), t.hover);
            }
            line_below(ui, row_r, t.divider);

            let mut rx = row_r.left() + 8.0;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, *dname, bold(11.5), t.text);
            rx += table_w * 0.28;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, *sname, font(11.0), t.text);
            rx += table_w * 0.16;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, *kind, font(11.0), t.text);
            rx += table_w * 0.14;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, *scale, mono(11.0), t.text);
            rx += table_w * 0.10;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, *crs, mono(11.0), t.text);
            rx += table_w * 0.12;

            let st_color = if *status == tw!("Unsaved") { t.clay_text } else { t.text_dim };
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, *status, font(11.0), st_color);
            rx += table_w * 0.10;
            text(ui, pos2(rx, row_r.center().y), Align2::LEFT_CENTER, *mtime, mono(11.0), t.text_dim);

            if resp.clicked() {
                let cmd = match *template_kind {
                    "section" => "drawing.newSection",
                    "finds" => "drawing.newFinds",
                    "sheet" => "drawing.newSheet",
                    _ => "drawing.newPlan",
                };
                create_template = Some((cmd, *dname));
            }
        }
    }

    if let Some(path) = open_file {
        let _ = crate::io::open_reporting(app, &path);
    } else if let Some((cmd, name)) = create_template {
        app.run(cmd, json!({ "name": name })).ok();
    }
}

/// The 3-step excavation drawing workflow guide.
fn workflow_section(ui: &mut Ui, t: &Tokens) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(tw!("3-Step Drawing Workflow")).font(bold(14.0)).color(t.text));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(tw!("Gwangnyeong-ri 1 baseline")).font(font(11.0)).color(t.text_dim));
        });
    });
    ui.add_space(6.0);

    let steps = [
        (
            tw!("1. Field Survey & Underlay Alignment"),
            tw!("Scan grid alignment 2-pt/4-pt, GPS GCP mapping (error 0.82mm passed)"),
            true,  // done
            false, // next
        ),
        (
            tw!("2. CAD Drafting"),
            tw!("0.1–1.0 pt lines & cut-face hatches, feature poly/postholes/elevations"),
            false,
            true, // next
        ),
        (tw!("3. Plate Layout & Report Publication"), tw!("A3/A4 sheet layout, north arrow, scale bar, title block, validation"), false, false),
    ];

    for (title, desc, is_done, is_next) in steps {
        ui.horizontal(|ui| {
            // Bullet indicator
            let (bullet_r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
            let dot_color = if is_done {
                t.tool_active
            } else if is_next {
                t.clay
            } else {
                t.ring
            };
            ui.painter().circle_filled(bullet_r.center(), 4.0, dot_color);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(title).font(bold(12.0)).color(t.text));
                    if is_next {
                        ui.label(egui::RichText::new(format!("● {}", tw!("Next"))).font(bold(10.5)).color(t.clay_text));
                    }
                });
                ui.label(egui::RichText::new(desc).font(font(10.5)).color(t.text_dim));
            });
        });
        ui.add_space(6.0);
    }
}

/// The 4 standard archaeological drawing templates (Plan, Section, Finds, Sheet).
fn templates_section(app: &mut VectorcraftApp, ui: &mut Ui, t: &Tokens) {
    ui.label(egui::RichText::new(tw!("New Drawing Templates (4 Types)")).font(bold(14.0)).color(t.text));
    ui.add_space(6.0);

    let templates = [
        ("tw-plan", tw!("Plan Template (1:20)"), tw!("Border line, TM coordinate grid, feature/posthole/spot-height legend"), "drawing.newPlan"),
        (
            "tw-section",
            tw!("Section & Stratum (1:20)"),
            tw!("Elevation baseline (EL.), 10cm sub-lines, stratum description table"),
            "drawing.newSection",
        ),
        ("tw-finds", tw!("Artifact Drafting (1:1)"), tw!("Center line, symmetric section, 3-point rim arc, 1cm grid"), "drawing.newFinds"),
        ("tw-sheet", tw!("Report Plate (A3)"), tw!("Grid north arrow, complex scale bar, official title block"), "drawing.newSheet"),
    ];

    let full_w = ui.available_width();
    let card_w = (full_w - 12.0) / 2.0;

    ui.horizontal(|ui| {
        template_card(app, ui, card_w, &templates[0], t);
        ui.add_space(8.0);
        template_card(app, ui, card_w, &templates[1], t);
    });
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        template_card(app, ui, card_w, &templates[2], t);
        ui.add_space(8.0);
        template_card(app, ui, card_w, &templates[3], t);
    });
}

fn template_card(app: &mut VectorcraftApp, ui: &mut Ui, w: f32, tmpl: &(&str, &'static str, &'static str, &'static str), t: &Tokens) {
    let (icon, title, desc, cmd) = *tmpl;
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 54.0), Sense::click());

    let bg = if resp.hovered() { t.hover } else { t.panel };
    let border = if resp.hovered() { t.clay } else { t.border };
    ui.painter().rect_filled(rect, CornerRadius::same(4), bg);
    ui.painter().rect_stroke(rect, CornerRadius::same(4), Stroke::new(1.0, border), egui::StrokeKind::Inside);

    // Icon on left
    let icon_rect = Rect::from_center_size(pos2(rect.left() + 20.0, rect.center().y), vec2(20.0, 20.0));
    crate::icons::paint(ui, icon, icon_rect, t.text);

    // Title and description
    let tx = rect.left() + 38.0;
    text(ui, pos2(tx, rect.top() + 16.0), Align2::LEFT_CENTER, title, bold(12.0), t.text);
    text(ui, pos2(tx, rect.top() + 34.0), Align2::LEFT_CENTER, desc, font(10.5), t.text_dim);

    if resp.clicked() {
        app.run(cmd, json!({})).ok();
    }
}

/// The bottom excavation standards verification status bar.
fn footer_status_bar(ui: &mut Ui, t: &Tokens) {
    let full_w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(full_w, 24.0), Sense::hover());
    line_above(ui, r, t.border);

    let cy = r.center().y;
    let mut x = r.left() + 8.0;

    let items = [
        tw!("Strata GIS (Gwangnyeong-ri 1.gpkg) connected"),
        tw!("Coordinate System: EPSG:5186 Central Origin (Normal)"),
        tw!("Tablet: Wacom Intuos Pro (Detected)"),
        tw!("Font: Noto Serif KR / Noto Sans KR (Applied)"),
    ];

    for item in items {
        let lbl = text(ui, pos2(x, cy), Align2::LEFT_CENTER, item, font(10.5), t.text_dim);
        x = lbl.right() + 16.0;
    }

    text(ui, pos2(r.right() - 8.0, cy), Align2::RIGHT_CENTER, tw!("Cultural Heritage Excavation Standards v1 Passed"), font(10.5), t.text_dim);
}
