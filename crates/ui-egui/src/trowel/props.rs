//! The properties at the bottom of the right column (design 2.1 §3): what is selected and how big
//! it is. The feature's kind, number, period and closed polygon metrics (J009).
//! ⋯ pops the full Properties panel out; the arrow folds this one to its head.

use serde_json::json;
use vectorcraft_geom::Shape;

use super::*;

/// The panel's padding at the sides.
const PAD: f32 = 14.0;
const FIELD_H: f32 = 22.0;

/// Standard archaeological feature kinds and historical periods.
const KINDS: &[&str] = &["주거지", "수혈", "수구", "유구", "석축", "석곽묘", "토광묘", "주혈", "기타"];
const PERIODS: &[&str] = &["청동기", "원삼국", "삼국", "백제", "신라", "가야", "통일신라", "고려", "조선", "미상"];

/// The properties' height: open, or folded to the head.
pub(super) fn height(ctx: &egui::Context) -> f32 {
    if folded(ctx) { 33.0 } else { 136.0 }
}

fn folded(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(folded_id())).unwrap_or(false)
}

fn folded_id() -> egui::Id {
    wid("props-folded", ())
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    let size = app.selection_box().map(|b| (b.rect.width(), b.rect.height()));
    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    panel_head(ui, head, tw!("Properties"), &summary(app), 2, &t);
    let folded = folded(ui.ctx());
    let (icon, tip) = if folded { ("chevron-right", tw!("Show the properties")) } else { ("chevron-down", tw!("Fold the properties")) };
    if head_button(ui, wid("props-fold", ()), head, 0, icon, tip, &t).clicked() {
        ui.data_mut(|d| d.insert_temp(folded_id(), !folded));
    }
    if more_button(ui, wid("props-more", ()), head, 1, tw!("Properties panel"), &t).clicked() {
        crate::menus::invoke(app, "window.panel", json!({ "panel": "properties" }));
    }
    if folded {
        return;
    }
    let left = r.left() + PAD;
    let fields = left + 40.0 + 6.0;

    // Feature metadata and metrics from selection (J009, J010):
    let (cur_kind, cur_num, cur_period, metrics, axes, stratum_info, section_info) = if let Some(st) = app.session.active() {
        let sel_node = st.selection.objects.first().copied().and_then(|id| st.doc.node(id));
        let (k, n, p, maj, min, str_no, str_name, str_munsell) = sel_node
            .and_then(|n| n.attrs.as_deref())
            .map(|a| {
                let k = a.data.iter().find(|(k, _)| k == "feature_kind" || k == "feature:kind").map(|(_, v)| v.clone());
                let n = a.data.iter().find(|(k, _)| k == "feature_number" || k == "feature:number").map(|(_, v)| v.clone());
                let p = a.data.iter().find(|(k, _)| k == "feature_period" || k == "feature:period").map(|(_, v)| v.clone());
                let maj = a.data.iter().find(|(k, _)| k == "feature_major_m").and_then(|(_, v)| v.parse::<f64>().ok());
                let min = a.data.iter().find(|(k, _)| k == "feature_minor_m").and_then(|(_, v)| v.parse::<f64>().ok());
                let s_no = a.data.iter().find(|(k, _)| k == "stratum:number").map(|(_, v)| v.clone());
                let s_name = a.data.iter().find(|(k, _)| k == "stratum:name").map(|(_, v)| v.clone());
                let s_mun = a.data.iter().find(|(k, _)| k == "stratum:munsell").map(|(_, v)| v.clone());
                (k, n, p, maj, min, s_no, s_name, s_mun)
            })
            .unwrap_or((None, None, None, None, None, None, None, None));

        let m = sel_node.and_then(|node| match &node.kind {
            vectorcraft_doc::NodeKind::Path { path, .. } if path.is_closed() => {
                let bp = path.to_bezpath();
                let ppm = st.doc.coord.as_ref().map(|c| c.points_per_meter()).unwrap_or(141.73228);
                let area_m2 = bp.area().abs() / (ppm * ppm);
                let perimeter_m = bp.perimeter(1e-6) / ppm;
                Some((area_m2, perimeter_m))
            }
            _ => None,
        });

        let ax = match (maj, min) {
            (Some(j), Some(n)) => Some((j, n)),
            _ => None,
        };

        let str_info = match (str_no, str_name, str_munsell) {
            (Some(no), name, mun) => Some((no, name.unwrap_or_default(), mun.unwrap_or_default())),
            _ => None,
        };

        let sec_info = st.doc.layers.first().and_then(|layer| layer.attrs.as_deref()).and_then(|a| {
            let name = a.data.iter().find(|(k, _)| k == "section:name").map(|(_, v)| v.clone());
            let datum_el = a.data.iter().find(|(k, _)| k == "section:datum_el").and_then(|(_, v)| v.parse::<f64>().ok());
            if name.is_some() || datum_el.is_some() { Some((name.unwrap_or_else(|| "A-A'".into()), datum_el.unwrap_or(0.0))) } else { None }
        });

        (k, n, p, m, ax, str_info, sec_info)
    } else {
        (None, None, None, None, None, None, None)
    };

    let kind_val = cur_kind.as_deref().unwrap_or(KINDS[0]);
    let num_val = cur_num.map(|n| format!("{n}호")).unwrap_or_else(|| "1호".to_string());
    let period_val = cur_period.as_deref().unwrap_or(PERIODS[0]);

    // Kind and No. row
    let cy = head.bottom() + 2.0 + FIELD_H / 2.0;
    text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Kind"), font(11.0), t.text_dim);
    let mut pick_kind = None;
    let x = field_picker(ui, wid("props-kind", ()), pos2(fields, cy), 112.0, kind_val, true, |ui| {
        for &k in KINDS {
            if ui.selectable_label(k == kind_val, k).clicked() {
                pick_kind = Some(k);
            }
        }
    });

    let mut pick_num = None;
    field_picker(ui, wid("props-number", ()), pos2(x + 6.0, cy), 44.0, &num_val, false, |ui| {
        for n in 1..=12 {
            let s = format!("{n}호");
            if ui.selectable_label(s == num_val, &s).clicked() {
                pick_num = Some(n);
            }
        }
    });

    // Period row
    let cy = cy + FIELD_H + 3.0;
    text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Period"), font(11.0), t.text_dim);
    let mut pick_period = None;
    field_picker(ui, wid("props-period", ()), pos2(fields, cy), 112.0, period_val, true, |ui| {
        for &p in PERIODS {
            if ui.selectable_label(p == period_val, p).clicked() {
                pick_period = Some(p);
            }
        }
    });

    // Apply any property updates
    if pick_kind.is_some() || pick_num.is_some() || pick_period.is_some() {
        let mut params = json!({});
        if let Some(k) = pick_kind {
            params["kind"] = json!(k);
        }
        if let Some(n) = pick_num {
            params["number"] = json!(n);
        }
        if let Some(p) = pick_period {
            params["period"] = json!(p);
        }
        app.run("feature.setProps", params).ok();
    }

    let mut cy = cy + FIELD_H / 2.0 + 5.0 + 6.0;
    if let Some((w, h)) = size {
        let u = app.session.general_unit();
        let x = text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Size"), font(11.0), t.text_dim).right() + 4.0;
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, format!("{} × {}", u.format(w), u.format(h)), mono(11.0), t.text);
        cy += 18.0;
    }
    if let Some((area_m2, perimeter_m)) = metrics {
        let x = text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Plan Area"), font(11.0), t.text_dim).right() + 4.0;
        let x = text(ui, pos2(x, cy), Align2::LEFT_CENTER, format!("{area_m2:.2} ㎡"), mono(11.0), t.text).right() + 8.0;
        let x = text(ui, pos2(x, cy), Align2::LEFT_CENTER, tw!("Perimeter"), font(11.0), t.text_dim).right() + 4.0;
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, format!("{perimeter_m:.2} m"), mono(11.0), t.text);
        cy += 16.0;
    }
    if let Some((major_m, minor_m)) = axes {
        let x = text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Major/Minor"), font(11.0), t.text_dim).right() + 4.0;
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, format!("{major_m:.2} m × {minor_m:.2} m"), mono(11.0), t.text);
        cy += 16.0;
    }
    if let Some((no, name, munsell)) = stratum_info {
        let label = if munsell.is_empty() { format!("{no}호 {name}") } else { format!("{no}호 {name} ({munsell})") };
        let x = text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Stratum"), font(11.0), t.text_dim).right() + 4.0;
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, label, font(11.0), t.text);
        cy += 16.0;
    }
    if let Some((name, datum_el)) = section_info {
        let txt = if datum_el > 0.0 { format!("{name} (EL. {datum_el:.2} m)") } else { name };
        let x = text(ui, pos2(left, cy), Align2::LEFT_CENTER, tw!("Section"), font(11.0), t.text_dim).right() + 4.0;
        text(ui, pos2(x, cy), Align2::LEFT_CENTER, txt, mono(11.0), t.text);
    }
}

/// What is selected: one object's name, how many, or nothing.
fn summary(app: &VectorcraftApp) -> String {
    let Some(st) = app.session.active() else { return String::new() };
    match st.selection.objects.as_slice() {
        [] => tw!("No selection").to_string(),
        [one] => st.doc.node(*one).map(|n| crate::panels::layers::painted_name(n, &n.display_name(), crate::i18n::current())).unwrap_or_default(),
        many => crate::i18n::fmt(tw!("{n} objects"), &[("n", &many.len().to_string())]),
    }
}

/// An interactive picker field box displaying `value` with a drop-down menu.
fn field_picker<R>(ui: &mut Ui, id: egui::Id, at: Pos2, w: f32, value: &str, chevron: bool, menu: impl FnOnce(&mut Ui) -> R) -> f32 {
    let t = Tokens::get(ui.ctx());
    let r = Rect::from_min_size(pos2(at.x, at.y - FIELD_H / 2.0), vec2(w, FIELD_H));
    let resp = ui.interact(r, id, Sense::click());
    let fill = if resp.hovered() { t.hover } else { t.panel_darker };
    ui.painter().rect(r, CornerRadius::same(6), fill, Stroke::new(1.0, t.input_border), StrokeKind::Inside);
    text(ui, pos2(r.left() + 8.0, at.y), Align2::LEFT_CENTER, value, font(11.0), t.text);
    if chevron {
        triangle(ui, pos2(r.right() - 8.0 - 3.5, at.y), 7.0, true, t.text_dim);
    }
    egui::Popup::menu(&resp).show(menu);
    r.right()
}
