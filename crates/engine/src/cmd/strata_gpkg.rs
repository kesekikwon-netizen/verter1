//! Strata Archaeological Survey GeoPackage Reader (J019, blueprint Section 9):
//! - Read-only OGC GeoPackage (.gpkg, SQLite 3 single-file database) import
//! - WKB (Well-Known Binary) geometry decoding (Points, LineStrings, Polygons, MultiPolygons)
//! - Layer key matching: survey_area, trial_trench, feature_poly, feature_line, section_line, control_points, artifact_point
//! - Real-world TM coordinates (EPSG:5186/5187) projection to document paper points
//! - Feature layout hatching by excavation feature kind (주거지/수혈: 45°, 석축/고분: 135°, 노지/소토: alternate)
//! - Attribute preservation: kind, era, no, uid, hatch

use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use serde_json::json;
#[cfg(not(target_arch = "wasm32"))]
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::Hatch;
#[cfg(not(target_arch = "wasm32"))]
use vectorcraft_doc::legend::Legend;
use vectorcraft_doc::legend::LineType;
#[cfg(not(target_arch = "wasm32"))]
use vectorcraft_doc::{Appearance, Document, Node, NodeId, NodeKind};
#[cfg(not(target_arch = "wasm32"))]
use vectorcraft_geom::{PathData, Point, Rect, SubPath, shapes};

#[cfg(not(target_arch = "wasm32"))]
use super::layer::make_layer;
use super::*;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "strata.inspectGpkg",
            "Inspect Strata GPKG File",
            [],
            None,
            "{path?: string, data?: string} inspect Strata GeoPackage survey database metadata, SRS ID, bounds and layer tables (Trowel, J019) → {ok, srs_id, bounds, tables}",
            always,
            inspect_gpkg
        ),
        cmd!(
            "strata.importGpkg",
            "Import Strata GPKG Survey Data",
            [],
            None,
            "{path?: string, data?: string, tables?: string[], apply_hatch?: bool} import Strata GeoPackage features into matching drawing layers with TM coordinate projection and feature hatching (Trowel, J019) → {imported_features, layers, bounds}",
            has_doc,
            import_gpkg
        ),
    ]
}

/// Standard Strata Layer Keys and their default Trowel styling.
pub const STRATA_LAYERS: [(&str, &str, f64, LineType); 7] = [
    ("survey_area", "조사구역 경계", 1.0, LineType::DashDot),
    ("trial_trench", "시굴격자", 0.5, LineType::Solid),
    ("feature_poly", "굴광선 (유구 면)", 0.8, LineType::Solid),
    ("feature_line", "유구 선 (구·벽구)", 0.5, LineType::Solid),
    ("section_line", "단면 위치선", 0.2, LineType::DashDot),
    ("control_points", "기준점", 0.5, LineType::Solid),
    ("artifact_point", "유물 출토 위치", 0.2, LineType::Solid),
];

/// Parsed GeoPackage geometry types.
#[derive(Clone, Debug, PartialEq)]
pub enum GpkgGeometry {
    Point(f64, f64),
    LineString(Vec<(f64, f64)>),
    Polygon(Vec<Vec<(f64, f64)>>), // outer ring, followed by optional holes
    MultiPoint(Vec<(f64, f64)>),
    MultiLineString(Vec<Vec<(f64, f64)>>),
    MultiPolygon(Vec<Vec<Vec<(f64, f64)>>>),
}

/// Typed errors for GeoPackage binary geometry and WKB parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpkgError {
    BlobTooShort,
    InvalidMagic(u8, u8),
    UnknownEnvelope(u8),
    HeaderExceedsBlob,
    UnexpectedEnd,
    UnexpectedEndU32,
    UnexpectedEndF64,
    InvalidByteOrder(u8),
    UnsupportedType(u32, u32),
    InvalidBase64(char),
}

impl std::fmt::Display for GpkgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlobTooShort => write!(f, "GPKG geometry blob too short"),
            Self::InvalidMagic(b0, b1) => write!(f, "invalid GPKG magic header: 0x{b0:02X}{b1:02X}"),
            Self::UnknownEnvelope(ind) => write!(f, "unknown GPKG envelope indicator: {ind}"),
            Self::HeaderExceedsBlob => write!(f, "GPKG geometry header exceeds blob size"),
            Self::UnexpectedEnd => write!(f, "unexpected end of WKB buffer"),
            Self::UnexpectedEndU32 => write!(f, "unexpected end of WKB buffer reading u32"),
            Self::UnexpectedEndF64 => write!(f, "unexpected end of WKB buffer reading f64"),
            Self::InvalidByteOrder(bo) => write!(f, "invalid WKB byte order flag: {bo}"),
            Self::UnsupportedType(other, raw) => write!(f, "unsupported WKB geometry type: {other} (raw: {raw})"),
            Self::InvalidBase64(c) => write!(f, "invalid base64 char: {c}"),
        }
    }
}

impl std::error::Error for GpkgError {}

pub type ParseResult<T> = std::result::Result<T, GpkgError>;

/// Decodes GeoPackage Binary Geometry (OGC 12-128r15) into a parsed GpkgGeometry.
pub fn parse_gpkg_geometry(blob: &[u8]) -> ParseResult<Option<GpkgGeometry>> {
    if blob.is_empty() {
        return Ok(None);
    }
    if blob.len() < 8 {
        return Err(GpkgError::BlobTooShort);
    }
    // 1. Magic check: 'GP' (0x47, 0x50)
    if blob[0] != 0x47 || blob[1] != 0x50 {
        return Err(GpkgError::InvalidMagic(blob[0], blob[1]));
    }
    let flags = blob[3];
    let envelope_indicator = (flags >> 1) & 0x07;
    let envelope_size = match envelope_indicator {
        0 => 0,
        1 => 32,
        2 => 48,
        3 => 48,
        4 => 64,
        _ => return Err(GpkgError::UnknownEnvelope(envelope_indicator)),
    };
    let wkb_offset = 8 + envelope_size;
    if blob.len() < wkb_offset {
        return Err(GpkgError::HeaderExceedsBlob);
    }

    parse_wkb(&blob[wkb_offset..]).map(Some)
}

/// Decodes OGC standard Well-Known Binary (WKB) buffer into GpkgGeometry.
pub fn parse_wkb(data: &[u8]) -> ParseResult<GpkgGeometry> {
    let mut cur = Cursor::new(data);
    parse_wkb_geometry(&mut cur)
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn read_u8(&mut self) -> ParseResult<u8> {
        if self.pos >= self.data.len() {
            return Err(GpkgError::UnexpectedEnd);
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    fn read_u32(&mut self, is_little_endian: bool) -> ParseResult<u32> {
        if self.remaining() < 4 {
            return Err(GpkgError::UnexpectedEndU32);
        }
        let b: [u8; 4] = [self.data[self.pos], self.data[self.pos + 1], self.data[self.pos + 2], self.data[self.pos + 3]];
        self.pos += 4;
        Ok(if is_little_endian { u32::from_le_bytes(b) } else { u32::from_be_bytes(b) })
    }

    fn read_f64(&mut self, is_little_endian: bool) -> ParseResult<f64> {
        if self.remaining() < 8 {
            return Err(GpkgError::UnexpectedEndF64);
        }
        let b: [u8; 8] = [
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ];
        self.pos += 8;
        Ok(if is_little_endian { f64::from_le_bytes(b) } else { f64::from_be_bytes(b) })
    }

    fn read_point(&mut self, is_little_endian: bool, has_z: bool, has_m: bool) -> ParseResult<(f64, f64)> {
        let x = self.read_f64(is_little_endian)?;
        let y = self.read_f64(is_little_endian)?;
        if has_z {
            let _ = self.read_f64(is_little_endian)?;
        }
        if has_m {
            let _ = self.read_f64(is_little_endian)?;
        }
        Ok((x, y))
    }
}

fn parse_wkb_geometry(cur: &mut Cursor) -> ParseResult<GpkgGeometry> {
    let byte_order = cur.read_u8()?;
    let is_little_endian = match byte_order {
        1 => true,
        0 => false,
        _ => return Err(GpkgError::InvalidByteOrder(byte_order)),
    };
    let raw_type = cur.read_u32(is_little_endian)?;

    // Handle ISO WKB types (1000 for Z, 2000 for M, 3000 for ZM) and standard 2D types
    let has_z = (1000..2000).contains(&raw_type) || raw_type >= 3000;
    let has_m = (2000..3000).contains(&raw_type) || raw_type >= 3000;
    let geom_type = raw_type % 1000;

    match geom_type {
        1 => {
            // Point
            let (x, y) = cur.read_point(is_little_endian, has_z, has_m)?;
            Ok(GpkgGeometry::Point(x, y))
        }
        2 => {
            // LineString
            let num_points = cur.read_u32(is_little_endian)? as usize;
            let mut pts = Vec::with_capacity(num_points.min(100_000));
            for _ in 0..num_points {
                pts.push(cur.read_point(is_little_endian, has_z, has_m)?);
            }
            Ok(GpkgGeometry::LineString(pts))
        }
        3 => {
            // Polygon
            let num_rings = cur.read_u32(is_little_endian)? as usize;
            let mut rings = Vec::with_capacity(num_rings.min(1_000));
            for _ in 0..num_rings {
                let num_pts = cur.read_u32(is_little_endian)? as usize;
                let mut ring = Vec::with_capacity(num_pts.min(100_000));
                for _ in 0..num_pts {
                    ring.push(cur.read_point(is_little_endian, has_z, has_m)?);
                }
                rings.push(ring);
            }
            Ok(GpkgGeometry::Polygon(rings))
        }
        4 => {
            // MultiPoint
            let num = cur.read_u32(is_little_endian)? as usize;
            let mut pts = Vec::with_capacity(num.min(100_000));
            for _ in 0..num {
                if let GpkgGeometry::Point(x, y) = parse_wkb_geometry(cur)? {
                    pts.push((x, y));
                }
            }
            Ok(GpkgGeometry::MultiPoint(pts))
        }
        5 => {
            // MultiLineString
            let num = cur.read_u32(is_little_endian)? as usize;
            let mut lines = Vec::with_capacity(num.min(10_000));
            for _ in 0..num {
                if let GpkgGeometry::LineString(pts) = parse_wkb_geometry(cur)? {
                    lines.push(pts);
                }
            }
            Ok(GpkgGeometry::MultiLineString(lines))
        }
        6 => {
            // MultiPolygon
            let num = cur.read_u32(is_little_endian)? as usize;
            let mut polys = Vec::with_capacity(num.min(10_000));
            for _ in 0..num {
                if let GpkgGeometry::Polygon(rings) = parse_wkb_geometry(cur)? {
                    polys.push(rings);
                }
            }
            Ok(GpkgGeometry::MultiPolygon(polys))
        }
        other => Err(GpkgError::UnsupportedType(other, raw_type)),
    }
}

/// Represents a single imported feature from Strata survey GPKG.
#[derive(Clone, Debug)]
pub struct StrataFeature {
    pub table: String,
    pub geom: GpkgGeometry,
    pub kind: Option<String>,
    pub era: Option<String>,
    pub no: Option<String>,
    pub uid: Option<String>,
    pub hatch: Option<String>,
}

/// Determine archaeological drawing hatch pattern from feature kind and attributes.
pub fn determine_feature_hatch(kind: Option<&str>, hatch_str: Option<&str>) -> Hatch {
    if let Some(h) = hatch_str {
        match h.trim().to_lowercase().as_str() {
            "45" => return Hatch::Deg45,
            "135" => return Hatch::Deg135,
            "alt" | "alternate" => return Hatch::Alternate,
            "none" => return Hatch::None,
            _ => {}
        }
    }
    if let Some(k) = kind {
        let k_lower = k.to_lowercase();
        if k_lower.contains("주거지") || k_lower.contains("수혈") || k_lower.contains("집터") || k_lower.contains("pit") {
            Hatch::Deg45
        } else if k_lower.contains("석축")
            || k_lower.contains("고분")
            || k_lower.contains("돌")
            || k_lower.contains("tomb")
            || k_lower.contains("wall")
        {
            Hatch::Deg135
        } else if k_lower.contains("노지") || k_lower.contains("소토") || k_lower.contains("화재") || k_lower.contains("hearth") {
            Hatch::Alternate
        } else {
            Hatch::None
        }
    } else {
        Hatch::None
    }
}

/// Inspects Strata GPKG survey file.
fn inspect_gpkg(_s: &mut Session, p: &Value) -> Result<Value> {
    let path = p.get("path").and_then(Value::as_str);
    let data_base64 = p.get("data").and_then(Value::as_str);

    #[cfg(not(target_arch = "wasm32"))]
    {
        inspect_gpkg_desktop(path, data_base64)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (path, data_base64);
        Err(bad("strata.inspectGpkg", "Strata GPKG binary inspection is supported on desktop platforms"))
    }
}

/// Imports Strata GPKG survey features into active document.
fn import_gpkg(s: &mut Session, p: &Value) -> Result<Value> {
    let path = p.get("path").and_then(Value::as_str);
    let data_base64 = p.get("data").and_then(Value::as_str);
    let apply_hatch = p.get("apply_hatch").and_then(Value::as_bool).unwrap_or(true);

    #[cfg(not(target_arch = "wasm32"))]
    {
        import_gpkg_desktop(s, path, data_base64, apply_hatch)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let _ = (s, path, data_base64, apply_hatch);
        Err(bad("strata.importGpkg", "Strata GPKG database import is supported on desktop platforms"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open_gpkg_conn(path: Option<&str>, data_base64: Option<&str>) -> Result<(rusqlite::Connection, Option<NamedTempFile>)> {
    if let Some(p) = path {
        let conn = rusqlite::Connection::open(p).map_err(|e| bad("strata.openGpkg", format!("failed to open GPKG file '{p}': {e}")))?;
        Ok((conn, None))
    } else if let Some(b64) = data_base64 {
        let bytes = decode_base64(b64).map_err(|e| bad("strata.openGpkg", format!("base64 decode error: {e}")))?;
        let mut tmp = tempfile_named()?;
        tmp.write_all(&bytes).map_err(|e| bad("strata.openGpkg", format!("tempfile write error: {e}")))?;
        let conn = rusqlite::Connection::open(&tmp.path).map_err(|e| bad("strata.openGpkg", format!("sqlite open error: {e}")))?;
        Ok((conn, Some(tmp)))
    } else {
        Err(bad("strata.openGpkg", "missing required 'path' or 'data' parameter"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct NamedTempFile {
    path: std::path::PathBuf,
}

#[cfg(not(target_arch = "wasm32"))]
impl NamedTempFile {
    fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        std::fs::write(&self.path, data)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for NamedTempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn tempfile_named() -> Result<NamedTempFile> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let tmp_path = std::env::temp_dir().join(format!("vectorcraft_strata_{nanos}.gpkg"));
    Ok(NamedTempFile { path: tmp_path })
}

#[cfg(not(target_arch = "wasm32"))]
fn inspect_gpkg_desktop(path: Option<&str>, data_base64: Option<&str>) -> Result<Value> {
    let (conn, _tmp) = open_gpkg_conn(path, data_base64)?;

    let mut stmt = conn
        .prepare("SELECT table_name, data_type, identifier, min_x, min_y, max_x, max_y, srs_id FROM gpkg_contents WHERE data_type = 'features'")
        .map_err(|e| bad("strata.inspectGpkg", format!("failed to read gpkg_contents: {e}")))?;

    let mut tables = Vec::new();
    let mut min_x: Option<f64> = None;
    let mut min_y: Option<f64> = None;
    let mut max_x: Option<f64> = None;
    let mut max_y: Option<f64> = None;
    let mut primary_srs: Option<u32> = None;

    let rows = stmt
        .query_map([], |row| {
            let table_name: String = row.get(0)?;
            let data_type: String = row.get(1)?;
            let identifier: Option<String> = row.get(2)?;
            let t_min_x: Option<f64> = row.get(3)?;
            let t_min_y: Option<f64> = row.get(4)?;
            let t_max_x: Option<f64> = row.get(5)?;
            let t_max_y: Option<f64> = row.get(6)?;
            let srs_id: Option<u32> = row.get(7)?;
            Ok((table_name, data_type, identifier, t_min_x, t_min_y, t_max_x, t_max_y, srs_id))
        })
        .map_err(|e| bad("strata.inspectGpkg", format!("query error: {e}")))?;

    for r in rows {
        let (table_name, _data_type, identifier, t_min_x, t_min_y, t_max_x, t_max_y, srs_id) =
            r.map_err(|e| bad("strata.inspectGpkg", format!("row error: {e}")))?;

        if let Some(srs) = srs_id
            && primary_srs.is_none()
        {
            primary_srs = Some(srs);
        }

        if let (Some(x0), Some(y0), Some(x1), Some(y1)) = (t_min_x, t_min_y, t_max_x, t_max_y) {
            min_x = Some(min_x.map_or(x0, |v| v.min(x0)));
            min_y = Some(min_y.map_or(y0, |v| v.min(y0)));
            max_x = Some(max_x.map_or(x1, |v| v.max(x1)));
            max_y = Some(max_y.map_or(y1, |v| v.max(y1)));
        }

        // Count features in table safely
        let count_query = format!("SELECT COUNT(*) FROM \"{table_name}\"");
        let count: i64 = conn.query_row(&count_query, [], |r| r.get(0)).unwrap_or(0);

        // Find standard strata key match
        let strata_key = STRATA_LAYERS.iter().find(|(k, _, _, _)| *k == table_name).map(|(k, _, _, _)| *k);

        tables.push(json!({
            "table_name": table_name,
            "strata_key": strata_key,
            "identifier": identifier.unwrap_or_else(|| table_name.clone()),
            "count": count,
            "srs_id": srs_id,
        }));
    }

    Ok(json!({
        "ok": true,
        "srs_id": primary_srs.unwrap_or(5186),
        "bounds": {
            "min_x": min_x,
            "min_y": min_y,
            "max_x": max_x,
            "max_y": max_y,
        },
        "tables": tables,
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn import_gpkg_desktop(s: &mut Session, path: Option<&str>, data_base64: Option<&str>, apply_hatch: bool) -> Result<Value> {
    let (conn, _tmp) = open_gpkg_conn(path, data_base64)?;

    // 1. Discover feature tables
    let mut stmt = conn
        .prepare("SELECT table_name FROM gpkg_contents WHERE data_type = 'features'")
        .map_err(|e| bad("strata.importGpkg", format!("failed to read gpkg_contents: {e}")))?;

    let table_names: Vec<String> =
        stmt.query_map([], |r| r.get(0)).map_err(|e| bad("strata.importGpkg", format!("query error: {e}")))?.filter_map(|r| r.ok()).collect();

    // 2. Read features from each table
    let mut all_features = Vec::new();
    let mut global_min_x: Option<f64> = None;
    let mut global_min_y: Option<f64> = None;
    let mut global_max_x: Option<f64> = None;
    let mut global_max_y: Option<f64> = None;

    for tbl in &table_names {
        // Find geometry column name
        let geom_col_query = format!("SELECT column_name FROM gpkg_geometry_columns WHERE table_name = '{tbl}'");
        let geom_col: String = conn.query_row(&geom_col_query, [], |r| r.get(0)).unwrap_or_else(|_| "geom".to_string());

        // Check available attribute columns
        let cols_query = format!("PRAGMA table_info(\"{tbl}\")");
        let mut pragma_stmt = conn.prepare(&cols_query).map_err(|e| bad("strata.importGpkg", format!("pragma error: {e}")))?;
        let col_names: Vec<String> = pragma_stmt
            .query_map([], |r| r.get(1))
            .map_err(|e| bad("strata.importGpkg", format!("col map error: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        let has_kind = col_names.iter().any(|c| c == "kind" || c == "종류");
        let has_era = col_names.iter().any(|c| c == "era" || c == "시대");
        let has_no = col_names.iter().any(|c| c == "no" || c == "번호" || c == "feature_no");
        let has_uid = col_names.iter().any(|c| c == "uid");
        let has_hatch = col_names.iter().any(|c| c == "hatch");

        let kind_col = if col_names.contains(&"kind".to_string()) { "kind" } else { "종류" };
        let era_col = if col_names.contains(&"era".to_string()) { "era" } else { "시대" };
        let no_col = if col_names.contains(&"no".to_string()) { "no" } else { "번호" };

        let query = format!(
            "SELECT \"{geom_col}\"{}{}{}{}{} FROM \"{tbl}\"",
            if has_kind { format!(", \"{kind_col}\"") } else { String::new() },
            if has_era { format!(", \"{era_col}\"") } else { String::new() },
            if has_no { format!(", \"{no_col}\"") } else { String::new() },
            if has_uid { ", \"uid\"" } else { "" },
            if has_hatch { ", \"hatch\"" } else { "" },
        );

        let mut feature_stmt = match conn.prepare(&query) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let mut col_idx = 1;
        let k_idx = if has_kind {
            let i = col_idx;
            col_idx += 1;
            Some(i)
        } else {
            None
        };
        let e_idx = if has_era {
            let i = col_idx;
            col_idx += 1;
            Some(i)
        } else {
            None
        };
        let n_idx = if has_no {
            let i = col_idx;
            col_idx += 1;
            Some(i)
        } else {
            None
        };
        let u_idx = if has_uid {
            let i = col_idx;
            col_idx += 1;
            Some(i)
        } else {
            None
        };
        let h_idx = if has_hatch {
            let i = col_idx;
            Some(i)
        } else {
            None
        };

        let mut rows = match feature_stmt.query([]) {
            Ok(r) => r,
            Err(_) => continue,
        };

        while let Ok(Some(row)) = rows.next() {
            let blob_res: rusqlite::Result<Vec<u8>> = row.get(0);
            let blob = match blob_res {
                Ok(b) => b,
                Err(_) => continue,
            };

            let geom = match parse_gpkg_geometry(&blob) {
                Ok(Some(g)) => g,
                _ => continue,
            };

            // Update bounds
            update_geom_bounds(&geom, &mut global_min_x, &mut global_min_y, &mut global_max_x, &mut global_max_y);

            let kind = k_idx.and_then(|i| row.get::<_, Option<String>>(i).ok().flatten());
            let era = e_idx.and_then(|i| row.get::<_, Option<String>>(i).ok().flatten());
            let no = n_idx.and_then(|i| row.get::<_, Option<String>>(i).ok().flatten());
            let uid = u_idx.and_then(|i| row.get::<_, Option<String>>(i).ok().flatten());
            let hatch = h_idx.and_then(|i| row.get::<_, Option<String>>(i).ok().flatten());

            all_features.push(StrataFeature { table: tbl.clone(), geom, kind, era, no, uid, hatch });
        }
    }

    if all_features.is_empty() {
        return Ok(json!({
            "imported_features": 0,
            "layers": [],
            "bounds": null,
        }));
    }

    // 3. Document insertion & Layer mapping
    let count = all_features.len();
    let mut imported_layers = Vec::new();

    s.edit("Import Strata GPKG Survey Data", |d, _| {
        // Real-world to paper coordinate projection
        let scale = d.coord.as_ref().map_or(20.0, |c| c.scale);
        let origin_x = d.coord.as_ref().map_or_else(|| global_min_x.unwrap_or(148_000.0), |c| c.origin_x);
        let origin_y = d.coord.as_ref().map_or_else(|| global_min_y.unwrap_or(98_150.0), |c| c.origin_y);
        let anchor_x = d.coord.as_ref().map_or(100.0, |c| c.anchor_x);
        let anchor_y = d.coord.as_ref().map_or(600.0, |c| c.anchor_y);
        let mm = 72.0 / 25.4;

        let to_paper = |x: f64, y: f64| -> Point {
            let px = (x - origin_x) * mm * 1000.0 / scale + anchor_x;
            let py = anchor_y - (y - origin_y) * mm * 1000.0 / scale;
            Point::new(px, py)
        };

        for feat in all_features {
            let target_layer_id = get_or_create_strata_layer(d, &feat.table)?;
            let layer_name = d.node(target_layer_id).map(|n| n.display_name()).unwrap_or_else(|| feat.table.clone());
            if !imported_layers.contains(&layer_name) {
                imported_layers.push(layer_name);
            }

            let hatch = if apply_hatch { determine_feature_hatch(feat.kind.as_deref(), feat.hatch.as_deref()) } else { Hatch::None };

            let stroke_width = if feat.table == "survey_area" {
                1.0
            } else if feat.table == "feature_poly" {
                0.8
            } else if feat.table == "section_line" {
                0.2
            } else {
                0.5
            };

            let app = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), stroke_width);

            // Convert geometry into nodes
            match feat.geom {
                GpkgGeometry::Point(x, y) => {
                    let pt = to_paper(x, y);
                    let circle = shapes::ellipse(Rect::new(pt.x - 3.5, pt.y - 3.5, pt.x + 3.5, pt.y + 3.5));
                    let pid = d.alloc_id();
                    let point_app = Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 0.0);
                    d.insert(Some(target_layer_id), usize::MAX, Node::path(pid, circle, point_app))?;
                }
                GpkgGeometry::MultiPoint(pts) => {
                    for (x, y) in pts {
                        let pt = to_paper(x, y);
                        let circle = shapes::ellipse(Rect::new(pt.x - 3.5, pt.y - 3.5, pt.x + 3.5, pt.y + 3.5));
                        let pid = d.alloc_id();
                        let point_app = Appearance::basic(Paint::solid(Color::BLACK), Paint::None, 0.0);
                        d.insert(Some(target_layer_id), usize::MAX, Node::path(pid, circle, point_app))?;
                    }
                }
                GpkgGeometry::LineString(pts) => {
                    if pts.len() >= 2 {
                        let paper_pts: Vec<Point> = pts.iter().map(|&(x, y)| to_paper(x, y)).collect();
                        let pd = PathData::new(vec![SubPath::polyline(&paper_pts, false)]);
                        let pid = d.alloc_id();
                        d.insert(Some(target_layer_id), usize::MAX, Node::path(pid, pd, app.clone()))?;
                    }
                }
                GpkgGeometry::MultiLineString(lines) => {
                    for pts in lines {
                        if pts.len() >= 2 {
                            let paper_pts: Vec<Point> = pts.iter().map(|&(x, y)| to_paper(x, y)).collect();
                            let pd = PathData::new(vec![SubPath::polyline(&paper_pts, false)]);
                            let pid = d.alloc_id();
                            d.insert(Some(target_layer_id), usize::MAX, Node::path(pid, pd, app.clone()))?;
                        }
                    }
                }
                GpkgGeometry::Polygon(rings) => {
                    if let Some(outer) = rings.first()
                        && outer.len() >= 3
                    {
                        let mut subpaths = Vec::new();
                        for ring in &rings {
                            let paper_pts: Vec<Point> = ring.iter().map(|&(x, y)| to_paper(x, y)).collect();
                            subpaths.push(SubPath::polyline(&paper_pts, true));
                        }
                        let pd = PathData::new(subpaths);
                        let pid = d.alloc_id();
                        let poly_node = Node::path(pid, pd, app.clone());
                        // Apply archaeological hatch if indicated
                        if hatch != Hatch::None {
                            // Cut face / feature hatch metadata
                            if let Some(mut leg) = d.node(target_layer_id).and_then(|n| n.legend()).cloned() {
                                leg.hatch = hatch;
                                set_layer_legend(d, target_layer_id, Some(leg));
                            }
                        }
                        d.insert(Some(target_layer_id), usize::MAX, poly_node)?;
                    }
                }
                GpkgGeometry::MultiPolygon(polys) => {
                    for rings in polys {
                        if let Some(outer) = rings.first()
                            && outer.len() >= 3
                        {
                            let mut subpaths = Vec::new();
                            for ring in &rings {
                                let paper_pts: Vec<Point> = ring.iter().map(|&(x, y)| to_paper(x, y)).collect();
                                subpaths.push(SubPath::polyline(&paper_pts, true));
                            }
                            let pd = PathData::new(subpaths);
                            let pid = d.alloc_id();
                            d.insert(Some(target_layer_id), usize::MAX, Node::path(pid, pd, app.clone()))?;
                        }
                    }
                }
            }
        }
        Ok(())
    })?;

    Ok(json!({
        "imported_features": count,
        "layers": imported_layers,
        "bounds": {
            "min_x": global_min_x,
            "min_y": global_min_y,
            "max_x": global_max_x,
            "max_y": global_max_y,
        }
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn update_geom_bounds(geom: &GpkgGeometry, min_x: &mut Option<f64>, min_y: &mut Option<f64>, max_x: &mut Option<f64>, max_y: &mut Option<f64>) {
    let mut check_pt = |x: f64, y: f64| {
        *min_x = Some(min_x.map_or(x, |v| v.min(x)));
        *min_y = Some(min_y.map_or(y, |v| v.min(y)));
        *max_x = Some(max_x.map_or(x, |v| v.max(x)));
        *max_y = Some(max_y.map_or(y, |v| v.max(y)));
    };
    match geom {
        GpkgGeometry::Point(x, y) => check_pt(*x, *y),
        GpkgGeometry::MultiPoint(pts) => {
            for (x, y) in pts {
                check_pt(*x, *y);
            }
        }
        GpkgGeometry::LineString(pts) => {
            for (x, y) in pts {
                check_pt(*x, *y);
            }
        }
        GpkgGeometry::MultiLineString(lines) => {
            for pts in lines {
                for (x, y) in pts {
                    check_pt(*x, *y);
                }
            }
        }
        GpkgGeometry::Polygon(rings) => {
            for ring in rings {
                for (x, y) in ring {
                    check_pt(*x, *y);
                }
            }
        }
        GpkgGeometry::MultiPolygon(polys) => {
            for rings in polys {
                for ring in rings {
                    for (x, y) in ring {
                        check_pt(*x, *y);
                    }
                }
            }
        }
    }
}

/// Helper to get existing layer matching strata key or create standard layer.
#[cfg(not(target_arch = "wasm32"))]
fn get_or_create_strata_layer(d: &mut Document, strata_key: &str) -> Result<NodeId> {
    // 1. Check if an existing top-level layer matches legend.strata
    if let Some(l) = d.layers.iter().find(|l| l.legend().and_then(|leg| leg.strata.as_deref()) == Some(strata_key)) {
        return Ok(l.id);
    }

    // 2. Check standard strata layer mapping
    let standard = STRATA_LAYERS.iter().find(|(k, _, _, _)| *k == strata_key);
    let (target_name, weight, line_type) =
        if let Some((_, name, w, lt)) = standard { (*name, *w, lt.clone()) } else { (strata_key, 0.5, LineType::Solid) };

    // Check by name
    if let Some(l) = d.layers.iter().find(|l| l.display_name().trim() == target_name) {
        return Ok(l.id);
    }

    // 3. Create new layer
    let layer = make_layer(d, Some(target_name));
    let lid = layer.id;
    let leg = Legend { weight: Some(weight), line: line_type, strata: Some(strata_key.to_string()), ..Default::default() };
    d.insert(None, usize::MAX, layer)?;
    set_layer_legend(d, lid, Some(leg));
    Ok(lid)
}

#[cfg(not(target_arch = "wasm32"))]
fn set_layer_legend(d: &mut Document, id: NodeId, row: Option<Legend>) {
    if let Some(NodeKind::Layer { legend, .. }) = d.node_mut(id).map(|n| &mut n.kind) {
        *legend = row.map(Box::new);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_base64(s: &str) -> ParseResult<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buf = Vec::new();
    let mut val = 0u32;
    let mut bits = 0;

    for &b in s.as_bytes() {
        if b == b'=' || b.is_ascii_whitespace() {
            continue;
        }
        let v = match TABLE.iter().position(|&c| c == b) {
            Some(idx) => idx as u32,
            None => return Err(GpkgError::InvalidBase64(b as char)),
        };
        val = (val << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buf.push((val >> bits) as u8);
            val &= (1 << bits) - 1;
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wkb_point() {
        // LE Point: (148000.5, 98150.25)
        let mut bytes = vec![1, 1, 0, 0, 0]; // LE, Type 1
        bytes.extend_from_slice(&148000.5f64.to_le_bytes());
        bytes.extend_from_slice(&98150.25f64.to_le_bytes());

        let geom = parse_wkb(&bytes).unwrap();
        assert_eq!(geom, GpkgGeometry::Point(148000.5, 98150.25));
    }

    #[test]
    fn test_wkb_linestring() {
        // LE LineString: 2 points
        let mut bytes = vec![1, 2, 0, 0, 0]; // LE, Type 2
        bytes.extend_from_slice(&2u32.to_le_bytes()); // 2 points
        bytes.extend_from_slice(&10.0f64.to_le_bytes());
        bytes.extend_from_slice(&20.0f64.to_le_bytes());
        bytes.extend_from_slice(&30.0f64.to_le_bytes());
        bytes.extend_from_slice(&40.0f64.to_le_bytes());

        let geom = parse_wkb(&bytes).unwrap();
        assert_eq!(geom, GpkgGeometry::LineString(vec![(10.0, 20.0), (30.0, 40.0)]));
    }

    #[test]
    fn test_wkb_polygon() {
        // LE Polygon: 1 ring, 4 points
        let mut bytes = vec![1, 3, 0, 0, 0]; // LE, Type 3
        bytes.extend_from_slice(&1u32.to_le_bytes()); // 1 ring
        bytes.extend_from_slice(&4u32.to_le_bytes()); // 4 points
        for &(x, y) in &[(0.0f64, 0.0f64), (10.0f64, 0.0f64), (10.0f64, 10.0f64), (0.0f64, 0.0f64)] {
            bytes.extend_from_slice(&x.to_le_bytes());
            bytes.extend_from_slice(&y.to_le_bytes());
        }

        let geom = parse_wkb(&bytes).unwrap();
        assert_eq!(geom, GpkgGeometry::Polygon(vec![vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 0.0)]]));
    }

    #[test]
    fn test_gpkg_binary_header() {
        // GPKG header + Point
        let mut blob = vec![0x47, 0x50, 0x00, 0x01]; // 'GP', ver 0, flags: LE (bit 0 = 1), no envelope (bits 1-3 = 0)
        blob.extend_from_slice(&5186i32.to_le_bytes()); // SRS ID
        // Append WKB Point
        blob.push(1); // LE
        blob.extend_from_slice(&1u32.to_le_bytes());
        blob.extend_from_slice(&100.0f64.to_le_bytes());
        blob.extend_from_slice(&200.0f64.to_le_bytes());

        let geom = parse_gpkg_geometry(&blob).unwrap().unwrap();
        assert_eq!(geom, GpkgGeometry::Point(100.0, 200.0));
    }

    #[test]
    fn test_feature_hatch_determination() {
        assert_eq!(determine_feature_hatch(Some("주거지 1호"), None), Hatch::Deg45);
        assert_eq!(determine_feature_hatch(Some("석축유구"), None), Hatch::Deg135);
        assert_eq!(determine_feature_hatch(Some("노지 화재흔"), None), Hatch::Alternate);
        assert_eq!(determine_feature_hatch(Some("자연구"), None), Hatch::None);
        assert_eq!(determine_feature_hatch(None, Some("135")), Hatch::Deg135);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_inspect_and_import_gpkg_database() {
        let tmp = tempfile_named().unwrap();
        let conn = rusqlite::Connection::open(&tmp.path).unwrap();
        conn.execute("CREATE TABLE gpkg_contents (table_name TEXT, data_type TEXT, identifier TEXT, description TEXT, last_change TEXT, min_x REAL, min_y REAL, max_x REAL, max_y REAL, srs_id INTEGER)", []).unwrap();
        conn.execute(
            "CREATE TABLE gpkg_geometry_columns (table_name TEXT, column_name TEXT, geometry_type_name TEXT, srs_id INTEGER, z TINYINT, m TINYINT)",
            [],
        )
        .unwrap();

        conn.execute("INSERT INTO gpkg_contents VALUES ('feature_poly', 'features', '유구 면', 'Pit outline', '2026-10-14', 148000.0, 98150.0, 148050.0, 98200.0, 5186)", []).unwrap();
        conn.execute("INSERT INTO gpkg_geometry_columns VALUES ('feature_poly', 'geom', 'POLYGON', 5186, 0, 0)", []).unwrap();

        conn.execute("CREATE TABLE feature_poly (id INTEGER PRIMARY KEY, geom BLOB, kind TEXT, era TEXT, no TEXT, uid TEXT, hatch TEXT)", [])
            .unwrap();

        // Build GPKG binary geometry blob for polygon
        let mut blob = vec![0x47, 0x50, 0x00, 0x01]; // 'GP', ver 0, flags: LE (bit 0 = 1), no envelope
        blob.extend_from_slice(&5186i32.to_le_bytes()); // SRS ID
        blob.push(1); // LE
        blob.extend_from_slice(&3u32.to_le_bytes()); // Polygon
        blob.extend_from_slice(&1u32.to_le_bytes()); // 1 ring
        blob.extend_from_slice(&4u32.to_le_bytes()); // 4 points
        for &(x, y) in &[(148000.0f64, 98150.0f64), (148050.0f64, 98150.0f64), (148050.0f64, 98200.0f64), (148000.0f64, 98150.0f64)] {
            blob.extend_from_slice(&x.to_le_bytes());
            blob.extend_from_slice(&y.to_le_bytes());
        }

        conn.execute("INSERT INTO feature_poly VALUES (1, ?1, '주거지', '청동기', '1호', 'uid-001', '45')", [&blob]).unwrap();
        drop(conn);

        let mut s = Session::new();
        s.execute("file.new", &json!({ "width": 1190, "height": 842 })).unwrap();

        let insp = s.execute("strata.inspectGpkg", &json!({ "path": tmp.path.to_str().unwrap() })).unwrap();
        assert_eq!(insp["ok"], true);
        assert_eq!(insp["srs_id"], 5186);
        let tables = insp["tables"].as_array().unwrap();
        assert!(tables.iter().any(|t| t["table_name"] == "feature_poly"));

        let imp = s.execute("strata.importGpkg", &json!({ "path": tmp.path.to_str().unwrap(), "apply_hatch": true })).unwrap();
        assert_eq!(imp["imported_features"], 1);
        let layers = imp["layers"].as_array().unwrap();
        assert!(layers.iter().any(|l| l.as_str().unwrap().contains("유구")));
    }
}
