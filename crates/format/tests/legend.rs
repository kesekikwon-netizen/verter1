//! Trowel legend rows (J005) in `.vectorcraft` files: a layer's legend saves and opens again,
//! layers without one save as before, and files from before legends open with none.
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use proptest::prelude::*;
use serde_json::{Value, json};
use vectorcraft_doc::legend::{MAX_DASHES, MAX_KEY, MAX_MARK, MAX_WEIGHT};
use vectorcraft_doc::{Dash, Document, Hatch, LayerColor, Legend, LegendGroup, LegendSample, LineType, Node, NodeKind};
use vectorcraft_format::{VERSION, load, save};

/// The saved layer's kind object (where its layer fields are).
fn layer_json(v: &Value) -> &Value {
    &v["document"]["layers"][0]["kind"]
}

fn set_legend(d: &mut Document, legend: Legend) {
    let id = d.layers[0].id;
    if let Some(NodeKind::Layer { legend: l, .. }) = d.node_mut(id).map(|n| &mut n.kind) {
        *l = Some(Box::new(legend));
    }
}

#[test]
fn a_legend_survives_save_and_open() {
    for legend in [
        Legend {
            weight: Some(0.8),
            line: LineType::DashDot,
            favourite: true,
            sample: LegendSample::Area,
            strata: Some("feature_poly".into()),
            group: Some(LegendGroup::Plan),
            hatch: Hatch::Deg45,
        },
        Legend { weight: Some(0.35), line: LineType::Custom(vec![2.5, 1.25]), sample: LegendSample::Mark("③".into()), ..Legend::default() },
        Legend::default(),
    ] {
        let mut d = Document::new(200.0, 100.0);
        set_legend(&mut d, legend.clone());
        let back = load(&save(&d, false)).unwrap();
        assert_eq!(back.layers[0].legend(), Some(&legend));
        assert_eq!(back, d);
    }
}

#[test]
fn a_layer_without_a_legend_saves_as_before() {
    let d = Document::new(200.0, 100.0);
    let v: Value = serde_json::from_slice(&save(&d, false)).unwrap();
    assert_eq!(v["version"], json!(4), "the file format version stays 4");
    assert_eq!(VERSION, 4);
    assert_eq!(layer_json(&v)["type"], json!("layer"));
    assert!(layer_json(&v).get("legend").is_none(), "no legend key: {}", layer_json(&v));
    let mut with = d.clone();
    set_legend(&mut with, Legend { weight: Some(0.5), ..Legend::default() });
    let v: Value = serde_json::from_slice(&save(&with, false)).unwrap();
    assert_eq!(layer_json(&v)["legend"], json!({"weight": 0.5}), "{}", layer_json(&v));
}

#[test]
fn older_files_open_without_a_legend() {
    let mut v: Value = serde_json::from_slice(&save(&Document::new(10.0, 10.0), false)).unwrap();
    let layer = &mut v["document"]["layers"][0]["kind"];
    assert_eq!(layer["type"], json!("layer"));
    assert!(layer.get("legend").is_none());
    let d = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(d.layers[0].legend(), None);
}

/// A saved document with `n` top-level layers, as JSON.
fn layers_json(n: usize) -> Value {
    let mut d = Document::new(10.0, 10.0);
    for i in 1..n {
        let id = d.alloc_id();
        d.layers.push(Arc::new(Node::layer(id, &format!("layer {i}"), LayerColor::Preset(0))));
    }
    serde_json::from_slice(&save(&d, false)).unwrap()
}

/// The file with each layer's legend set to `legends` (in the layers' order), opened.
fn open_with(legends: &[Value]) -> Result<Document, vectorcraft_format::FormatError> {
    let mut v = layers_json(legends.len());
    for (i, legend) in legends.iter().enumerate() {
        v["document"]["layers"][i]["kind"]["legend"] = legend.clone();
    }
    load(&serde_json::to_vec(&v).unwrap())
}

/// A file's legends are untrusted: odd values open as the commands would take them.
#[test]
fn odd_legend_values_open_tidied() {
    let dashed = LineType::Dashed.dash_pt().unwrap();
    let d = open_with(&[
        json!({"weight": -3, "line": {"custom": [0, 0]}, "sample": {"mark": "   "}}),
        json!({"weight": 5000, "line": {"custom": vec![1.0; 17]}, "strata": "k".repeat(100)}),
        json!({"weight": 0.5, "line": {"custom": dashed}, "sample": {"mark": "x".repeat(40)}, "strata": "  "}),
    ])
    .unwrap();
    let legends: Vec<Option<&Legend>> = d.layers.iter().map(|l| l.legend()).collect();
    assert_eq!(legends[0], Some(&Legend { weight: Some(0.0), ..Legend::default() }), "no dashes drawn: solid; no mark: a line");
    assert_eq!(legends[1], Some(&Legend { weight: Some(1000.0), strata: Some("k".repeat(64)), ..Legend::default() }), "17 dashes: solid");
    assert_eq!(
        legends[2],
        Some(&Legend { weight: Some(0.5), line: LineType::Dashed, sample: LegendSample::Mark("x".repeat(32)), ..Legend::default() })
    );
}

/// A legend this version can't read (a newer line type, a value of another kind) leaves its layer
/// without a legend; the file still opens, with its other legends.
#[test]
fn a_legend_this_version_cannot_read_is_left_out() {
    let d = open_with(&[json!({"weight": 0.5, "line": "wavy"}), json!({"weight": "thick"}), json!({"weight": 0.8, "favourite": true})]).unwrap();
    let legends: Vec<Option<&Legend>> = d.layers.iter().map(|l| l.legend()).collect();
    assert_eq!(legends, vec![None, None, Some(&Legend { weight: Some(0.8), favourite: true, ..Legend::default() })]);
}

/// Older builds read layers with serde's default rule (unknown keys are skipped), which is what
/// lets them open files with legends: a key this build doesn't know opens the same way.
#[test]
fn unknown_layer_fields_are_ignored() {
    let mut v: Value = serde_json::from_slice(&save(&Document::new(10.0, 10.0), false)).unwrap();
    v["document"]["layers"][0]["kind"]["somethingNewer"] = json!({"weight": 3});
    v["document"]["layers"][0]["somethingNewer"] = json!(true);
    let d = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(d.layers.len(), 1);
    assert!(d.layers[0].is_layer());
}

/// Is `l` as the commands would take it?
fn is_tidy(l: &Legend) -> Result<(), String> {
    if l.weight.is_some_and(|w| !w.is_finite() || !(0.0..=MAX_WEIGHT).contains(&w)) {
        return Err(format!("weight {:?}", l.weight));
    }
    if let LineType::Custom(p) = &l.line
        && (p.len() > MAX_DASHES || p.iter().any(|x| !x.is_finite()) || !Dash { pattern: p.clone(), ..Dash::default() }.is_dashed())
    {
        return Err(format!("dashes {p:?}"));
    }
    if let LegendSample::Mark(m) = &l.sample
        && (m.is_empty() || m.trim() != m || m.chars().count() > MAX_MARK)
    {
        return Err(format!("mark {m:?}"));
    }
    if l.strata.as_ref().is_some_and(|k| k.is_empty() || k.trim() != k || k.chars().count() > MAX_KEY) {
        return Err(format!("strata {:?}", l.strata));
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, failure_persistence: None, ..ProptestConfig::default() })]

    /// Whatever legend a file holds, it opens (never a panic) as a tidy row that saves and opens
    /// again unchanged.
    #[test]
    fn any_legend_opens_tidy(
        weight in prop::option::of(any::<f64>()),
        dashes in prop::option::of(prop::collection::vec(prop_oneof![any::<f64>(), 0.0..20.0], 0..24)),
        mark in prop::option::of(".{0,40}"),
        strata in prop::option::of(".{0,80}"),
        favourite in any::<bool>(),
    ) {
        let legend = Legend {
            weight,
            line: dashes.map_or(LineType::Solid, LineType::Custom),
            favourite,
            sample: mark.map_or(LegendSample::Area, LegendSample::Mark),
            strata,
            group: None,
            hatch: Hatch::None,
        };
        let mut d = Document::new(10.0, 10.0);
        set_legend(&mut d, legend);
        let Ok(back) = load(&save(&d, false)) else { return Err(TestCaseError::fail("the file opens")) };
        if let Some(l) = back.layers[0].legend() {
            is_tidy(l).map_err(TestCaseError::fail)?;
            let again = load(&save(&back, false)).unwrap();
            prop_assert_eq!(again.layers[0].legend(), Some(l));
        }
    }

    /// Flipping bytes of a file with a legend never panics the loader, and what opens is tidy.
    #[test]
    fn a_damaged_file_with_a_legend_never_panics(cut in 0usize..4000, flips in prop::collection::vec((0usize..4000, any::<u8>()), 0..8)) {
        let mut d = Document::new(100.0, 100.0);
        set_legend(&mut d, Legend {
            weight: Some(0.35),
            line: LineType::Custom(vec![2.5, 1.25, 0.5, 1.25]),
            favourite: true,
            sample: LegendSample::Mark("③".into()),
            strata: Some("artifact_point".into()),
            group: Some(LegendGroup::Finds),
            hatch: Hatch::Deg135,
        });
        let mut bytes = save(&d, false);
        for (i, b) in flips {
            let n = bytes.len();
            bytes[i % n] = b;
        }
        bytes.truncate(cut.min(bytes.len()));
        if let Ok(back) = load(&bytes) {
            for l in back.layers.iter().filter_map(|l| l.legend()) {
                is_tidy(l).map_err(TestCaseError::fail)?;
            }
        }
    }
}
