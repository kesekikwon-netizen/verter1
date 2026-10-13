//! The legend and layers, the biggest part of the right column (design 2.1 §3, §7): one row per
//! top-level layer, the top one first. A legend row (J005, `vectorcraft_doc::Legend`) is the look of
//! its layer's lines: it shows a sample drawn at its weight, the weight in points, ★ when it is used
//! often, and how many of its lines are set apart from it ("따로"). Rows matching Strata's layers
//! get a section of their own above the others, and the favourites come first in each.
//!
//! A click picks a row for the pen (`legend.pick`), and with a selection tool in hand moves the
//! selection into it; a double click opens its editor and leaves the selection where it was. A
//! locked or hidden row isn't picked: the status bar says why. A row's eye and padlock show, hide,
//! lock and unlock the layer, and its arrow lists the objects on it: a click selects one, and ↺
//! puts a line set apart back to the row's look. The head adds the legend groups, and Weights at
//! Once is at its foot.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_doc::legend::{MM, lines, stroke_of};
use vectorcraft_doc::{Document, Legend, LegendGroup, LegendSample, Node, NodeId};

use super::*;

const ROW_H: f32 = 24.0;
const OBJECT_H: f32 = 20.0;
/// A section's caption in the list: 10 above its text, 14 for it, 4 under it (the mockup's `.cap2`).
const CAPTION_H: f32 = 10.0 + 14.0 + 4.0;
/// The panel's padding at the sides.
const PAD: f32 = 14.0;
/// Its head over the list, and Weights at Once under it ([`footer`]).
const ABOVE: f32 = 32.0;
const BELOW: f32 = 6.0 + FOOT_H;
/// The footer: 6 over its buttons, 24 for them and 12 under them.
const FOOT_H: f32 = 6.0 + 24.0 + 12.0;
/// How tall the legend has to be to list two layers under its caption.
pub(super) const MIN_HEIGHT: f32 = ABOVE + CAPTION_H + 2.0 * ROW_H + BELOW;
/// A row's sample, in front of its name.
const SAMPLE: egui::Vec2 = vec2(20.0, 12.0);
/// How thick a weight (points) is drawn in a sample (design 2.1 §7), straight between these and on
/// past the ends.
const SAMPLE_PX: [(f64, f32); 6] = [(0.1, 0.45), (0.2, 0.6), (0.3, 0.85), (0.5, 1.25), (0.8, 2.0), (1.0, 2.5)];
/// How long a paper millimetre of a line type's dashes is drawn in a sample: long enough to tell
/// the types apart at a glance.
const SAMPLE_MM: f64 = 1.5;
/// The ratio Weights at Once starts with.
pub(super) const RATIO: f64 = 0.8;
/// How soon a second click on a row opens its editor (seconds): a system's usual double click.
const DOUBLE_CLICK: f64 = 0.5;

/// A layer's row as shown.
struct Row<'a> {
    id: NodeId,
    name: String,
    visible: bool,
    locked: bool,
    /// Hidden, or an underlay (a template layer): its name shows faint.
    dim: bool,
    colour: Color32,
    current: bool,
    open: bool,
    /// The current layer's when it is locked or hidden: where lines go instead ([`draw_target`]).
    note: Option<String>,
    /// Its objects, bottom first; an open row lists them under it, top first.
    objects: &'a [Arc<Node>],
    legend: Option<&'a Legend>,
    /// How many of its lines are set apart from its look.
    apart: usize,
}

impl Row<'_> {
    /// How tall the row is with the objects it lists.
    fn height(&self) -> f32 {
        ROW_H + if self.open { self.objects.len() as f32 * OBJECT_H } else { 0.0 }
    }

    fn favourite(&self) -> bool {
        self.legend.is_some_and(|l| l.favourite)
    }
}

/// What the list shows, top to bottom.
enum Item<'a> {
    /// A section's caption.
    Caption(&'static str),
    Layer(Row<'a>),
}

impl Item<'_> {
    fn height(&self) -> f32 {
        match self {
            Item::Caption(_) => CAPTION_H,
            Item::Layer(row) => row.height(),
        }
    }
}

/// What a click on the list asks for.
enum Act {
    Pick(NodeId),
    /// Open a row's editor level with its row's top.
    Edit(NodeId, f32),
    /// List a row's objects under it, or stop.
    Open(NodeId, bool),
    Show(NodeId, bool),
    Lock(NodeId, bool),
    Select(NodeId),
    Revert(NodeId),
}

/// A card beside the right column: a row's editor or Weights at Once.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Card {
    Row(NodeId),
    Weights,
    Coord,
    Tablet,
    UnderlayAlign,
}

/// The card that is open, for which drawing (it closes when another one comes to the front), how
/// far down (a row's editor opens level with its row, when not given as it is laid out; Weights at
/// Once level with the legend's head) and since which pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Open {
    pub(super) card: Card,
    pub(super) doc: u64,
    pub(super) top: Option<f32>,
    pub(super) since: u64,
}

/// Where the open card is kept.
fn card_id() -> egui::Id {
    wid("legend-card", ())
}

/// Open `card` for the active drawing, from `top` (see [`Open`]); Weights at Once opens as it
/// starts.
pub(super) fn open_card(app: &VectorcraftApp, ctx: &egui::Context, card: Card, top: Option<f32>) {
    if let Some(st) = app.session.active() {
        if card == Card::Weights {
            super::weights::forget(ctx);
        }
        if card == Card::Coord {
            super::coord_card::forget(ctx);
        }
        if card == Card::Tablet {
            super::tablet_card::forget(ctx);
        }
        if card == Card::UnderlayAlign {
            super::underlay_card::forget(ctx);
        }
        let since = ctx.cumulative_pass_nr();
        ctx.data_mut(|d| d.insert_temp(card_id(), Open { card, doc: st.uid, top, since }));
    }
}

pub(super) fn open_coord(app: &VectorcraftApp, ctx: &egui::Context) {
    open_card(app, ctx, Card::Coord, None);
}

pub(super) fn open_tablet(app: &VectorcraftApp, ctx: &egui::Context) {
    open_card(app, ctx, Card::Tablet, None);
}

pub(super) fn open_underlay_align(app: &VectorcraftApp, ctx: &egui::Context) {
    open_card(app, ctx, Card::UnderlayAlign, None);
}

/// Show the open card beside the column whose visible part `ui` clips to; `r` is the legend's.
fn show_card(app: &mut VectorcraftApp, ui: &Ui, r: Rect) {
    let Some(open) = ui.data(|d| d.get_temp::<Open>(card_id())) else { return };
    let column = ui.clip_rect();
    let ctx = ui.ctx();
    // Over the canvas at the column's left, within its height and under the tab row.
    let top = app.canvas_rect.map_or(column.top(), |c| c.top().max(column.top()));
    let bounds = Rect::from_min_max(pos2(ctx.content_rect().left(), top), pos2(column.left(), column.bottom()));
    let top = match open.card {
        Card::Row(id) => open.top.or_else(|| ctx.read_response(wid("legend-row", id.0)).map(|row| row.rect.top())),
        Card::Weights | Card::Coord | Card::Tablet | Card::UnderlayAlign => open.top,
    };
    let at = pos2(column.left() - 8.0 - super::card::W, top.unwrap_or(r.top()));
    // A press in the pass it opened in opened it, or came before it: it doesn't put it away.
    let fresh = open.since == ctx.cumulative_pass_nr();
    let place = super::card::Place { at, bounds, fresh };
    let keep = app.session.active().is_some_and(|st| st.uid == open.doc)
        && match open.card {
            Card::Row(id) => super::rowedit::show(app, ctx, place, id),
            Card::Weights => super::weights::show(app, ctx, place),
            Card::Coord => super::coord_card::show(app, ctx, place),
            Card::Tablet => super::tablet_card::show(app, ctx, place),
            Card::UnderlayAlign => super::underlay_card::show(app, ctx, place),
        };
    if !keep {
        ctx.data_mut(|d| d.remove::<Open>(card_id()));
    }
}

pub(super) fn show(app: &mut VectorcraftApp, ui: &mut Ui, r: Rect) {
    let t = Tokens::get(ui.ctx());
    line_below(ui, r, t.border);
    let head = Rect::from_min_size(r.min, vec2(r.width(), 32.0));
    panel_head(ui, head, tw!("Legend · Layers"), "", 2, &t);
    let open = app.session.active().is_some();
    let group = if open { group_button(app, ui, head, &t) } else { None };
    let new_layer = open && head_button(ui, wid("legend-new", ()), head, 1, "plus", tw!("New layer"), &t).clicked();
    let more = more_button(ui, wid("legend-more", ()), head, 0, tw!("Layers panel"), &t).clicked();
    let apart = app.session.active().map(|st| apart(ui, st)).unwrap_or_default();
    let (weights, revert_all) = footer(ui, r, apart.lines.len(), open, &t);
    let area = Rect::from_min_max(pos2(r.left() + PAD, r.top() + ABOVE), pos2(r.right() - PAD, r.bottom() - BELOW));
    let act = list(app, ui, area, &apart, &t);
    if more {
        crate::menus::invoke(app, "window.panel", json!({ "panel": "layers" }));
    }
    if weights {
        open_card(app, ui.ctx(), Card::Weights, None);
    }
    let result = match act {
        Some(Act::Pick(id)) => pick(app, ui.ctx(), id),
        Some(Act::Edit(id, top)) => {
            let undone = unmove(app, ui.ctx(), id);
            // Lines going back change the open rows' heights: the card finds its row where it is
            // laid out next.
            let top = if matches!(undone, Ok(true)) { None } else { Some(top) };
            open_card(app, ui.ctx(), Card::Row(id), top);
            undone.map(drop)
        }
        Some(Act::Open(id, on)) => {
            ui.data_mut(|d| d.insert_temp(opened(id), on));
            Ok(())
        }
        Some(Act::Show(id, on)) => app.run("layer.setProps", json!({ "ids": [id.0], "visible": on })).map(drop),
        Some(Act::Lock(id, on)) => app.run("layer.setProps", json!({ "ids": [id.0], "locked": on })).map(drop),
        Some(Act::Select(id)) => app.run("select.set", json!({ "ids": [id.0] })).map(drop),
        Some(Act::Revert(id)) => revert(app, json!({ "ids": [id.0] })),
        None if revert_all => revert(app, json!({ "all": true })),
        None if new_layer => app.run("layer.new", json!({})).map(drop),
        None => match group {
            Some(g) => add_group(app, g),
            None => Ok(()),
        },
    };
    if let Err(e) = result {
        app.status(e);
    }
    show_card(app, ui, r);
}

/// Why row `id` isn't drawn with: its layer is hidden or locked.
pub(super) fn refused(st: &vectorcraft_engine::DocState, id: NodeId) -> Option<String> {
    let layer = st.doc.node(id)?;
    let why = if !layer.visible {
        tw!("{row} is hidden: show it to draw with it")
    } else if layer.locked {
        tw!("{row} is locked: unlock it to draw with it")
    } else {
        return None;
    };
    Some(crate::i18n::fmt(why, &[("row", &layer.display_name())]))
}

/// Where the last pick that moved the selection is kept: the row, the drawing and the drawing's
/// revision after the move ([`unmove`]).
fn moved_id() -> egui::Id {
    wid("legend-moved", ())
}

/// Pick row `id` for the pen, unless its layer is hidden or locked ([`refused`]). With a selection
/// tool in hand the selection moves into it; a drawing tool's line just drawn stays selected, and
/// stays where it was drawn when the next row is picked.
fn pick(app: &mut VectorcraftApp, ctx: &egui::Context, id: NodeId) -> Result<(), String> {
    ctx.data_mut(|d| d.remove::<(NodeId, u64, u64)>(moved_id()));
    if let Some(why) = app.session.active().and_then(|st| refused(st, id)) {
        return Err(why);
    }
    let tool = app.session.tool_id();
    let carry = (vectorcraft_tools::catalog::is_selection_tool(tool) || matches!(tool, "lasso" | "magicWand"))
        && app.session.active().is_some_and(|st| !st.selection.objects.is_empty());
    let moved = app.run("legend.pick", json!({ "id": id.0, "move": carry }))?["moved"].as_u64().unwrap_or(0);
    if moved > 0
        && let Some(st) = app.session.active()
    {
        ctx.data_mut(|d| d.insert_temp(moved_id(), (id, st.uid, st.revision)));
        let row = st.doc.node(id).map(|n| n.display_name()).unwrap_or_default();
        app.status(crate::i18n::fmt(tw!("Moved {n} to {row}"), &[("n", &moved.to_string()), ("row", &row)]));
    }
    Ok(())
}

/// A double click's first click on row `id` moved the selection into it: take that move back, the
/// row staying the one drawn with, so a double click only opens the row's editor. Nothing is taken
/// back when the drawing changed in between. Returns whether lines went back.
fn unmove(app: &mut VectorcraftApp, ctx: &egui::Context, id: NodeId) -> Result<bool, String> {
    let Some((row, doc, revision)) = ctx.data_mut(|d| d.remove_temp::<(NodeId, u64, u64)>(moved_id())) else { return Ok(false) };
    if row != id || !app.session.active().is_some_and(|st| st.uid == doc && st.revision == revision) {
        return Ok(false);
    }
    app.run("edit.undo", json!({}))?;
    app.run("legend.pick", json!({ "id": id.0 }))?;
    app.status(tw!("The selected lines stay where they were"));
    Ok(true)
}

/// Put lines set apart back to their row's look (`legend.revert` with `params`) and say how many.
pub(super) fn revert(app: &mut VectorcraftApp, params: serde_json::Value) -> Result<(), String> {
    let done = app.run("legend.revert", params)?;
    let (n, skipped) = (done["lines"].as_u64().unwrap_or(0), done["skipped"].as_u64().unwrap_or(0));
    let template =
        if skipped > 0 { tw!("Lines back to the legend: {n} · locked ones left as they were: {m}") } else { tw!("Lines back to the legend: {n}") };
    app.status(crate::i18n::fmt(template, &[("n", &n.to_string()), ("m", &skipped.to_string())]));
    Ok(())
}

/// Add legend group `g`'s rows and draw on with its first ★ row (a plan's 굴광선): the next line
/// goes into the legend, not into a layer outside it.
fn add_group(app: &mut VectorcraftApp, g: LegendGroup) -> Result<(), String> {
    let done = app.run("legend.addGroup", json!({ "group": g.key() }))?;
    let added: Vec<NodeId> = done["added"].as_array().into_iter().flatten().filter_map(Value::as_u64).map(NodeId).collect();
    let first =
        app.session.active().and_then(|st| added.iter().copied().find(|id| st.doc.node(*id).and_then(|l| l.legend()).is_some_and(|l| l.favourite)));
    if let Some(id) = first {
        app.run("legend.pick", json!({ "id": id.0 }))?;
    }
    let n = added.len();
    let note =
        if n > 0 { crate::i18n::fmt(tw!("Legend rows added: {n}"), &[("n", &n.to_string())]) } else { tw!("No legend rows to add").to_string() };
    app.status(note);
    Ok(())
}

/// The lines set apart from their row's look, counted again when the drawing changes.
#[derive(Default)]
pub(super) struct Apart {
    /// How many, by row.
    rows: HashMap<NodeId, usize>,
    pub(super) lines: HashSet<NodeId>,
}

pub(super) fn apart(ui: &Ui, st: &vectorcraft_engine::DocState) -> Arc<Apart> {
    let now = (st.uid, st.revision);
    let id = wid("legend-apart", ());
    if let Some((was, apart)) = ui.data(|d| d.get_temp::<((u64, u64), Arc<Apart>)>(id))
        && was == now
    {
        return apart;
    }
    let mut apart = Apart::default();
    for layer in &st.doc.layers {
        let Some(legend) = layer.legend() else { continue };
        for line in lines(layer) {
            if stroke_of(line).is_some_and(|s| legend.differs(s)) {
                apart.lines.insert(line.id);
                *apart.rows.entry(layer.id).or_default() += 1;
            }
        }
    }
    let apart = Arc::new(apart);
    ui.data_mut(|d| d.insert_temp(id, (now, apart.clone())));
    apart
}

/// The legend group most of the drawing's rows come from, the first in the groups' order (plan,
/// section, finds) when two have as many.
pub(super) fn main_group(doc: &Document) -> Option<LegendGroup> {
    let mut counts = [0usize; 3];
    for g in doc.layers.iter().filter_map(|l| l.legend().and_then(|l| l.group)) {
        if let Some(n) = LegendGroup::ALL.iter().position(|x| *x == g).and_then(|i| counts.get_mut(i)) {
            *n += 1;
        }
    }
    // The last of the most in reverse: the first in the groups' order.
    LegendGroup::ALL.iter().zip(counts).rev().filter(|(_, n)| *n > 0).max_by_key(|(_, n)| *n).map(|(g, _)| *g)
}

/// The head's group button, after its title: the legend group most rows come from ("For Plans ▾",
/// [`main_group`]), or "Add a Legend ▾"; it lists the groups to add. Returns the one picked.
fn group_button(app: &VectorcraftApp, ui: &Ui, head: Rect, t: &Tokens) -> Option<LegendGroup> {
    let label = match app.session.active().and_then(|st| main_group(&st.doc)) {
        Some(LegendGroup::Plan) => tw!("For Plans"),
        Some(LegendGroup::Section) => tw!("For Sections and Elevations"),
        Some(LegendGroup::Finds) => tw!("For Finds"),
        None => tw!("Add a Legend"),
    };
    let cy = head.center().y;
    let x = head.left() + 12.0 + width_of(ui, tw!("Legend · Layers"), font(13.0)) + 6.0;
    // Up to the head's two buttons.
    let room = head.right() - 12.0 - 24.0 * 2.0 - 4.0;
    let r = Rect::from_min_max(pos2(x - 4.0, cy - 10.0), pos2((x + width_of(ui, label, font(11.0)) + 4.0 + 7.0 + 4.0).min(room), cy + 10.0));
    let resp = ui.interact(r, wid("legend-group", ()), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(r, CornerRadius::same(4), t.hover);
    }
    let p = ui.painter().with_clip_rect(r);
    let said = p.text(pos2(x, cy), Align2::LEFT_CENTER, label, font(11.0), t.text_dim);
    if said.right() + 4.0 + 7.0 <= r.right() {
        triangle(ui, pos2(said.right() + 4.0 + 3.5, cy), 7.0, true, t.text_dim);
    }
    let resp = resp.on_hover_text(tw!("Add a legend group's rows"));
    let mut pick = None;
    egui::Popup::menu(&resp).show(|ui| {
        for (g, label) in [
            (LegendGroup::Plan, tw!("Add the Plan Legend")),
            (LegendGroup::Section, tw!("Add the Section and Elevation Legend")),
            (LegendGroup::Finds, tw!("Add the Finds Legend")),
        ] {
            if ui.button(label).clicked() {
                pick = Some(g);
                ui.close();
            }
        }
    });
    pick
}

/// The layers, top first, each followed by its objects when open, in their sections. Only the rows
/// in view are laid out: one layer can hold thousands of objects.
fn list(app: &VectorcraftApp, ui: &mut Ui, area: Rect, apart: &Apart, t: &Tokens) -> Option<Act> {
    let items = items(app, ui.ctx(), apart);
    // Where each item starts, from the top of the list.
    let tops: Vec<f32> = items
        .iter()
        .scan(0.0, |y, item| {
            let top = *y;
            *y += item.height();
            Some(top)
        })
        .collect();
    let total: f32 = items.iter().map(Item::height).sum();
    let selected: HashSet<NodeId> = match app.session.active() {
        Some(st) if items.iter().any(|item| matches!(item, Item::Layer(row) if row.open)) => st.selection.objects.iter().copied().collect(),
        _ => HashSet::new(),
    };
    let lang = crate::i18n::current();
    let click = clicked(ui, area);
    let mut act = None;
    let mut list = ui.new_child(egui::UiBuilder::new().max_rect(area).layout(egui::Layout::top_down(egui::Align::Min)));
    // No fading edges: a faded row reads as a hidden layer.
    list.spacing_mut().scroll.fade.strength = 0.0;
    // Exactly `area`, however short: egui would make it at least 64 tall, over the footer.
    let scroll = egui::ScrollArea::vertical().id_salt(wid("legend-scroll", ())).auto_shrink([false, false]).min_scrolled_height(0.0);
    scroll.show_viewport(&mut list, |ui, view| {
        ui.set_height(total);
        let origin = ui.max_rect().min;
        let w = ui.available_width();
        // The first item in view is the last one starting at or above the view's top.
        let first = tops.partition_point(|top| *top <= view.top()).saturating_sub(1);
        for (item, top) in items.iter().zip(&tops).skip(first) {
            if *top >= view.bottom() {
                break;
            }
            let row = match item {
                Item::Caption(label) => {
                    list_caption(ui, origin.x, origin.x + w, origin.y + top + 10.0 + 7.0, label, t);
                    continue;
                }
                Item::Layer(row) => row,
            };
            let rr = Rect::from_min_size(origin + vec2(0.0, *top), vec2(w, ROW_H));
            if let Some(a) = layer_row(ui, row, rr, t) {
                act = Some(a);
            }
            if !row.open {
                continue;
            }
            // Its objects in view, top first.
            let below = top + ROW_H;
            let from = ((view.top() - below) / OBJECT_H).floor().max(0.0) as usize;
            let to = ((view.bottom() - below) / OBJECT_H).ceil().max(0.0) as usize;
            for (k, node) in row.objects.iter().rev().enumerate().take(to).skip(from) {
                let rr = Rect::from_min_size(origin + vec2(0.0, below + k as f32 * OBJECT_H), vec2(w, OBJECT_H));
                if let Some(a) = object_row(ui, node, selected.contains(&node.id), apart.lines.contains(&node.id), lang, rr, t) {
                    act = Some(a);
                }
            }
        }
        // A second click opens the row first clicked, whatever is under it now.
        if let Some(id) = click.and_then(|c| c.second)
            && let Some(top) = items.iter().zip(&tops).find_map(|(item, top)| matches!(item, Item::Layer(row) if row.id == id).then_some(*top))
        {
            act = Some(Act::Edit(id, origin.y + top));
        }
    });
    if let Some(c) = click {
        // The click a second one may follow: one picking a row.
        let first = match &act {
            Some(Act::Pick(id)) if c.second.is_none() => Some((*id, c.time, c.at)),
            _ => None,
        };
        ui.data_mut(|d| match first {
            Some(first) => {
                d.insert_temp(last_click(), first);
            }
            None => d.remove::<(NodeId, f64, Pos2)>(last_click()),
        });
    }
    act
}

/// A click that ended in this pass: when and where, and the row it opens ([`Act::Edit`]) when it
/// is a second click on the list soon after one picking that row, as near to it as egui's own
/// clicks go. That row, whatever is under the pointer now: the first click's pick may have moved
/// lines out of an open row above, the rows going up under it. Counted here, not by egui, which
/// takes the second click for a third one when another click (✕ on the card) came just before.
#[derive(Clone, Copy)]
struct Click {
    time: f64,
    at: Pos2,
    second: Option<NodeId>,
}

/// Where the click a second one may follow is kept: the row it picked, when and where.
fn last_click() -> egui::Id {
    wid("legend-last-click", ())
}

/// This pass's click, if one ended in it ([`Click`]); `area` is the list's.
fn clicked(ui: &Ui, area: Rect) -> Option<Click> {
    // Where the button came up: a touch or a pen leaves no pointer behind it.
    let (time, at) = ui.input(|i| {
        let up = |e: &egui::Event| match e {
            egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, .. } => Some(*pos),
            _ => None,
        };
        (i.time, if i.pointer.primary_clicked() { i.events.iter().rev().find_map(up) } else { None })
    });
    let at = at?;
    let near = ui.ctx().options(|o| o.input_options.max_click_dist);
    let on_list = ui.clip_rect().intersect(area).contains(at) && ui.ctx().layer_id_at(at) == Some(ui.layer_id());
    let before: Option<(NodeId, f64, Pos2)> = ui.data(|d| d.get_temp(last_click()));
    let second = before.filter(|(_, when, was)| on_list && time - when <= DOUBLE_CLICK && at.distance(*was) <= near).map(|(id, _, _)| id);
    Some(Click { time, at, second })
}

/// The drawing's layers in the legend's sections, top first: those matching Strata's layers, then
/// the others, or all under one caption when none does; ★ rows first in each, then the layers'
/// order.
pub(super) fn sections(doc: &Document) -> Vec<(&'static str, Vec<&Arc<Node>>)> {
    let layers: Vec<&Arc<Node>> = doc.layers.iter().rev().collect();
    let strata = |l: &&Arc<Node>| l.legend().is_some_and(|g| g.strata.is_some());
    let mut sections = if layers.iter().any(strata) {
        let (survey, drawing): (Vec<_>, Vec<_>) = layers.into_iter().partition(strata);
        vec![(tw!("Survey Data · Same Layers as Strata"), survey), (tw!("Drawing · Trowel"), drawing)]
    } else {
        vec![(tw!("This drawing's layers"), layers)]
    };
    for (_, layers) in &mut sections {
        // A stable sort: each part keeps the layers' order.
        layers.sort_by_key(|l| !l.legend().is_some_and(|g| g.favourite));
    }
    sections
}

/// The rows of the active drawing in their [`sections`], each under its caption; a section with
/// no rows is left out after the first.
fn items<'a>(app: &'a VectorcraftApp, ctx: &egui::Context, apart: &Apart) -> Vec<Item<'a>> {
    let Some(st) = app.session.active() else { return vec![Item::Caption(tw!("This drawing's layers"))] };
    let current = st.current_layer();
    let note = draw_target(st).note;
    let row = |l: &'a Arc<Node>| Row {
        id: l.id,
        name: l.display_name(),
        visible: l.visible,
        locked: l.locked,
        dim: !l.visible || l.is_template(),
        colour: crate::panels::layers::layer_colour(&st.doc, l.id),
        current: Some(l.id) == current,
        open: ctx.data(|d| d.get_temp::<bool>(opened(l.id))).unwrap_or(false),
        note: note.clone().filter(|_| Some(l.id) == current),
        objects: l.children().map_or(&[][..], |c| c.as_slice()),
        legend: l.legend(),
        apart: apart.rows.get(&l.id).copied().unwrap_or(0),
    };
    let mut out = vec![];
    for (caption, layers) in sections(&st.doc) {
        if layers.is_empty() && !out.is_empty() {
            continue;
        }
        out.push(Item::Caption(caption));
        out.extend(layers.into_iter().map(|l| Item::Layer(row(l))));
    }
    out
}

/// Where whether a layer's row is open is kept.
fn opened(id: NodeId) -> egui::Id {
    wid("legend-opened", id.0)
}

/// One layer's row in `rr`: its arrow, eye, padlock, sample (or colour), ★, name, how many of its
/// lines are set apart, its weight and its number of objects.
fn layer_row(ui: &Ui, row: &Row, rr: Rect, t: &Tokens) -> Option<Act> {
    let x0 = rr.left() + 4.0;
    let column = |a: f32, b: f32| Rect::from_min_max(pos2(a, rr.top()), pos2(b, rr.bottom()));
    let resp = ui.interact(rr, wid("legend-row", row.id.0), Sense::click());
    let open = ui.interact(column(rr.left(), x0 + 12.5), wid("legend-open", row.id.0), Sense::click());
    let eye = ui.interact(column(x0 + 12.5, x0 + 34.5), wid("legend-eye", row.id.0), Sense::click());
    let lock = ui.interact(column(x0 + 34.5, x0 + 56.5), wid("legend-lock", row.id.0), Sense::click());
    if row.current {
        ui.painter().rect_filled(rr, CornerRadius::same(4), t.row_selected);
    } else if resp.hovered() || open.hovered() || eye.hovered() || lock.hovered() {
        ui.painter().rect_filled(rr, CornerRadius::same(4), t.hover);
    }
    let cy = rr.center().y;
    // As in the mockup: a small arrow; the eye dark when shown, pale when hidden; the padlock
    // black when locked, pale when not.
    triangle(ui, pos2(x0 + 4.5, cy), 7.0, row.open, t.text_dim);
    let icon = |name: &str, cx: f32, color: Color32| crate::icons::paint(ui, name, Rect::from_center_size(pos2(cx, cy), vec2(15.0, 15.0)), color);
    icon(if row.visible { "eye" } else { "eye-off" }, x0 + 23.5, if row.visible { t.icon } else { t.ring });
    icon(if row.locked { "lock" } else { "lock-open" }, x0 + 45.5, if row.locked { t.text } else { t.ring });
    let slot = Rect::from_min_size(pos2(x0 + 60.0, cy - SAMPLE.y / 2.0), SAMPLE);
    match row.legend {
        Some(legend) => sample(ui, slot, legend, t),
        None => {
            ui.painter().rect(slot, 0.0, row.colour.gamma_multiply(0.25), Stroke::new(1.5, row.colour), StrokeKind::Inside);
        }
    }
    if row.favourite() {
        text(ui, pos2(x0 + 87.0, cy), Align2::LEFT_CENTER, "★", font(11.0), t.clay);
    }
    // At the right: the number of objects, the weight, how many lines are set apart.
    let count = row.objects.len().to_string();
    let mut right = rr.right() - 4.0;
    text(ui, pos2(right, cy), Align2::RIGHT_CENTER, &count, mono(10.0), t.text_dim);
    right -= width_of(ui, &count, mono(10.0)).max(16.0) + 7.0;
    if let Some(w) = row.legend.and_then(|l| l.weight) {
        right = text(ui, pos2(right, cy), Align2::RIGHT_CENTER, pt(w), mono(10.0), t.text_dim).left() - 7.0;
    }
    let mut apart = None;
    if row.apart > 0 {
        let label = crate::i18n::fmt(tw!("Apart {n}"), &[("n", &row.apart.to_string())]);
        let b = badge(ui, pos2(right, cy), &label, t);
        right = b.left() - 7.0;
        apart = Some(b);
    }
    // The current row on oat, its name regular as the others'; a favourite's name bold in clay.
    let (f, color) = if row.note.is_some() {
        (font(12.0), t.clay_text)
    } else if row.favourite() {
        (bold(12.0), t.clay_text)
    } else if row.dim {
        (font(12.0), t.text_disabled)
    } else {
        (font(12.0), t.text)
    };
    let name_x = x0 + 104.0;
    text_to(ui, pos2(name_x, cy), right, &row.name, f.clone(), color);
    let open = open.on_hover_text(if row.open { tw!("Hide the objects") } else { tw!("Show the objects") });
    let eye = eye.on_hover_text(if row.visible { tw!("Hide") } else { tw!("Show") });
    let lock = lock.on_hover_text(if row.locked { tw!("Unlock") } else { tw!("Lock") });
    // On hover, over the badge what it counts; else the note or what a click does, after the
    // whole name when it is cut short.
    let tip = if apart.zip(resp.hover_pos()).is_some_and(|(b, p)| b.contains(p)) {
        crate::i18n::fmt(tw!("Lines of this row with a weight or line type of their own: {n}"), &[("n", &row.apart.to_string())])
    } else {
        let tip = row.note.as_deref().unwrap_or(tw!("Draw with this row · double-click to change it"));
        if width_of(ui, &row.name, f) > right - name_x { format!("{}\n{tip}", row.name) } else { tip.to_string() }
    };
    let resp = resp.on_hover_text(tip);
    if open.clicked() {
        Some(Act::Open(row.id, !row.open))
    } else if eye.clicked() {
        Some(Act::Show(row.id, !row.visible))
    } else if lock.clicked() {
        Some(Act::Lock(row.id, !row.locked))
    } else if resp.clicked() {
        // A second click soon after opens the row's editor instead ([`Click`]).
        Some(Act::Pick(row.id))
    } else {
        None
    }
}

/// One object under an open layer's row, in `rr`; a line set apart from the row's look shows its
/// weight and ↺ at the right, which reverts it.
fn object_row(ui: &Ui, node: &Node, selected: bool, apart: bool, lang: crate::i18n::Lang, rr: Rect, t: &Tokens) -> Option<Act> {
    let resp = ui.interact(rr, wid("legend-obj", node.id.0), Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rr, CornerRadius::same(4), t.hover);
    }
    let mut right = rr.right() - 4.0;
    let mut revert = false;
    if let Some(st) = stroke_of(node).filter(|_| apart) {
        let label = crate::i18n::fmt(tw!("{w} Apart ↺"), &[("w", &pt(st.width))]);
        let b = badge(ui, pos2(right, rr.center().y), &label, t);
        revert = ui.interact(b, wid("legend-revert", node.id.0), Sense::click()).on_hover_text(tw!("Back to the row's look")).clicked();
        right = b.left() - 6.0;
    }
    let name = crate::panels::layers::painted_name(node, &node.display_name(), lang);
    // The selected one black and bold; the others in the mockup's ink2, of which `icon` is the
    // nearest token.
    let (f, color) = if selected { (bold(11.0), t.text) } else { (font(11.0), t.icon) };
    text_to(ui, pos2(rr.left() + 36.0, rr.center().y), right, &name, f, color);
    if revert {
        Some(Act::Revert(node.id))
    } else if resp.clicked() {
        Some(Act::Select(node.id))
    } else {
        None
    }
}

/// The mockup's `.ovr` badge: `label` in caution on its wash, edged, ending at `at` (its right
/// centre). Returns where it is.
pub(super) fn badge(ui: &Ui, at: Pos2, label: &str, t: &Tokens) -> Rect {
    let f = font(10.0);
    let r = Rect::from_min_max(pos2(at.x - width_of(ui, label, f.clone()) - 8.0, at.y - 8.0), pos2(at.x, at.y + 8.0));
    ui.painter().rect(r, CornerRadius::same(4), t.caution_wash, Stroke::new(1.0, t.caution_edge), StrokeKind::Inside);
    text(ui, r.center(), Align2::CENTER_CENTER, label, f, t.caution);
    r
}

/// Weights at Once under the list, [`FOOT_H`] tall: "All Weights ×0.8 …" opens its card (dimmed
/// with no drawing `open`), and "Apart N → Legend" reverts every line set apart (dimmed when none
/// is). Returns which was pressed.
fn footer(ui: &Ui, r: Rect, apart: usize, open: bool, t: &Tokens) -> (bool, bool) {
    let foot = Rect::from_min_max(pos2(r.left() + PAD, r.bottom() - FOOT_H), pos2(r.right() - PAD, r.bottom()));
    line_above(ui, foot, t.border);
    let cy = foot.top() + 6.0 + 12.0;
    let x = text(ui, pos2(foot.left(), cy), Align2::LEFT_CENTER, tw!("Weights at once"), font(11.0), t.text_dim).right() + 6.0;
    let label = crate::i18n::fmt(tw!("All Weights ×{r} …"), &[("r", &RATIO.to_string())]);
    let weights = small_button(ui, wid("legend-weights", ()), pos2(x, cy), 24.0, &label, open, t);
    let label = crate::i18n::fmt(tw!("Apart {n} → Legend"), &[("n", &apart.to_string())]);
    let revert = small_button(ui, wid("legend-overrides", ()), pos2(weights.rect.right() + 4.0, cy), 24.0, &label, apart > 0, t);
    let weights = weights.on_hover_text(if open { tw!("Change every line's weight at once") } else { tw!("Open a drawing to change its weights") });
    let revert = revert.on_hover_text(if apart > 0 { tw!("Put every line set apart back to its row's look") } else { tw!("No line is set apart") });
    (weights.clicked(), revert.clicked())
}

/// A weight in points as the legend shows it: one decimal at least, three at most ("1.0", "0.64").
pub(super) fn pt(w: f64) -> String {
    let s = format!("{w:.3}");
    let s = s.trim_end_matches('0');
    if s.ends_with('.') { format!("{s}0") } else { s.to_string() }
}

/// How thick weight `w` (points) is drawn in a sample.
fn sample_px(w: f64) -> f32 {
    if !w.is_finite() {
        return 1.0;
    }
    // The pair of the table holding `w`, or the first or last one carried on past its ends.
    let mut seg = None;
    for pair in SAMPLE_PX.windows(2) {
        if let [a, b] = pair {
            seg = Some((*a, *b));
            if w <= b.0 {
                break;
            }
        }
    }
    let Some(((x0, y0), (x1, y1))) = seg else { return 1.0 };
    (y0 + (y1 - y0) * ((w - x0) / (x1 - x0)) as f32).clamp(0.3, 5.0)
}

/// A dash pattern (points) as drawn in a sample: its dashes and its gaps ([`SAMPLE_MM`] a paper
/// millimetre); `None` when too fine to show.
fn dash_px(pattern: &[f64]) -> Option<(Vec<f32>, Vec<f32>)> {
    let mut px: Vec<f32> = pattern.iter().map(|p| ((p / MM) * SAMPLE_MM).max(0.5) as f32).collect();
    // An odd pattern repeats to make dash and gap pairs, as in PDF and SVG.
    if px.len() % 2 == 1 {
        px.extend(px.clone());
    }
    if px.iter().sum::<f32>() < 2.0 {
        return None;
    }
    Some((px.iter().step_by(2).copied().collect(), px.iter().skip(1).step_by(2).copied().collect()))
}

/// A legend row's sample in `r`: its line or area drawn at its weight in its dashes, or its mark;
/// without a look, a thin faint line or box.
pub(super) fn sample(ui: &Ui, r: Rect, legend: &Legend, t: &Tokens) {
    let look = legend.look();
    let stroke = match look {
        Some((w, _)) => Stroke::new(sample_px(w), t.text),
        None => Stroke::new(1.0, t.text_disabled),
    };
    let dashes = look.and_then(|(_, line)| line.dash_pt()).and_then(|p| dash_px(&p));
    let path = |points: &[Pos2]| match &dashes {
        Some((dash, gap)) => ui.painter().extend(egui::Shape::dashed_line_with_offset(points, stroke, dash, gap, 0.0)),
        None => {
            ui.painter().add(egui::Shape::line(points.to_vec(), stroke));
        }
    };
    match &legend.sample {
        // The datum's △ drawn as the mockup's: 10 × 9 in one point of ink.
        LegendSample::Mark(mark) if mark == "△" => {
            let b = Rect::from_center_size(r.center(), vec2(12.0, 11.0));
            let p = |x: f32, y: f32| b.min + vec2(x, y);
            ui.painter().add(egui::Shape::closed_line(vec![p(6.0, 1.0), p(11.0, 10.0), p(1.0, 10.0)], Stroke::new(1.0, t.text)));
        }
        LegendSample::Mark(mark) => {
            // 12 points, shrunk to fit down to 7.
            let w = width_of(ui, mark, font(12.0));
            let size = if w > 24.0 { (12.0 * 24.0 / w).max(7.0) } else { 12.0 };
            ui.painter().with_clip_rect(r.expand2(vec2(3.0, 2.0))).text(r.center(), Align2::CENTER_CENTER, mark, font(size), t.text);
        }
        LegendSample::Line => path(&[pos2(r.left(), r.center().y), pos2(r.right(), r.center().y)]),
        LegendSample::Area if dashes.is_none() => {
            ui.painter().rect(r, 0.0, t.input, stroke, StrokeKind::Inside);
        }
        LegendSample::Area => {
            ui.painter().rect_filled(r, 0.0, t.input);
            let b = r.shrink(stroke.width / 2.0);
            path(&[b.left_top(), b.right_top(), b.right_bottom(), b.left_bottom(), b.left_top()]);
        }
    }
}
