//! Legend rows (Trowel, design 2.1 §7): a top-level layer's legend row is the default look of
//! the lines drawn on it, its weight in points and its line type. Lines that follow the look
//! change with the row; a line given another look is set apart ("따로") and keeps it until it is
//! reverted. Picking a row makes it the current layer and gives the pen its look. Change Weights
//! at Once, the legend groups and Add to Legend work on the rows too.

use std::collections::HashSet;

use serde::Serialize;
use serde_json::{Value, json};
use vectorcraft_color::Paint;
use vectorcraft_doc::legend::{MAX_DASHES, MAX_KEY, MAX_MARK, MAX_WEIGHT, clip};
use vectorcraft_doc::{
    Appearance, AppearanceItem, Dash, Document, Hatch, Legend, LegendGroup, LegendSample, LineType, Node, NodeId, NodeKind, StrokeLayer, legend,
};
use vectorcraft_geom::{BezPath, CUT_FACE_STROKE_WIDTH, FillRule, HATCH_GAP_PT, HATCH_STROKE_WIDTH, PathData, PathEl, Shape, hatch_deg};

use super::layer::{apply_options, locked_within, make_layer};
use super::*;

/// The palette's weight chips (points): Change Weights at Once steps along them.
pub const LADDER: [f64; 6] = [0.1, 0.2, 0.3, 0.5, 0.8, 1.0];
/// The longest row name (characters).
const MAX_NAME: usize = 512;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "drawing.cutFace",
            "Cut Face",
            [],
            None,
            "{ids?: [id…]} apply 0.8 pt outer border and 0.1 pt 1.0 mm hatch lines to closed shapes (Trowel); alternating 45°/135° for adjacent faces → {faces, lines, warning?}",
            has_doc,
            cut_face
        ),
        cmd!(
            "legend.set",
            "Set Legend Entry",
            [],
            None,
            "{id?: top-level layer (default: the current layer's), weight?: pt (0–1000)|null (no look), line?: solid|dashed|dotted|dashdot|[dash, gap, …] (pt), favourite?: bool, sample?: line|area|{mark: text}, strata?: Strata layer key|null, group?: plan|section|finds|null, hatch?: none|45|135|alt, clear?: true (remove the row)} a layer's legend row (Trowel): the default look of its lines. A new look restyles the lines that followed the old one (weight and dashes; dash offset, corner alignment, cap and colour stay); lines set apart and locked lines keep theirs, and a row's first look changes no line. A locked row is refused; one undo step → {id, legend, lines}",
            has_doc,
            set
        ),
        cmd!(
            "legend.pick",
            "Use Legend Entry",
            [],
            None,
            "{id: top-level layer, move?: bool} use a legend row (Trowel): it becomes the current layer (not an undo step) and the pen takes its look (weight and dashes); a locked or hidden row is refused (its lines would go to another layer). `move`: the selected objects outside the row move to its top in their order and their lines take its look, one undo step → {id, moved}",
            has_doc,
            pick
        ),
        cmd!(
            "legend.revert",
            "Revert to the Legend",
            [],
            None,
            "{ids?: [id…], all?: bool} lines set apart go back to their row's look (weight and dashes): the lines in `ids` (groups and layers: the lines in them), with `all` every line of the drawing, else the selection's. Lines of rows without a look are left, locked ones are skipped and counted; one undo step. With nothing selected and no ids, the pen takes the current row's look → {lines, skipped}",
            has_doc,
            revert
        ),
        cmd!(
            "legend.weights",
            "Change Weights at Once",
            [],
            None,
            "{how: ratio|value|step, ratio?: factor above 0, from?: pt, to?: pt, step?: 1|-1, overrides?: bool} every row with a look takes the rule (ratio: × factor; value: a weight within 1e-9 pt of `from` becomes `to`; step: the next palette weight 0.1, 0.2, 0.3, 0.5, 0.8, 1.0 up or down, none past the ends) and the lines that follow it its new weight; lines set apart take the rule on their own weight only with `overrides`, lines of layers without a look always. Results round to 0.001 pt within 0–1000; hidden lines change, locked ones are skipped and counted; one undo step → {rows: [{id, name, from, to}], lines, skipped}",
            has_doc,
            weights
        ),
        cmd!(
            "legend.addGroup",
            "Add Legend Group",
            [],
            None,
            "{group: plan|section|finds} add a legend group's rows (Trowel design 2.1 §7) as layers on top of the drawing, its first row topmost; a row whose Strata key or name (trimmed) a top-level layer already has is skipped. Nothing added: no undo step → {added: [id…], skipped}",
            has_doc,
            add_group
        ),
        cmd!(
            "legend.add",
            "Add to Legend",
            [],
            None,
            "{name?} a new legend row directly above the current layer, made current, with the look of the first selected line (its top stroke's weight and line type; a closed path or compound shows as an area), else the pen's. The selected objects move into it and keep their look; one undo step → {id, moved}",
            has_doc,
            add
        ),
    ]
}

// ---------- params ----------

/// An optional `true`/`false` param (anything else is an error).
fn flag(p: &Value, key: &str, cmd: &str) -> Result<Option<bool>> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(v) => Err(bad(cmd, format!("`{key}` is true or false, not {v}"))),
    }
}

/// The row a command acts on: `id`, which must be a top-level layer, else the current layer's
/// top-level layer.
fn row_param(s: &Session, p: &Value, cmd: &str) -> Result<NodeId> {
    let st = s.doc()?;
    let id = match p.get("id") {
        None | Some(Value::Null) => st.current_layer().and_then(|l| st.doc.layer_of(l)).ok_or_else(|| bad(cmd, "no current layer"))?,
        Some(v) => v.as_u64().map(NodeId).ok_or_else(|| bad(cmd, format!("`id` is a layer id, not {v}")))?,
    };
    if !st.doc.node(id).is_some_and(Node::is_layer) || st.doc.parent_of(id).is_some() {
        return Err(bad(cmd, format!("{} is not a top-level layer: legend rows are top-level layers", id.0)));
    }
    Ok(id)
}

/// `weight`: points (clamped to 0–1000) or null for no look.
fn weight_param(p: &Value, cmd: &str) -> Result<Option<Option<f64>>> {
    match p.get("weight") {
        None => Ok(None),
        Some(Value::Null) => Ok(Some(None)),
        Some(v) => v
            .as_f64()
            .filter(|w| w.is_finite())
            .map(|w| Some(Some(w.clamp(0.0, MAX_WEIGHT))))
            .ok_or_else(|| bad(cmd, format!("`weight` is a number of points or null, not {v}"))),
    }
}

/// `line`: a line type's name or dash lengths in points that draw dashes.
fn line_param(p: &Value, cmd: &str) -> Result<Option<LineType>> {
    let Some(v) = p.get("line") else { return Ok(None) };
    let t = match v {
        Value::String(name) => match name.as_str() {
            "solid" => LineType::Solid,
            "dashed" => LineType::Dashed,
            "dotted" => LineType::Dotted,
            "dashdot" => LineType::DashDot,
            _ => return Err(bad(cmd, format!("unknown line type `{name}`: solid, dashed, dotted, dashdot or dash lengths"))),
        },
        Value::Array(a) if (1..=MAX_DASHES).contains(&a.len()) => {
            let lengths: Option<Vec<f64>> = a.iter().map(|x| x.as_f64().filter(|x| x.is_finite())).collect();
            let lengths = lengths.ok_or_else(|| bad(cmd, format!("dash lengths are numbers, not {v}")))?;
            match LineType::from_dash(&lengths) {
                LineType::Solid => return Err(bad(cmd, format!("{v} draws no dashes: lengths are 0 or more and one is more than 0"))),
                t => t,
            }
        }
        _ => return Err(bad(cmd, format!("`line` is solid, dashed, dotted, dashdot or 1 to {MAX_DASHES} dash lengths, not {v}"))),
    };
    Ok(Some(t))
}

/// `sample`: `"line"`, `"area"` or `{"mark": text}`.
fn sample_param(p: &Value, cmd: &str) -> Result<Option<LegendSample>> {
    let Some(v) = p.get("sample") else { return Ok(None) };
    let mark = v.get("mark").and_then(Value::as_str).map(str::trim).filter(|m| !m.is_empty());
    match (v.as_str(), mark) {
        (Some("line"), _) => Ok(Some(LegendSample::Line)),
        (Some("area"), _) => Ok(Some(LegendSample::Area)),
        (None, Some(m)) if v.as_object().is_some_and(|o| o.len() == 1) => Ok(Some(LegendSample::Mark(clip(m, MAX_MARK)))),
        _ => Err(bad(cmd, format!("`sample` is \"line\", \"area\" or {{\"mark\": text}}, not {v}"))),
    }
}

/// `strata`: a Strata layer key or null.
fn strata_param(p: &Value, cmd: &str) -> Result<Option<Option<String>>> {
    match p.get("strata") {
        None => Ok(None),
        Some(Value::Null) => Ok(Some(None)),
        Some(Value::String(k)) if !k.trim().is_empty() => Ok(Some(Some(clip(k, MAX_KEY)))),
        Some(v) => Err(bad(cmd, format!("`strata` is a Strata layer key or null, not {v}"))),
    }
}

/// `group`: plan, section, finds or null.
fn group_param(p: &Value, key: &str, cmd: &str) -> Result<Option<Option<LegendGroup>>> {
    match p.get(key) {
        None => Ok(None),
        Some(Value::Null) => Ok(Some(None)),
        Some(v) => v
            .as_str()
            .and_then(LegendGroup::parse)
            .map(|g| Some(Some(g)))
            .ok_or_else(|| bad(cmd, format!("`{key}` is plan, section or finds, not {v}"))),
    }
}

/// `hatch`: none, 45, 135, alt.
fn hatch_param(p: &Value, cmd: &str) -> Result<Option<Hatch>> {
    let Some(v) = p.get("hatch") else { return Ok(None) };
    let h = match v.as_str() {
        Some("none") | Some("") => Hatch::None,
        Some("45") | Some("deg45") => Hatch::Deg45,
        Some("135") | Some("deg135") => Hatch::Deg135,
        Some("alt") | Some("alternate") => Hatch::Alternate,
        _ => return Err(bad(cmd, format!("`hatch` is \"none\", \"45\", \"135\" or \"alt\", not {v}"))),
    };
    Ok(Some(h))
}

// ---------- looks ----------

/// A row's look, when it has one.
fn look_of(n: &Node) -> Option<(f64, LineType)> {
    n.legend().and_then(|l| l.look().map(|(w, t)| (w, t.clone())))
}

/// Give a stroke a row's look: its weight, and its line type's dashes (a new pattern keeps the
/// dash offset and corner alignment, as in `stroke.set`). Cap, join, colour and the rest stay.
fn restyle(st: &mut StrokeLayer, w: f64, line: &LineType) {
    st.width = w;
    if !line.same(&LineType::of(st)) {
        st.dash = line.dash_pt().map(|pattern| Dash { pattern, ..st.dash.take().unwrap_or_default() });
    }
}

/// Restyle line `id`, unless it or a container around it is locked; whether it was.
fn restyle_line(d: &mut Document, id: NodeId, w: f64, line: &LineType) -> bool {
    if locked_within(d, id) {
        return false;
    }
    let Some(st) = d.node_mut(id).and_then(|n| n.appearance.stroke_mut()) else { return false };
    restyle(st, w, line);
    true
}

/// The lines of `n`: itself when it is a line, else the lines in it (top first).
fn lines_in(n: &Node) -> Vec<&Node> {
    if legend::is_line(n) { vec![n] } else { legend::lines(n) }
}

/// The pen (the next object drawn) takes a look.
fn set_pen(s: &mut Session, w: f64, line: &LineType) {
    s.paint.stroke_width = w;
    if let Some(st) = s.new_art_stroke_mut() {
        restyle(st, w, line);
    }
}

fn set_row(d: &mut Document, id: NodeId, row: Option<Legend>) {
    if let Some(NodeKind::Layer { legend, .. }) = d.node_mut(id).map(|n| &mut n.kind) {
        *legend = row.map(Box::new);
    }
}

// ---------- commands ----------

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "legend.set";
    let weight = weight_param(p, C)?;
    let line = line_param(p, C)?;
    let favourite = flag(p, "favourite", C)?;
    let sample = sample_param(p, C)?;
    let strata = strata_param(p, C)?;
    let group = group_param(p, "group", C)?;
    let hatch = hatch_param(p, C)?;
    let clear = match p.get("clear") {
        None => false,
        Some(Value::Bool(true)) => true,
        Some(v) => return Err(bad(C, format!("`clear` is true, not {v}"))),
    };
    if !clear
        && weight.is_none()
        && line.is_none()
        && favourite.is_none()
        && sample.is_none()
        && strata.is_none()
        && group.is_none()
        && hatch.is_none()
    {
        return Err(bad(C, "nothing to set: weight, line, favourite, sample, strata, group, hatch or clear"));
    }
    let id = row_param(s, p, C)?;
    let d = &s.doc()?.doc;
    let n = d.node(id).ok_or(EngineError::NoNode(id))?;
    if n.locked {
        return Err(bad(C, format!("the layer `{}` is locked", n.display_name())));
    }
    let old = n.legend().cloned();
    let new = (!clear).then(|| {
        let mut row = old.clone().unwrap_or_default();
        if let Some(w) = weight {
            row.weight = w;
        }
        if let Some(t) = line {
            row.line = t;
        }
        if let Some(f) = favourite {
            row.favourite = f;
        }
        if let Some(x) = sample {
            row.sample = x;
        }
        if let Some(k) = strata {
            row.strata = k;
        }
        if let Some(g) = group {
            row.group = g;
        }
        if let Some(h) = hatch {
            row.hatch = h;
        }
        row
    });
    // The lines that followed the old look take the new one (a first look changes none).
    let new_look = match (old.as_ref().and_then(Legend::look), new.as_ref().and_then(Legend::look)) {
        (Some((ow, ot)), Some((nw, nt))) => (ow - nw).abs() > legend::SAME || !ot.same(nt),
        _ => false,
    };
    let follow: Vec<NodeId> = match &old {
        Some(o) if new_look => legend::lines(n).into_iter().filter(|l| legend::stroke_of(l).is_some_and(|st| !o.differs(st))).map(|l| l.id).collect(),
        _ => vec![],
    };
    let look = new.as_ref().and_then(|r| r.look().map(|(w, t)| (w, t.clone())));
    let lines = s.edit("Set Legend Entry", |d, _| {
        let mut changed = 0;
        if let Some((w, t)) = &look {
            for l in &follow {
                changed += usize::from(restyle_line(d, *l, *w, t));
            }
        }
        set_row(d, id, new.clone());
        Ok(changed)
    })?;
    Ok(json!({ "id": id.0, "legend": new, "lines": lines }))
}

fn pick(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "legend.pick";
    if p.get("id").is_none_or(Value::is_null) {
        return Err(bad(C, "missing id"));
    }
    let id = row_param(s, p, C)?;
    let mv = flag(p, "move", C)?.unwrap_or(false);
    let st = s.doc()?;
    // Its lines would go to another layer, without its look.
    if !st.doc.is_editable(id) {
        return Err(bad(C, "the row's layer is locked or hidden: unlock or show it to draw on it"));
    }
    let look = st.doc.node(id).and_then(look_of);
    let mut moved = 0;
    if mv {
        let roots: Vec<NodeId> =
            super::edit::selected_roots(s)?.into_iter().filter(|r| st.doc.layer_of(*r) != Some(id) && st.doc.is_editable(*r)).collect();
        if !roots.is_empty() {
            moved = s.edit("Use Legend Entry", |d, _| {
                for r in &roots {
                    d.move_node(*r, Some(id), usize::MAX)?;
                    if let Some((w, t)) = &look {
                        let ids: Vec<NodeId> = d.node(*r).map(|n| lines_in(n).iter().map(|l| l.id).collect()).unwrap_or_default();
                        for l in ids {
                            restyle_line(d, l, *w, t);
                        }
                    }
                }
                Ok(roots.len())
            })?;
        }
    }
    let st = s.doc_mut()?;
    st.active_layer = Some(id);
    st.layer_rows = vec![id];
    st.revision += 1;
    if let Some((w, t)) = &look {
        set_pen(s, *w, t);
    }
    Ok(json!({ "id": id.0, "moved": moved }))
}

fn revert(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "legend.revert";
    let all = flag(p, "all", C)?.unwrap_or(false);
    let ids = match p.get("ids") {
        None | Some(Value::Null) => None,
        Some(_) => {
            let mut ids = checked_ids_param(s, p, "ids", C)?;
            ids.truncate(super::layer::MAX_ROWS);
            Some(ids)
        }
    };
    let st = s.doc()?;
    let d = &st.doc;
    let starts: Vec<NodeId> = match (all, ids) {
        (true, _) => d.layers.iter().map(|l| l.id).collect(),
        (false, Some(ids)) => ids,
        (false, None) => super::edit::selected_roots(s)?,
    };
    if !all && p.get("ids").is_none_or(Value::is_null) && starts.is_empty() {
        // Nothing to revert: the pen takes the current row's look.
        let look = st.current_layer().and_then(|l| d.layer_of(l)).and_then(|r| d.node(r)).and_then(look_of);
        if let Some((w, t)) = look {
            set_pen(s, w, &t);
        }
        return Ok(json!({ "lines": 0, "skipped": 0 }));
    }
    let mut seen = HashSet::new();
    let (mut todo, mut skipped) = (vec![], 0);
    for start in starts {
        let Some(row) = d.layer_of(start).and_then(|r| d.node(r)) else { continue };
        let (Some(legend), Some((w, t))) = (row.legend(), look_of(row)) else { continue };
        let Some(n) = d.node(start) else { continue };
        for l in lines_in(n) {
            if !seen.insert(l.id) || !legend::stroke_of(l).is_some_and(|st| legend.differs(st)) {
                continue;
            }
            if locked_within(d, l.id) {
                skipped += 1;
            } else {
                todo.push((l.id, w, t.clone()));
            }
        }
    }
    if !todo.is_empty() {
        s.edit("Revert to the Legend", |d, _| {
            for (l, w, t) in &todo {
                restyle_line(d, *l, *w, t);
            }
            Ok(())
        })?;
    }
    Ok(json!({ "lines": todo.len(), "skipped": skipped }))
}

/// A row whose weight Change Weights at Once changes.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RowChange {
    pub id: u64,
    pub name: String,
    pub from: f64,
    pub to: f64,
}

/// What Change Weights at Once does: the rows and lines it changes (each line's new weight) and
/// the locked lines it leaves.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeightPlan {
    pub rows: Vec<RowChange>,
    pub lines: Vec<(NodeId, f64)>,
    pub skipped: usize,
}

/// How Change Weights at Once changes a weight.
#[derive(Clone, Copy, Debug)]
enum Rule {
    Ratio(f64),
    Value { from: f64, to: f64 },
    Step { up: bool },
}

impl Rule {
    fn parse(p: &Value, cmd: &str) -> Result<Rule> {
        let finite = |k: &str| p.get(k).and_then(Value::as_f64).filter(|v| v.is_finite());
        match str_param(p, "how") {
            Some("ratio") => finite("ratio").filter(|r| *r > 0.0).map(Rule::Ratio).ok_or_else(|| bad(cmd, "`ratio` is a number above 0")),
            Some("value") => match (finite("from"), finite("to")) {
                (Some(from), Some(to)) => Ok(Rule::Value { from, to }),
                _ => Err(bad(cmd, "`value` takes the weights `from` and `to` (pt)")),
            },
            Some("step") => match p.get("step").and_then(Value::as_f64) {
                Some(1.0) => Ok(Rule::Step { up: true }),
                Some(-1.0) => Ok(Rule::Step { up: false }),
                _ => Err(bad(cmd, "`step` is 1 (heavier) or -1 (lighter)")),
            },
            _ => Err(bad(cmd, "`how` is ratio, value or step")),
        }
    }

    /// The new weight for `w`, rounded to 0.001 pt within 0–1000; `None` when it stays.
    fn apply(self, w: f64) -> Option<f64> {
        if !w.is_finite() {
            return None;
        }
        let to = match self {
            Rule::Ratio(r) => w * r,
            Rule::Value { from, to } => {
                if (w - from).abs() > legend::SAME {
                    return None;
                }
                to
            }
            Rule::Step { up: true } => LADDER.into_iter().find(|v| *v > w + legend::SAME)?,
            Rule::Step { up: false } => LADDER.into_iter().rev().find(|v| *v < w - legend::SAME)?,
        };
        let to = (to.clamp(0.0, MAX_WEIGHT) * 1000.0).round() / 1000.0;
        ((to - w).abs() > legend::SAME).then_some(to)
    }
}

/// Change Weights at Once with these params (as `legend.weights`), without changing the
/// document: its preview.
pub fn plan_weights(d: &Document, p: &Value) -> Result<WeightPlan> {
    const C: &str = "legend.weights";
    let rule = Rule::parse(p, C)?;
    let overrides = flag(p, "overrides", C)?.unwrap_or(false);
    let mut plan = WeightPlan::default();
    for layer in d.layers.iter().rev() {
        let row = layer.legend().filter(|l| l.look().is_some());
        let row_to = row.and_then(|l| l.weight).and_then(|w| rule.apply(w).map(|to| (w, to)));
        if let Some((from, to)) = row_to.filter(|_| !layer.locked) {
            plan.rows.push(RowChange { id: layer.id.0, name: layer.display_name().to_string(), from, to });
        }
        for l in legend::lines(layer) {
            let Some(st) = legend::stroke_of(l) else { continue };
            let to = match row {
                // It follows the row: the row's new weight.
                Some(r) if !r.differs(st) => row_to.map(|(_, to)| to).filter(|to| (to - st.width).abs() > legend::SAME),
                Some(_) if !overrides => None,
                _ => rule.apply(st.width),
            };
            let Some(to) = to else { continue };
            if locked_within(d, l.id) {
                plan.skipped += 1;
            } else {
                plan.lines.push((l.id, to));
            }
        }
    }
    Ok(plan)
}

fn weights(s: &mut Session, p: &Value) -> Result<Value> {
    let plan = plan_weights(&s.doc()?.doc, p)?;
    if !plan.rows.is_empty() || !plan.lines.is_empty() {
        s.edit("Change Weights at Once", |d, _| {
            for r in &plan.rows {
                let id = NodeId(r.id);
                let row = d.node(id).and_then(Node::legend).cloned().map(|l| Legend { weight: Some(r.to), ..l });
                set_row(d, id, row);
            }
            for (l, to) in &plan.lines {
                if let Some(st) = d.node_mut(*l).and_then(|n| n.appearance.stroke_mut()) {
                    st.width = *to;
                }
            }
            Ok(())
        })?;
    }
    Ok(json!({ "rows": plan.rows, "lines": plan.lines.len(), "skipped": plan.skipped }))
}

/// How a group row shows: a line, an area or a mark.
#[derive(Clone, Copy)]
enum Show {
    Line,
    Area,
    Mark(&'static str),
}

/// A group row's layer options.
#[derive(Clone, Copy, PartialEq)]
enum Opts {
    Plain,
    /// A template layer (locked, images dimmed), as Layer Options' Template.
    Template,
    HiddenTemplate,
    NoPrint,
}

/// A group row: name, weight (pt), line type, how it shows, Strata key, ★ and layer options.
type GroupRow = (&'static str, Option<f64>, LineType, Show, Option<&'static str>, bool, Opts);

/// Design 2.1 §7's legend groups, first row topmost. Weights in points; the Strata keys are
/// hgis `data/schemas/ka_hgis_layers.yaml`'s.
const PLAN: [GroupRow; 15] = [
    ("조사구역 경계", Some(1.0), LineType::DashDot, Show::Area, Some("survey_area"), false, Opts::Plain),
    ("시굴격자", Some(0.5), LineType::Solid, Show::Area, Some("trial_trench"), false, Opts::Plain),
    ("굴광선 (유구 면)", Some(0.8), LineType::Solid, Show::Area, Some("feature_poly"), true, Opts::Plain),
    ("바닥선", Some(0.5), LineType::Solid, Show::Area, None, true, Opts::Plain),
    ("유구 선 (구·벽구)", Some(0.5), LineType::Solid, Show::Line, Some("feature_line"), false, Opts::Plain),
    ("단면 위치선", Some(0.2), LineType::DashDot, Show::Line, Some("section_line"), true, Opts::Plain),
    ("기준점", None, LineType::Solid, Show::Mark("△"), Some("control_points"), false, Opts::Plain),
    ("유물 출토 위치", Some(0.2), LineType::Solid, Show::Mark("③"), Some("artifact_point"), false, Opts::Plain),
    ("내부 시설", Some(0.5), LineType::Solid, Show::Area, None, false, Opts::Plain),
    ("돌", Some(0.5), LineType::Solid, Show::Area, None, false, Opts::Plain),
    ("경사 표시", Some(0.2), LineType::Solid, Show::Area, None, false, Opts::Plain),
    ("표고점", None, LineType::Solid, Show::Mark("·152"), None, false, Opts::Plain),
    ("격자 · 도곽", Some(0.1), LineType::Solid, Show::Line, None, false, Opts::Plain),
    ("글자 · 번호", None, LineType::Solid, Show::Mark("가"), None, false, Opts::Plain),
    ("밑그림", None, LineType::Solid, Show::Area, None, false, Opts::Template),
];

const SECTION: [GroupRow; 12] = [
    ("단면선 (잘린 땅)", Some(0.8), LineType::Solid, Show::Line, None, true, Opts::Plain),
    ("잘린 돌·유물·기와", Some(0.8), LineType::Solid, Show::Area, None, true, Opts::Plain),
    ("보이는 돌·유물", Some(0.5), LineType::Solid, Show::Area, None, false, Opts::Plain),
    ("해발선 50 cm", Some(0.5), LineType::Solid, Show::Line, None, true, Opts::Plain),
    ("해발선 10 cm", Some(0.2), LineType::Solid, Show::Line, None, false, Opts::Plain),
    ("해발 숫자", None, LineType::Solid, Show::Mark("152.5m"), None, false, Opts::Plain),
    ("토층선", Some(0.5), LineType::Solid, Show::Line, None, true, Opts::Plain),
    ("불분명한 토층선", Some(0.5), LineType::Dotted, Show::Line, None, false, Opts::Plain),
    ("교란", Some(0.5), LineType::Solid, Show::Area, None, false, Opts::Plain),
    ("생토", None, LineType::Solid, Show::Area, None, false, Opts::Plain),
    ("토층 번호", None, LineType::Solid, Show::Mark("①"), None, false, Opts::Plain),
    ("desca 원본", None, LineType::Solid, Show::Line, None, false, Opts::HiddenTemplate),
];

const FINDS: [GroupRow; 7] = [
    ("외형선", Some(0.8), LineType::Solid, Show::Line, None, true, Opts::Plain),
    ("단면", Some(0.8), LineType::Solid, Show::Area, None, true, Opts::Plain),
    ("내면선", Some(0.5), LineType::Solid, Show::Line, None, false, Opts::Plain),
    ("중심선", Some(0.2), LineType::DashDot, Show::Line, None, false, Opts::Plain),
    ("문양 · 조정흔", Some(0.2), LineType::Solid, Show::Line, None, false, Opts::Plain),
    ("복원선", Some(0.5), LineType::Dashed, Show::Line, None, false, Opts::Plain),
    ("계측선", None, LineType::Solid, Show::Line, None, false, Opts::NoPrint),
];

/// A legend group's rows, first row topmost.
pub fn group_rows(g: LegendGroup) -> Vec<(&'static str, Legend)> {
    let rows: &[GroupRow] = match g {
        LegendGroup::Plan => &PLAN,
        LegendGroup::Section => &SECTION,
        LegendGroup::Finds => &FINDS,
    };
    rows.iter()
        .map(|(name, weight, line, show, strata, favourite, _)| {
            let sample = match show {
                Show::Line => LegendSample::Line,
                Show::Area => LegendSample::Area,
                Show::Mark(m) => LegendSample::Mark((*m).to_string()),
            };
            let legend = Legend {
                weight: *weight,
                line: line.clone(),
                favourite: *favourite,
                sample,
                strata: strata.map(str::to_string),
                group: Some(g),
                hatch: Hatch::None,
            };
            (*name, legend)
        })
        .collect()
}

fn add_group(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "legend.addGroup";
    let g = group_param(p, "group", C)?.flatten().ok_or_else(|| bad(C, "`group` is plan, section or finds"))?;
    let opts: Vec<Opts> = match g {
        LegendGroup::Plan => PLAN.iter().map(|r| r.6).collect(),
        LegendGroup::Section => SECTION.iter().map(|r| r.6).collect(),
        LegendGroup::Finds => FINDS.iter().map(|r| r.6).collect(),
    };
    let d = &s.doc()?.doc;
    let there = |name: &str, key: Option<&str>| {
        d.layers.iter().any(|l| l.display_name().trim() == name || key.is_some_and(|k| l.legend().and_then(|x| x.strata.as_deref()) == Some(k)))
    };
    let rows: Vec<(&str, Legend, Opts)> =
        group_rows(g).into_iter().zip(opts).filter(|((name, row), _)| !there(name, row.strata.as_deref())).map(|((n, r), o)| (n, r, o)).collect();
    let skipped = group_rows(g).len() - rows.len();
    if rows.is_empty() {
        return Ok(json!({ "added": [], "skipped": skipped }));
    }
    let added = s.edit("Add Legend Group", |d, _| {
        // Each row goes under the ones added before it, above the drawing's layers.
        let at = d.layers.len();
        let mut ids = vec![];
        for (name, row, opts) in &rows {
            let mut n = make_layer(d, Some(name));
            let options = match opts {
                Opts::Plain => json!({}),
                Opts::Template => json!({"template": true}),
                Opts::HiddenTemplate => json!({"template": true, "visible": false}),
                Opts::NoPrint => json!({"printable": false}),
            };
            apply_options(&mut n, &options);
            if let NodeKind::Layer { legend, .. } = &mut n.kind {
                *legend = Some(Box::new(row.clone()));
            }
            ids.push(d.insert(None, at, n)?);
        }
        Ok(ids)
    })?;
    Ok(json!({ "added": added.iter().map(|i| i.0).collect::<Vec<_>>(), "skipped": skipped }))
}

fn add(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "legend.add";
    let name: Option<String> = match p.get("name") {
        None | Some(Value::Null) => None,
        Some(Value::String(n)) => Some(n.trim().chars().take(MAX_NAME).collect::<String>()).filter(|n| !n.is_empty()),
        Some(v) => return Err(bad(C, format!("`name` is text, not {v}"))),
    };
    let roots = super::edit::selected_roots(s)?;
    let st = s.doc()?;
    let d = &st.doc;
    let roots: Vec<NodeId> = roots.into_iter().filter(|r| d.is_editable(*r)).collect();
    let moving: HashSet<NodeId> = roots.iter().copied().collect();
    // The look of the first selected line (in the order selected, as the palette shows it), else
    // the pen's.
    let first = st
        .selection
        .objects
        .iter()
        .filter_map(|id| super::edit::roots_of(d, vec![*id]).first().copied())
        .filter(|r| moving.contains(r))
        .filter_map(|r| d.node(r))
        .find_map(|n| lines_in(n).first().copied());
    let (weight, line, sample) = match first.and_then(|l| legend::stroke_of(l).map(|st| (l, st))) {
        Some((l, st)) => {
            let area = match &l.kind {
                NodeKind::Path { path, .. } => path.is_closed(),
                _ => true,
            };
            (st.width, LineType::of(st), if area { LegendSample::Area } else { LegendSample::Line })
        }
        None => {
            let pen = s.new_art_stroke();
            (pen.width, LineType::of(&pen), LegendSample::Line)
        }
    };
    // As a file keeps it: a line's dashes past the legend's most become a solid row (the line set
    // apart keeps its own), so the row opens again as it was made.
    let row = Legend { weight: Some(weight), line, sample, ..Legend::default() }.tidied();
    let at = st.current_layer().and_then(|l| d.layer_of(l)).and_then(|l| d.position(l)).map_or(usize::MAX, |(_, i, _)| i + 1);
    let id = s.edit("Add to Legend", |d, _| {
        let mut n = make_layer(d, name.as_deref());
        if let NodeKind::Layer { legend, .. } = &mut n.kind {
            *legend = Some(Box::new(row));
        }
        let id = d.insert(None, at, n)?;
        for r in &roots {
            d.move_node(*r, Some(id), usize::MAX)?;
        }
        Ok(id)
    })?;
    let st = s.doc_mut()?;
    st.active_layer = Some(id);
    st.layer_rows = vec![id];
    Ok(json!({ "id": id.0, "moved": roots.len() }))
}

/// Detect whether two Bézier paths touch or overlap within a given tolerance.
fn path_adjacent(bp1: &BezPath, bp2: &BezPath, tol: f64) -> bool {
    use kurbo::{ParamCurve, ParamCurveNearest};
    let b1 = bp1.bounding_box();
    let b2 = bp2.bounding_box();
    let tol_sq = tol * tol;
    if b1.x0 - tol > b2.x1 || b1.x1 + tol < b2.x0 || b1.y0 - tol > b2.y1 || b1.y1 + tol < b2.y0 {
        return false;
    }
    let segs1: Vec<_> = bp1.segments().collect();
    let segs2: Vec<_> = bp2.segments().collect();
    for seg1 in &segs1 {
        let sb1 = seg1.bounding_box();
        for seg2 in &segs2 {
            let sb2 = seg2.bounding_box();
            if sb1.x0 - tol > sb2.x1 || sb1.x1 + tol < sb2.x0 || sb1.y0 - tol > sb2.y1 || sb1.y1 + tol < sb2.y0 {
                continue;
            }
            let p1_0 = seg1.eval(0.0);
            let p1_1 = seg1.eval(1.0);
            if seg2.nearest(p1_0, 1e-3).distance_sq <= tol_sq || seg2.nearest(p1_1, 1e-3).distance_sq <= tol_sq {
                return true;
            }
            let p2_0 = seg2.eval(0.0);
            let p2_1 = seg2.eval(1.0);
            if seg1.nearest(p2_0, 1e-3).distance_sq <= tol_sq || seg1.nearest(p2_1, 1e-3).distance_sq <= tol_sq {
                return true;
            }
        }
    }
    false
}

fn collect_closed_paths(n: &Node, d: &Document, out: &mut Vec<(NodeId, BezPath)>) {
    if !d.is_editable(n.id) {
        return;
    }
    match &n.kind {
        NodeKind::Path { path, .. } if path.subpaths.iter().any(|sp| sp.closed) => {
            out.push((n.id, path.to_bezpath()));
        }
        _ => {
            if let Some(children) = n.children() {
                for c in children {
                    collect_closed_paths(c, d, out);
                }
            }
        }
    }
}

fn cut_face(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "drawing.cutFace";
    let ids = match p.get("ids") {
        None | Some(Value::Null) => None,
        Some(_) => {
            let mut ids = checked_ids_param(s, p, "ids", C)?;
            ids.truncate(super::layer::MAX_ROWS * 10);
            Some(ids)
        }
    };
    let st = s.doc()?;
    let d = &st.doc;

    let roots: Vec<NodeId> = match ids {
        Some(ids) => ids,
        None => super::edit::selected_roots(s)?,
    };

    let mut faces: Vec<(NodeId, BezPath)> = Vec::new();
    for r in roots {
        if let Some(n) = d.node(r) {
            collect_closed_paths(n, d, &mut faces);
        }
    }

    if faces.is_empty() {
        return Err(bad(C, "select at least one closed shape to create a cut face"));
    }

    let n_faces = faces.len();
    let mut adj = vec![vec![]; n_faces];
    for i in 0..n_faces {
        for j in (i + 1)..n_faces {
            if path_adjacent(&faces[i].1, &faces[j].1, 1.0) {
                adj[i].push(j);
                adj[j].push(i);
            }
        }
    }

    let mut color: Vec<Option<usize>> = vec![None; n_faces];
    let mut conflict = false;

    for start in 0..n_faces {
        if color[start].is_some() {
            continue;
        }
        let mut queue = std::collections::VecDeque::new();
        color[start] = Some(0);
        queue.push_back(start);

        while let Some(u) = queue.pop_front() {
            let u_col = color.get(u).copied().flatten().unwrap_or(0);
            let next_col = 1 - u_col;
            if let Some(neighbors) = adj.get(u) {
                for &v in neighbors {
                    match color.get(v).copied().flatten() {
                        Some(v_col) => {
                            if v_col == u_col {
                                conflict = true;
                            }
                        }
                        None => {
                            if let Some(c) = color.get_mut(v) {
                                *c = Some(next_col);
                            }
                            queue.push_back(v);
                        }
                    }
                }
            }
        }
    }

    let warning = if conflict { Some("3개 이상의 면이 서로 맞닿아 2색 엇갈림이 불가능합니다".to_string()) } else { None };

    let mut total_lines = 0;
    let warning_ret = warning.clone();

    s.edit("Cut Face", |d, sel| {
        let mut created_ids = Vec::new();
        for (i, (face_id, face_bez)) in faces.iter().enumerate() {
            let angle = if color.get(i).copied().flatten() == Some(1) { 135.0 } else { 45.0 };
            let hatch_bp = hatch_deg(face_bez, HATCH_GAP_PT, angle);
            let lines_in_hatch = hatch_bp.elements().iter().filter(|el| matches!(el, PathEl::LineTo(_))).count();
            total_lines += lines_in_hatch;

            if let Some(node) = d.node_mut(*face_id) {
                if let Some(st) = node.appearance.stroke_mut() {
                    st.width = CUT_FACE_STROKE_WIDTH;
                    st.paint = Paint::solid(Color::BLACK);
                } else {
                    node.appearance.items.push(AppearanceItem::Stroke(StrokeLayer::new(Paint::solid(Color::BLACK), CUT_FACE_STROKE_WIDTH)));
                }
            }

            if !hatch_bp.is_empty() {
                let hatch_data = PathData::from_bezpath(&hatch_bp);
                let hatch_id = d.alloc_id();
                let mut hatch_node =
                    Node::new(hatch_id, NodeKind::Path { path: hatch_data, rule: FillRule::NonZero, live: None, clipping: false, guide: false });
                hatch_node.name = Some(format!("빗금 ({angle:.0}°)"));
                hatch_node.appearance = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), HATCH_STROKE_WIDTH);

                if let Some((par, idx, _)) = d.position(*face_id) {
                    d.insert(par, idx + 1, hatch_node)?;
                    created_ids.push(hatch_id);
                }
            }
        }
        for cid in created_ids {
            sel.add(cid);
        }
        Ok(())
    })?;

    Ok(json!({
        "faces": n_faces,
        "lines": total_lines,
        "warning": warning_ret,
    }))
}
