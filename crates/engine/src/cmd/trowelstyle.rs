//! Archaeological CAD Style Templates & Legend Defaults (J018, blueprint Section 7):
//! - .trowel-style JSON specification export and import with diff preview
//! - "My Defaults" (내 기본값) user preference synchronization
//! - Custom stroke weight registration for drawing palette
//! - Used-only layer filtering for publication legend tables

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use vectorcraft_doc::{Document, Hatch, Legend, LegendGroup, LineType, NodeId, NodeKind};

use super::layer::make_layer;
use super::*;

/// Trowel Style JSON specification schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrowelStyle {
    pub format: String,
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_scale")]
    pub scale: f64,
    #[serde(default = "default_weights")]
    pub weights: Vec<f64>,
    pub layers: Vec<TrowelStyleLayer>,
}

fn default_scale() -> f64 {
    20.0
}

fn default_weights() -> Vec<f64> {
    vec![0.1, 0.2, 0.3, 0.5, 0.8, 1.0]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrowelStyleLayer {
    pub name: String,
    pub weight: f64,
    #[serde(default = "default_line_type")]
    pub line_type: String,
    #[serde(default)]
    pub dash: Option<Vec<f64>>,
    #[serde(default)]
    pub hatch: String,
    #[serde(default = "default_color")]
    pub color: [u8; 3],
    #[serde(default)]
    pub favourite: bool,
    #[serde(default = "default_group")]
    pub group: String,
}

fn default_line_type() -> String {
    "solid".to_string()
}

fn default_color() -> [u8; 3] {
    [0, 0, 0]
}

fn default_group() -> String {
    "plan".to_string()
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "style.exportTrowelStyle",
            "Export Trowel Style",
            [],
            None,
            "{name?: string, description?: string} export current document layers and legend formatting as a .trowel-style JSON specification",
            has_doc,
            export_trowel_style
        ),
        cmd!(
            "style.importTrowelStyle",
            "Import Trowel Style",
            [],
            None,
            "{style: value, preview_only?: bool, apply?: bool} analyze and import .trowel-style specification into active document, comparing and syncing legend layers",
            has_doc,
            import_trowel_style
        ),
        cmd!(
            "style.setDefaults",
            "Set My Default Style",
            [],
            None,
            "{style?: value} save the active document's layer setup and legend styling as the user's permanent default template",
            has_doc,
            set_defaults
        ),
        cmd!(
            "style.getDefaults",
            "Get My Default Style",
            [],
            None,
            "{} retrieve the user's permanent default template specification",
            always,
            get_defaults
        ),
        cmd!(
            "style.applyDefaults",
            "Apply My Default Style",
            [],
            None,
            "{} apply the user's default layer template to the active document",
            has_doc,
            apply_defaults
        ),
        cmd!(
            "style.addCustomWeight",
            "Add Custom Stroke Weight",
            [],
            None,
            "{weight: number} register a custom stroke weight (points) in user preferences and drawing palette",
            always,
            add_custom_weight
        ),
    ]
}

/// Helper to set a legend on a layer node in Document.
fn set_layer_legend(d: &mut Document, id: NodeId, row: Option<Legend>) {
    if let Some(NodeKind::Layer { legend, .. }) = d.node_mut(id).map(|n| &mut n.kind) {
        *legend = row.map(Box::new);
    }
}

fn clip_weight(w: f64) -> f64 {
    (w.clamp(0.0, 1000.0) * 1000.0).round() / 1000.0
}

/// Exports current document's layers and legend styles into .trowel-style JSON format.
fn export_trowel_style(s: &mut Session, p: &Value) -> Result<Value> {
    let doc = s.active().ok_or_else(|| bad("style.exportTrowelStyle", "no active document"))?;

    let name = p.get("name").and_then(Value::as_str).unwrap_or("Trowel Archaeological Style");
    let description = p.get("description").and_then(Value::as_str).unwrap_or("Standard layer and legend specification");

    let scale = doc.doc.coord.as_ref().map_or(20.0, |c| c.scale);
    let mut custom_weights = parse_custom_weights(&s.prefs.trowel_custom_weights);
    let mut all_weights = vec![0.1, 0.2, 0.3, 0.5, 0.8, 1.0];
    for cw in custom_weights.drain(..) {
        if !all_weights.iter().any(|&w| (w - cw).abs() < 1e-4) {
            all_weights.push(cw);
        }
    }
    all_weights.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut layers = Vec::new();
    for l in &doc.doc.layers {
        let l_name = l.display_name().trim().to_string();
        if l_name.is_empty() || l_name == "표제란" || l_name == "방위표" || l_name == "축척 막대" || l_name == "범례표" {
            continue;
        }

        let (weight, line_type, dash, hatch, favourite, group) = if let Some(leg) = l.legend() {
            let lt = match &leg.line {
                LineType::Solid => "solid",
                LineType::Dashed => "dashed",
                LineType::Dotted => "dotted",
                LineType::DashDot => "chain",
                LineType::Custom(_) => "custom",
            };
            let ht = match leg.hatch {
                Hatch::None => "none",
                Hatch::Deg45 => "45",
                Hatch::Deg135 => "135",
                Hatch::Alternate => "alt",
            };
            let dashes = match &leg.line {
                LineType::Custom(pts) => Some(pts.clone()),
                _ => None,
            };
            let grp = match leg.group {
                Some(LegendGroup::Plan) => "plan",
                Some(LegendGroup::Section) => "section",
                Some(LegendGroup::Finds) => "finds",
                None => "plan",
            };
            (leg.weight.unwrap_or(0.5), lt, dashes, ht, leg.favourite, grp)
        } else {
            (0.5, "solid", None, "none", false, "plan")
        };

        layers.push(TrowelStyleLayer {
            name: l_name,
            weight,
            line_type: line_type.to_string(),
            dash,
            hatch: hatch.to_string(),
            color: [0, 0, 0],
            favourite,
            group: group.to_string(),
        });
    }

    let style = TrowelStyle {
        format: "trowel-style".to_string(),
        version: 1,
        name: name.to_string(),
        description: description.to_string(),
        scale,
        weights: all_weights,
        layers,
    };

    serde_json::to_value(style).map_err(|e| bad("style.exportTrowelStyle", format!("serialization error: {e}")))
}

/// Imports and optionally applies a .trowel-style specification into the active document.
fn import_trowel_style(s: &mut Session, p: &Value) -> Result<Value> {
    let style_val = p.get("style").ok_or_else(|| bad("style.importTrowelStyle", "missing 'style' parameter"))?;

    let style: TrowelStyle = if let Some(s_str) = style_val.as_str() {
        serde_json::from_str(s_str).map_err(|e| bad("style.importTrowelStyle", format!("invalid style JSON string: {e}")))?
    } else {
        serde_json::from_value(style_val.clone()).map_err(|e| bad("style.importTrowelStyle", format!("invalid style object: {e}")))?
    };

    if style.format != "trowel-style" {
        return Err(bad("style.importTrowelStyle", format!("unsupported style format: {}", style.format)));
    }

    let preview_only = p.get("preview_only").and_then(Value::as_bool).unwrap_or(false);
    let apply = p.get("apply").and_then(Value::as_bool).unwrap_or(true);

    let doc = s.active().ok_or_else(|| bad("style.importTrowelStyle", "no active document"))?;

    // Diff analysis against current document layers
    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut unchanged = Vec::new();

    for sl in &style.layers {
        if let Some(existing) = doc.doc.layers.iter().find(|l| l.display_name().trim() == sl.name.trim()) {
            let leg = existing.legend();
            let current_w = leg.and_then(|l| l.weight).unwrap_or(0.5);
            let current_lt = leg
                .map(|l| match &l.line {
                    LineType::Solid => "solid",
                    LineType::Dashed => "dashed",
                    LineType::Dotted => "dotted",
                    LineType::DashDot => "chain",
                    LineType::Custom(_) => "custom",
                })
                .unwrap_or("solid");

            if (current_w - sl.weight).abs() > 1e-4 || current_lt != sl.line_type {
                modified.push(json!({
                    "name": sl.name,
                    "old_weight": current_w,
                    "new_weight": sl.weight,
                    "old_line_type": current_lt,
                    "new_line_type": sl.line_type,
                }));
            } else {
                unchanged.push(json!({ "name": sl.name, "weight": sl.weight }));
            }
        } else {
            added.push(json!({
                "name": sl.name,
                "weight": sl.weight,
                "line_type": sl.line_type,
                "group": sl.group,
            }));
        }
    }

    if !preview_only && apply {
        s.edit("Import Trowel Style", |d, _| {
            for sl in &style.layers {
                let existing_id = d.layers.iter().find(|l| l.display_name().trim() == sl.name.trim()).map(|l| l.id);
                let lt = parse_line_type(&sl.line_type, sl.dash.as_deref());
                let ht = parse_hatch(&sl.hatch);
                let grp = parse_group(&sl.group);

                let leg = Legend {
                    weight: Some(clip_weight(sl.weight)),
                    line: lt,
                    hatch: ht,
                    favourite: sl.favourite,
                    group: Some(grp),
                    ..Default::default()
                };

                if let Some(eid) = existing_id {
                    set_layer_legend(d, eid, Some(leg));
                } else {
                    let layer = make_layer(d, Some(&sl.name));
                    let lid = layer.id;
                    d.insert(None, usize::MAX, layer)?;
                    set_layer_legend(d, lid, Some(leg));
                }
            }
            Ok(())
        })?;
    }

    Ok(json!({
        "style_name": style.name,
        "added": added,
        "modified": modified,
        "unchanged": unchanged,
        "applied": !preview_only && apply,
    }))
}

/// Saves active document layer setup as permanent "My Defaults".
fn set_defaults(s: &mut Session, p: &Value) -> Result<Value> {
    let exported = if let Some(style_val) = p.get("style") { style_val.clone() } else { export_trowel_style(s, p)? };
    let style_str = serde_json::to_string(&exported).map_err(|e| bad("style.setDefaults", format!("serialization error: {e}")))?;

    s.prefs.trowel_default_style = style_str;
    let count = exported.get("layers").and_then(Value::as_array).map_or(0, |a| a.len());

    Ok(json!({
        "saved": true,
        "layer_count": count,
    }))
}

/// Retrieves the user's permanent "My Defaults" style.
fn get_defaults(s: &mut Session, _p: &Value) -> Result<Value> {
    if s.prefs.trowel_default_style.trim().is_empty() || s.prefs.trowel_default_style == "{}" {
        // Built-in standard default
        Ok(json!({
            "format": "trowel-style",
            "version": 1,
            "name": "Standard Archaeological Default",
            "description": "Built-in default layers for excavation drawings",
            "scale": 20.0,
            "weights": [0.1, 0.2, 0.3, 0.5, 0.8, 1.0],
            "layers": [
                { "name": "유구 윤곽선", "weight": 0.8, "line_type": "solid", "favourite": true, "group": "plan" },
                { "name": "토층 경계선", "weight": 0.3, "line_type": "solid", "favourite": false, "group": "section" },
                { "name": "레벨선", "weight": 0.2, "line_type": "solid", "favourite": false, "group": "section" },
            ]
        }))
    } else {
        serde_json::from_str(&s.prefs.trowel_default_style).map_err(|e| bad("style.getDefaults", format!("invalid stored default style: {e}")))
    }
}

/// Applies "My Defaults" style to the active document.
fn apply_defaults(s: &mut Session, p: &Value) -> Result<Value> {
    let defs = get_defaults(s, p)?;
    let mut import_params = json!({
        "style": defs,
        "apply": true,
        "preview_only": false,
    });
    if let Some(obj) = p.as_object() {
        for (k, v) in obj {
            import_params[k] = v.clone();
        }
    }
    import_trowel_style(s, &import_params)
}

/// Registers a custom stroke weight in preferences.
fn add_custom_weight(s: &mut Session, p: &Value) -> Result<Value> {
    let w = p.get("weight").and_then(Value::as_f64).ok_or_else(|| bad("style.addCustomWeight", "missing 'weight' parameter"))?;
    if !w.is_finite() || !(0.01..=100.0).contains(&w) {
        return Err(bad("style.addCustomWeight", "stroke weight must be a finite number between 0.01 and 100 pt"));
    }
    let rounded = (w * 1000.0).round() / 1000.0;

    let mut weights = parse_custom_weights(&s.prefs.trowel_custom_weights);
    if !weights.iter().any(|&v| (v - rounded).abs() < 1e-4) {
        weights.push(rounded);
        weights.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }

    let joined = weights.iter().map(|v| format!("{v:.3}")).collect::<Vec<_>>().join(",");
    s.prefs.trowel_custom_weights = joined;

    Ok(json!({
        "weight": rounded,
        "custom_weights": weights,
    }))
}

// Helpers
pub(crate) fn parse_custom_weights(s: &str) -> Vec<f64> {
    s.split(',').filter_map(|part| part.trim().parse::<f64>().ok()).filter(|w| w.is_finite() && *w > 0.0 && *w <= 100.0).collect()
}

fn parse_line_type(s: &str, dash: Option<&[f64]>) -> LineType {
    match s.trim().to_lowercase().as_str() {
        "dashed" | "dash" => LineType::Dashed,
        "dotted" | "dot" => LineType::Dotted,
        "chain" | "dashdot" => LineType::DashDot,
        "custom" => {
            if let Some(d) = dash {
                LineType::Custom(d.to_vec())
            } else {
                LineType::Solid
            }
        }
        _ => LineType::Solid,
    }
}

fn parse_hatch(s: &str) -> Hatch {
    match s.trim().to_lowercase().as_str() {
        "45" => Hatch::Deg45,
        "135" => Hatch::Deg135,
        "alt" | "alternate" => Hatch::Alternate,
        _ => Hatch::None,
    }
}

fn parse_group(s: &str) -> LegendGroup {
    match s.trim().to_lowercase().as_str() {
        "section" => LegendGroup::Section,
        "finds" => LegendGroup::Finds,
        _ => LegendGroup::Plan,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_with_doc() -> Session {
        let mut s = Session::new();
        s.execute(
            "file.new",
            &json!({
                "width": 1190,
                "height": 842
            }),
        )
        .unwrap();
        s
    }

    #[test]
    fn test_custom_weights() {
        let mut s = session_with_doc();
        let res = s.execute("style.addCustomWeight", &json!({ "weight": 0.4 })).unwrap();
        assert_eq!(res["weight"], 0.4);
        let list = res["custom_weights"].as_array().unwrap();
        assert!(list.iter().any(|v| (v.as_f64().unwrap() - 0.4).abs() < 1e-4));
        assert_eq!(s.prefs.trowel_custom_weights, "0.400");
    }

    #[test]
    fn test_export_import_trowel_style() {
        let mut s = session_with_doc();
        let exp = s.execute("style.exportTrowelStyle", &json!({ "name": "Test Style" })).unwrap();
        assert_eq!(exp["name"], "Test Style");
        assert_eq!(exp["format"], "trowel-style");
        assert_eq!(exp["version"], 1);

        // Preview import diff
        let preview = s.execute("style.importTrowelStyle", &json!({ "style": exp, "preview_only": true })).unwrap();
        assert_eq!(preview["applied"], false);

        // Apply style
        let applied = s.execute("style.importTrowelStyle", &json!({ "style": exp, "apply": true, "preview_only": false })).unwrap();
        assert_eq!(applied["applied"], true);
    }

    #[test]
    fn test_defaults_prefs() {
        let mut s = session_with_doc();
        let exp = s.execute("style.exportTrowelStyle", &json!({ "name": "Default Test" })).unwrap();
        let set_res = s.execute("style.setDefaults", &json!({ "style": exp })).unwrap();
        assert_eq!(set_res["saved"], true);

        let get_res = s.execute("style.getDefaults", &json!({})).unwrap();
        assert_eq!(get_res["name"], "Default Test");

        let apply_res = s.execute("style.applyDefaults", &json!({})).unwrap();
        assert_eq!(apply_res["applied"], true);
    }
}
