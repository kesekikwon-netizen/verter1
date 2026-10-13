//! Legend rows (Trowel, design 2.1 §7): a top-level layer can carry a legend row, the default look
//! of the lines drawn on it (weight in points and line type) and how the row shows in the legend
//! panel.

use serde::{Deserialize, Serialize};

use crate::{Document, Node, NodeId, NodeKind, StrokeLayer};

/// One paper millimetre in points. Line type dash lengths are set in millimetres.
pub const MM: f64 = 72.0 / 25.4;
/// How close two weights or dash lengths (points) must be to count as the same: the Select Same
/// Stroke Weight tolerance (`select.same.strokeWeight`).
pub const SAME: f64 = 1e-9;
/// The heaviest weight a row takes (points), as Stroke Weight (`stroke.set`).
pub const MAX_WEIGHT: f64 = 1000.0;
/// The most dash lengths a line type takes.
pub const MAX_DASHES: usize = 16;
/// The longest mark text and Strata key (characters).
pub const MAX_MARK: usize = 32;
pub const MAX_KEY: usize = 64;

/// A legend row: the look of its layer's lines and how the row shows. Every field is optional in
/// the file; defaults are not written.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Legend {
    /// Line weight in points; `None`: the row has no look (marks, type, underlays) and leaves its
    /// lines alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(default, skip_serializing_if = "crate::skip::is_default")]
    pub line: LineType,
    /// Hatch pattern for cut faces and areas (J014).
    #[serde(default, skip_serializing_if = "crate::skip::is_default")]
    pub hatch: Hatch,
    /// ★: a row used often (pinned first in its section, clay name).
    #[serde(default, skip_serializing_if = "crate::skip::is_default")]
    pub favourite: bool,
    #[serde(default, skip_serializing_if = "crate::skip::is_default")]
    pub sample: LegendSample,
    /// The Strata layer key this row matches (`feature_poly`, …; hgis
    /// `data/schemas/ka_hgis_layers.yaml`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strata: Option<String>,
    /// The legend group the row came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<LegendGroup>,
}

/// A line type. Saved by name; dash lengths are paper millimetres from Strata's line looks (hgis
/// `src/core/FeaturePresets.cpp`: 구 3;1.5, 수혈 0.6;1.2, 도랑 3;1.2;0.6;1.2).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineType {
    #[default]
    Solid,
    Dashed,
    Dotted,
    DashDot,
    /// Any other dash pattern, in points.
    Custom(Vec<f64>),
}

/// A hatch pattern for cut faces and areas (design 2.1 §7, J014).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hatch {
    #[default]
    #[serde(rename = "none")]
    None,
    #[serde(rename = "45")]
    Deg45,
    #[serde(rename = "135")]
    Deg135,
    #[serde(rename = "alt")]
    Alternate,
}

impl Hatch {
    pub const ALL: [Hatch; 4] = [Hatch::None, Hatch::Deg45, Hatch::Deg135, Hatch::Alternate];

    pub fn key(self) -> &'static str {
        match self {
            Hatch::None => "none",
            Hatch::Deg45 => "45",
            Hatch::Deg135 => "135",
            Hatch::Alternate => "alt",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Hatch::None),
            "45" => Some(Hatch::Deg45),
            "135" => Some(Hatch::Deg135),
            "alt" => Some(Hatch::Alternate),
            _ => None,
        }
    }
}

/// The little picture in front of a row's name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LegendSample {
    #[default]
    Line,
    Area,
    /// A mark row (△, ③, 152.5m …) shows this text.
    Mark(String),
}

/// The legend groups of design 2.1 §7.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LegendGroup {
    Plan,
    Section,
    Finds,
}

impl LegendGroup {
    pub const ALL: [LegendGroup; 3] = [LegendGroup::Plan, LegendGroup::Section, LegendGroup::Finds];
    pub fn key(self) -> &'static str {
        match self {
            LegendGroup::Plan => "plan",
            LegendGroup::Section => "section",
            LegendGroup::Finds => "finds",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|g| g.key() == s)
    }
}

fn same_lengths(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= SAME)
}

impl LineType {
    /// The dash pattern in points (`None`: solid).
    pub fn dash_pt(&self) -> Option<Vec<f64>> {
        let mm: &[f64] = match self {
            LineType::Solid => return None,
            LineType::Dashed => &[3.0, 1.5],
            LineType::Dotted => &[0.6, 1.2],
            LineType::DashDot => &[3.0, 1.2, 0.6, 1.2],
            LineType::Custom(p) => return Some(p.clone()),
        };
        Some(mm.iter().map(|v| v * MM).collect())
    }

    /// What a dash pattern (points) reads as: solid when it draws no dashes, a named type when
    /// every length is within [`SAME`] of it, else the pattern itself.
    pub fn from_dash(pattern: &[f64]) -> LineType {
        let dash = crate::Dash { pattern: pattern.to_vec(), ..crate::Dash::default() };
        if !dash.is_dashed() {
            return LineType::Solid;
        }
        [LineType::Dashed, LineType::Dotted, LineType::DashDot]
            .into_iter()
            .find(|t| t.dash_pt().is_some_and(|p| same_lengths(&p, pattern)))
            .unwrap_or(LineType::Custom(dash.pattern))
    }

    /// What a stroke's dashes read as ([`Self::from_dash`]; no dashes: solid).
    pub fn of(st: &StrokeLayer) -> LineType {
        st.dash.as_ref().map_or(LineType::Solid, |d| LineType::from_dash(&d.pattern))
    }

    /// The dashes it draws: `None` when solid or when its pattern draws none.
    fn drawn(&self) -> Option<Vec<f64>> {
        self.dash_pt().filter(|p| crate::Dash { pattern: p.clone(), ..crate::Dash::default() }.is_dashed())
    }

    /// The same dashes (a custom pattern equal to a named one is that type; one that draws none is
    /// solid).
    pub fn same(&self, other: &LineType) -> bool {
        match (self.drawn(), other.drawn()) {
            (None, None) => true,
            (Some(a), Some(b)) => same_lengths(&a, &b),
            _ => false,
        }
    }
}

impl Legend {
    /// The row's look (weight and line type), when it has a weight.
    pub fn look(&self) -> Option<(f64, &LineType)> {
        self.weight.map(|w| (w, &self.line))
    }

    /// Does this stroke differ from the row's look, a line set apart ("따로")? Only weight and
    /// dashes count; a row without a look has none.
    pub fn differs(&self, st: &StrokeLayer) -> bool {
        let Some((w, line)) = self.look() else { return false };
        (st.width - w).abs() > SAME || !line.same(&LineType::of(st))
    }

    /// The row as the commands take it, for one a file gave (untrusted): a weight within
    /// 0–[`MAX_WEIGHT`] (none when not a number), dashes that draw, at most [`MAX_DASHES`] (else
    /// solid; a named type's read as that type), a mark and a Strata key trimmed and cut to
    /// [`MAX_MARK`] and [`MAX_KEY`] characters (an empty mark shows a line, an empty key is none).
    pub fn tidied(mut self) -> Legend {
        self.weight = self.weight.filter(|w| w.is_finite()).map(|w| w.clamp(0.0, MAX_WEIGHT));
        if let LineType::Custom(p) = &self.line {
            self.line = if p.len() <= MAX_DASHES { LineType::from_dash(p) } else { LineType::Solid };
        }
        if let LegendSample::Mark(m) = &self.sample {
            let m = clip(m, MAX_MARK);
            self.sample = if m.is_empty() { LegendSample::Line } else { LegendSample::Mark(m) };
        }
        self.strata = self.strata.map(|k| clip(&k, MAX_KEY)).filter(|k| !k.is_empty());
        self
    }
}

/// Text trimmed and cut to `max` characters (and trimmed again where the cut lands on a space).
pub fn clip(s: &str, max: usize) -> String {
    let cut: String = s.trim().chars().take(max).collect();
    cut.trim_end().to_string()
}

/// Read a layer's legend leniently: one this version can't read (a line type from a newer version,
/// a value of another kind) leaves the layer without a legend instead of failing the whole file;
/// one it reads is [tidied](Legend::tidied).
pub(crate) fn read<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Box<Legend>>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Read {
        Legend(Legend),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Option::<Read>::deserialize(d)? {
        Some(Read::Legend(l)) => Some(Box::new(l.tidied())),
        Some(Read::Other(_)) | None => None,
    })
}

/// The painted top stroke of a line.
pub fn stroke_of(n: &Node) -> Option<&StrokeLayer> {
    n.appearance.stroke().filter(|s| !s.paint.is_none())
}

/// Is this node a line: a path (not a guide or a clipping path) or a compound path with a painted
/// top stroke?
pub fn is_line(n: &Node) -> bool {
    matches!(n.kind, NodeKind::Path { guide: false, clipping: false, .. } | NodeKind::Compound { .. }) && stroke_of(n).is_some()
}

/// The lines of a row, top first: [`is_line`] objects in the layer, its groups and sublayers. Type,
/// images, symbols, blends and other live art are not lines and are not looked into, nor are a
/// compound's own paths.
pub fn lines(layer: &Node) -> Vec<&Node> {
    fn walk<'a>(n: &'a Node, out: &mut Vec<&'a Node>) {
        let (NodeKind::Layer { children, .. } | NodeKind::Group { children, .. }) = &n.kind else { return };
        for c in children.iter().rev() {
            if matches!(c.kind, NodeKind::Layer { .. } | NodeKind::Group { .. }) {
                walk(c, out);
            } else if is_line(c) {
                out.push(c);
            }
        }
    }
    let mut out = vec![];
    walk(layer, &mut out);
    out
}

impl Node {
    /// A top-level layer's legend row.
    pub fn legend(&self) -> Option<&Legend> {
        match &self.kind {
            NodeKind::Layer { legend, .. } => legend.as_deref(),
            _ => None,
        }
    }
}

impl Document {
    /// The legend row of the top-level layer `layer` (a sublayer is no row).
    pub fn legend(&self, layer: NodeId) -> Option<&Legend> {
        self.layers.iter().find(|l| l.id == layer)?.legend()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use vectorcraft_color::{Color, Paint};
    use vectorcraft_geom::{Affine, PathData, Point, SubPath};

    use super::*;
    use crate::{Appearance, Dash, ImageObject, LayerColor};

    const MM: f64 = 72.0 / 25.4;

    fn stroke(w: f64, dash: Option<Vec<f64>>) -> StrokeLayer {
        let mut st = StrokeLayer::new(Paint::solid(Color::BLACK), w);
        st.dash = dash.map(|pattern| Dash { pattern, ..Dash::default() });
        st
    }

    fn line(id: u64, w: f64) -> Node {
        let path = PathData::new(vec![SubPath::polyline(&[Point::new(0.0, 0.0), Point::new(10.0, 0.0)], false)]);
        Node::path(NodeId(id), path, Appearance::basic(Paint::None, Paint::solid(Color::BLACK), w))
    }

    fn with_children(mut n: Node, kids: Vec<Node>) -> Node {
        if let Some(c) = n.children_mut() {
            c.extend(kids.into_iter().map(Arc::new));
        }
        n
    }

    #[test]
    fn dash_lengths_are_stratas_paper_mm() {
        // Strata's line looks (hgis src/core/FeaturePresets.cpp:21-23): 구 3;1.5, 수혈 0.6;1.2,
        // 도랑 3;1.2;0.6;1.2 (mm).
        assert_eq!(LineType::Solid.dash_pt(), None);
        assert_eq!(LineType::Dashed.dash_pt(), Some(vec![3.0 * MM, 1.5 * MM]));
        assert_eq!(LineType::Dotted.dash_pt(), Some(vec![0.6 * MM, 1.2 * MM]));
        assert_eq!(LineType::DashDot.dash_pt(), Some(vec![3.0 * MM, 1.2 * MM, 0.6 * MM, 1.2 * MM]));
        let dashed = LineType::Dashed.dash_pt().unwrap_or_default();
        assert!((dashed[0] - 8.503_937_007_874_016).abs() < 1e-12, "3 mm is 8.504 pt: {}", dashed[0]);
        assert_eq!(LineType::Custom(vec![2.0, 1.0]).dash_pt(), Some(vec![2.0, 1.0]));
    }

    #[test]
    fn line_type_is_read_back_from_a_stroke() {
        assert_eq!(LineType::of(&stroke(0.5, None)), LineType::Solid);
        assert_eq!(LineType::of(&stroke(0.5, Some(vec![0.0, 0.0]))), LineType::Solid, "a dash that draws nothing is solid");
        for t in [LineType::Dashed, LineType::Dotted, LineType::DashDot] {
            let p = t.dash_pt().unwrap_or_default();
            assert_eq!(LineType::of(&stroke(0.5, Some(p.clone()))), t);
            let near: Vec<f64> = p.iter().map(|v| v + 5e-10).collect();
            assert_eq!(LineType::of(&stroke(0.5, Some(near))), t, "within 1e-9 pt");
            let off: Vec<f64> = p.iter().map(|v| v + 2e-9).collect();
            assert_eq!(LineType::of(&stroke(0.5, Some(off.clone()))), LineType::Custom(off), "past 1e-9 pt");
        }
        assert_eq!(LineType::of(&stroke(0.5, Some(vec![5.0, 5.0]))), LineType::Custom(vec![5.0, 5.0]));
        let dashed = LineType::Dashed.dash_pt().unwrap_or_default();
        let doubled: Vec<f64> = dashed.iter().chain(dashed.iter()).copied().collect();
        assert_eq!(LineType::of(&stroke(0.5, Some(doubled.clone()))), LineType::Custom(doubled), "the number of lengths counts");
        assert!(LineType::Custom(dashed).same(&LineType::Dashed));
        assert!(!LineType::Solid.same(&LineType::Dashed));
        assert!(LineType::Solid.same(&LineType::Solid));
    }

    #[test]
    fn a_dash_pattern_reads_as_its_line_type() {
        for t in [LineType::Dashed, LineType::Dotted, LineType::DashDot] {
            assert_eq!(LineType::from_dash(&t.dash_pt().unwrap_or_default()), t);
        }
        assert_eq!(LineType::from_dash(&[4.0, 2.0]), LineType::Custom(vec![4.0, 2.0]));
        assert_eq!(LineType::from_dash(&[0.0, 0.0]), LineType::Solid, "dashes that draw nothing");
        assert_eq!(LineType::from_dash(&[]), LineType::Solid);
        assert_eq!(LineType::from_dash(&[-1.0, 2.0]), LineType::Solid, "a pattern that doesn't dash");
    }

    #[test]
    fn lines_are_stroked_paths_and_compounds_under_groups_and_sublayers() {
        let mut fill_only = line(2, 1.0);
        fill_only.appearance = Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 1.0);
        let mut guide = line(3, 1.0);
        if let NodeKind::Path { guide: g, .. } = &mut guide.kind {
            *g = true;
        }
        let mut clipping = line(5, 1.0);
        if let NodeKind::Path { clipping: c, .. } = &mut clipping.kind {
            *c = true;
        }
        let mut clip_group = with_children(Node::group(NodeId(6), vec![]), vec![clipping, line(7, 1.0)]);
        if let NodeKind::Group { clip, .. } = &mut clip_group.kind {
            *clip = true;
        }
        let group = with_children(Node::group(NodeId(4), vec![]), vec![line(8, 1.0), clip_group]);
        let mut compound =
            with_children(Node::new(NodeId(9), NodeKind::Compound { children: vec![], rule: Default::default() }), vec![line(10, 1.0)]);
        compound.appearance = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 0.5);
        let image = Node::new(
            NodeId(11),
            NodeKind::Image(ImageObject { key: "k".into(), width: 1, height: 1, xf: Affine::IDENTITY, link: None, placement: Default::default() }),
        );
        let sub = with_children(Node::layer(NodeId(12), "sub", LayerColor::Preset(0)), vec![line(13, 1.0)]);
        let layer =
            with_children(Node::layer(NodeId(1), "row", LayerColor::Preset(0)), vec![line(14, 1.0), fill_only, guide, group, compound, image, sub]);
        let ids: Vec<u64> = lines(&layer).iter().map(|n| n.id.0).collect();
        // Top first: the last child is the topmost.
        assert_eq!(ids, vec![13, 9, 7, 8, 14]);
    }

    #[test]
    fn an_override_differs_in_weight_or_line_type() {
        let row = Legend { weight: Some(0.8), ..Legend::default() };
        assert!(!row.differs(&stroke(0.8, None)));
        assert!(!row.differs(&stroke(0.8 + 5e-10, None)), "within 1e-9 pt");
        assert!(row.differs(&stroke(0.8 + 2e-9, None)), "past 1e-9 pt");
        assert!(row.differs(&stroke(0.5, None)));
        assert!(row.differs(&stroke(0.8, LineType::Dashed.dash_pt())));
        let dashed = Legend { weight: Some(0.5), line: LineType::Dashed, ..Legend::default() };
        let mut st = stroke(0.5, LineType::Dashed.dash_pt());
        if let Some(d) = st.dash.as_mut() {
            (d.offset, d.align_corners) = (2.0, true);
        }
        st.cap = crate::LineCap::Round;
        st.paint = Paint::solid(Color::rgb(1.0, 0.0, 0.0));
        assert!(!dashed.differs(&st), "offset, corner alignment, cap and colour are not the look");
        assert!(dashed.differs(&stroke(0.5, None)));
        let no_look = Legend { weight: None, line: LineType::Dashed, ..Legend::default() };
        assert!(!no_look.differs(&stroke(3.0, None)), "a row without a weight has no overrides");
        assert_eq!(no_look.look(), None);
        assert_eq!(dashed.look(), Some((0.5, &LineType::Dashed)));
    }

    #[test]
    fn a_pattern_that_draws_nothing_is_solid() {
        // A file can hold dashes that draw nothing: they look like a solid line.
        let nothing = LineType::Custom(vec![0.0, 0.0]);
        assert!(nothing.same(&LineType::Solid));
        assert!(LineType::Solid.same(&nothing));
        assert!(!nothing.same(&LineType::Dashed));
        let row = Legend { weight: Some(0.5), line: nothing, ..Legend::default() };
        assert!(!row.differs(&stroke(0.5, None)), "a solid line follows the row");
        assert!(!row.differs(&stroke(0.5, Some(vec![0.0]))));
    }

    #[test]
    fn only_top_level_layers_are_legend_rows() {
        let row = Legend { weight: Some(0.5), ..Legend::default() };
        let with_row = |mut n: Node| {
            if let NodeKind::Layer { legend, .. } = &mut n.kind {
                *legend = Some(Box::new(row.clone()));
            }
            n
        };
        let sub = with_row(Node::layer(NodeId(3), "sub", LayerColor::Preset(1)));
        let top = with_children(with_row(Node::layer(NodeId(2), "top", LayerColor::Preset(0))), vec![sub]);
        let mut d = Document::new(100.0, 100.0);
        d.layers = vec![Arc::new(top)];
        assert_eq!(d.legend(NodeId(2)), Some(&row));
        assert_eq!(d.legend(NodeId(3)), None, "a sublayer is not a row");
    }

    #[test]
    fn odd_values_are_tidied_as_the_commands_take_them() {
        let dashed = LineType::Dashed.dash_pt().unwrap_or_default();
        let cases = [
            (
                Legend { weight: Some(-3.0), line: LineType::Custom(vec![0.0, 0.0]), sample: LegendSample::Mark("  ".into()), ..Legend::default() },
                Legend { weight: Some(0.0), ..Legend::default() },
            ),
            (
                Legend {
                    weight: Some(5000.0),
                    line: LineType::Custom(vec![1.0; 17]),
                    strata: Some(format!(" {} ", "k".repeat(100))),
                    ..Legend::default()
                },
                Legend { weight: Some(1000.0), strata: Some("k".repeat(64)), ..Legend::default() },
            ),
            (
                Legend { weight: Some(f64::NAN), line: LineType::Custom(vec![1.0, f64::INFINITY]), strata: Some("  ".into()), ..Legend::default() },
                Legend::default(),
            ),
            (
                Legend {
                    weight: Some(0.5),
                    line: LineType::Custom(dashed),
                    sample: LegendSample::Mark(format!(" {} ", "가".repeat(40))),
                    ..Legend::default()
                },
                Legend { weight: Some(0.5), line: LineType::Dashed, sample: LegendSample::Mark("가".repeat(32)), ..Legend::default() },
            ),
        ];
        for (odd, tidy) in cases {
            assert_eq!(odd.clone().tidied(), tidy, "{odd:?}");
            assert_eq!(tidy.clone().tidied(), tidy, "tidy rows stay as they are");
        }
        let fine = Legend {
            weight: Some(0.35),
            line: LineType::Custom(vec![2.5, 1.25]),
            hatch: Hatch::None,
            favourite: true,
            sample: LegendSample::Mark("③".into()),
            strata: Some("artifact_point".into()),
            group: Some(LegendGroup::Finds),
        };
        assert_eq!(fine.clone().tidied(), fine);
    }

    /// A legend this version can't read leaves its layer without one (the file still opens), and
    /// one it reads is tidied.
    #[test]
    fn layers_read_their_legend_leniently() {
        let read = |legend: serde_json::Value| -> Option<Legend> {
            let mut v = serde_json::to_value(Node::layer(NodeId(1), "row", LayerColor::Preset(0))).unwrap_or_default();
            v["kind"]["legend"] = legend;
            let n: Node = serde_json::from_value(v).unwrap_or_else(|e| panic!("the layer opens: {e}"));
            n.legend().cloned()
        };
        assert_eq!(read(serde_json::json!({"weight": 0.5, "line": "wavy"})), None, "a line type from a newer version");
        assert_eq!(read(serde_json::json!({"weight": "thick"})), None);
        assert_eq!(read(serde_json::json!([1, 2])), None);
        assert_eq!(read(serde_json::Value::Null), None);
        assert_eq!(
            read(serde_json::json!({"weight": 0.8, "group": "plan"})),
            Some(Legend { weight: Some(0.8), group: Some(LegendGroup::Plan), ..Legend::default() })
        );
        assert_eq!(
            read(serde_json::json!({"weight": -1, "line": {"custom": [0, 0]}})),
            Some(Legend { weight: Some(0.0), ..Legend::default() }),
            "tidied"
        );
    }

    #[test]
    fn legends_save_by_name() {
        let row = Legend {
            weight: Some(0.2),
            line: LineType::DashDot,
            hatch: Hatch::None,
            favourite: true,
            sample: LegendSample::Mark("③".into()),
            strata: Some("artifact_point".into()),
            group: Some(LegendGroup::Plan),
        };
        let v = serde_json::to_value(&row).unwrap_or_default();
        assert_eq!(
            v,
            serde_json::json!({"weight": 0.2, "line": "dashdot", "favourite": true, "sample": {"mark": "③"}, "strata": "artifact_point", "group": "plan"})
        );
        let back: Legend = serde_json::from_value(v).unwrap_or_default();
        assert_eq!(back, row);
        assert_eq!(serde_json::to_value(Legend::default()).unwrap_or_default(), serde_json::json!({}), "defaults are not written");
        let custom = Legend { line: LineType::Custom(vec![1.0, 2.0]), sample: LegendSample::Area, ..Legend::default() };
        assert_eq!(serde_json::to_value(&custom).unwrap_or_default(), serde_json::json!({"line": {"custom": [1.0, 2.0]}, "sample": "area"}));
        let hatched = Legend { hatch: Hatch::Deg45, ..Legend::default() };
        assert_eq!(serde_json::to_value(&hatched).unwrap_or_default(), serde_json::json!({"hatch": "45"}));
        let back_hatch: Legend = serde_json::from_value(serde_json::json!({"hatch": "135"})).unwrap_or_default();
        assert_eq!(back_hatch.hatch, Hatch::Deg135);
    }
}
