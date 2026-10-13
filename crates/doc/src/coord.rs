//! Archaeological CAD Scale & TM Coordinate Reference System (Trowel, J006).
//! Defines scale ratios (1:10, 1:20, 1:40, 1:50, 1:100, 1:1), Korean national TM coordinates
//! (EPSG:5186 Central Origin, EPSG:5187 Eastern Origin, etc.), bidirectional point-to-TM mapping,
//! and paper millimeter conversions.

use serde::{Deserialize, Serialize};

/// One paper millimetre in points (72 pt / 25.4 mm).
pub const MM: f64 = 72.0 / 25.4;

/// Korean Archaeological CAD Scale and Coordinate Reference System settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocCoord {
    /// Scale ratio denominator (e.g. 20.0 for 1:20, 1.0 for 1:1, 100.0 for 1:100).
    #[serde(default = "default_scale")]
    pub scale: f64,
    /// National EPSG code (e.g. 5186: Central, 5187: Eastern, 5185: Western, 5188: East Sea).
    #[serde(default = "default_epsg")]
    pub epsg: u32,
    /// Easting (X) origin in meters (Korean survey convention: X = Easting).
    #[serde(default = "default_origin_x")]
    pub origin_x: f64,
    /// Northing (Y) origin in meters (Korean survey convention: Y = Northing).
    #[serde(default = "default_origin_y")]
    pub origin_y: f64,
    /// Document anchor point X on paper (points). Default 0.0.
    #[serde(default)]
    pub anchor_x: f64,
    /// Document anchor point Y on paper (points). Default 0.0.
    #[serde(default)]
    pub anchor_y: f64,
    /// Elevation datum (default "해발").
    #[serde(default = "default_datum")]
    pub elevation_datum: String,
}

fn default_scale() -> f64 {
    20.0
}

fn default_epsg() -> u32 {
    5186
}

fn default_origin_x() -> f64 {
    148_000.0
}

fn default_origin_y() -> f64 {
    98_150.0
}

fn default_datum() -> String {
    "해발".to_string()
}

impl Default for DocCoord {
    fn default() -> Self {
        Self {
            scale: default_scale(),
            epsg: default_epsg(),
            origin_x: default_origin_x(),
            origin_y: default_origin_y(),
            anchor_x: 0.0,
            anchor_y: 0.0,
            elevation_datum: default_datum(),
        }
    }
}

impl DocCoord {
    pub fn new(scale: f64, epsg: u32, origin_x: f64, origin_y: f64) -> Self {
        Self {
            scale: if scale > 0.0 { scale } else { 20.0 },
            epsg,
            origin_x,
            origin_y,
            anchor_x: 0.0,
            anchor_y: 0.0,
            elevation_datum: default_datum(),
        }
    }

    /// Formats the scale ratio as a string (e.g. "1:20", "1:1", "1:100").
    pub fn scale_label(&self) -> String {
        if (self.scale - 1.0).abs() < 1e-6 {
            "1:1".to_string()
        } else if (self.scale - self.scale.round()).abs() < 1e-6 {
            format!("1:{}", self.scale.round() as u64)
        } else {
            format!("1:{:.1}", self.scale)
        }
    }

    /// Korean description of the national TM coordinate origin.
    pub fn crs_name(epsg: u32) -> &'static str {
        match epsg {
            5186 => "5186 · 중부원점",
            5187 => "5187 · 동부원점",
            5185 => "5185 · 서부원점",
            5188 => "5188 · 동해원점",
            _ => "사용자 정의 좌표계",
        }
    }

    /// Label of the current CRS (e.g. "5186 · 중부원점").
    pub fn crs_label(&self) -> &'static str {
        Self::crs_name(self.epsg)
    }

    /// Combined summary (e.g. "1:20 · 작업 5186 · 중부원점").
    pub fn summary_label(&self) -> String {
        format!("{} · 작업 {}", self.scale_label(), self.crs_label())
    }

    /// Points per real-world meter on paper at current scale.
    /// At 1:20: 1000 mm / 20 = 50 mm = 50 * MM pt = 141.732 pt.
    pub fn points_per_meter(&self) -> f64 {
        if self.scale <= 0.0 {
            return 1.0;
        }
        (1000.0 / self.scale) * MM
    }

    /// Converts document paper point (pt_x, pt_y in points) to real-world TM coordinates (X Easting, Y Northing in meters).
    /// Notice: Paper Y increases downwards, while TM Northing Y increases upwards (North).
    pub fn pt_to_tm(&self, pt_x: f64, pt_y: f64) -> (f64, f64) {
        let ppm = self.points_per_meter();
        let dx = pt_x - self.anchor_x;
        let dy = pt_y - self.anchor_y;
        let tm_x = self.origin_x + dx / ppm;
        let tm_y = self.origin_y - dy / ppm;
        (tm_x, tm_y)
    }

    /// Converts real-world TM coordinates (tm_x Easting, tm_y Northing in meters) to document paper point (points).
    pub fn tm_to_pt(&self, tm_x: f64, tm_y: f64) -> (f64, f64) {
        let ppm = self.points_per_meter();
        let dx = (tm_x - self.origin_x) * ppm;
        let dy = (self.origin_y - tm_y) * ppm;
        (self.anchor_x + dx, self.anchor_y + dy)
    }

    /// Converts stroke weight in points to paper thickness in millimeters.
    /// E.g. 0.8 pt -> 0.282 mm (~ 0.28 mm).
    pub fn stroke_pt_to_paper_mm(weight_pt: f64) -> f64 {
        weight_pt * (25.4 / 72.0)
    }

    /// Formats a stroke conversion string (e.g. "0.8 pt 굴광선 = 종이 실측 0.28 mm").
    pub fn stroke_conversion_label(weight_pt: f64, label: &str) -> String {
        let mm = Self::stroke_pt_to_paper_mm(weight_pt);
        if label.is_empty() {
            format!("{weight_pt:.1} pt = 종이 실측 {mm:.2} mm")
        } else {
            format!("{weight_pt:.1} pt {label} = 종이 실측 {mm:.2} mm")
        }
    }
}

/// Formats a floating-point meter coordinate with thousand separators and 3 decimals.
/// E.g. 147999.305 -> "147,999.305".
pub fn format_tm_coord(val: f64) -> String {
    let sign = if val < 0.0 { "-" } else { "" };
    let abs_val = val.abs();
    let int_part = abs_val.floor() as u64;
    let frac_part = ((abs_val - abs_val.floor()) * 1000.0).round() as u64;
    let frac_str = format!("{frac_part:03}");

    let int_str = int_part.to_string();
    let mut with_commas = String::with_capacity(int_str.len() + int_str.len() / 3 + 1);
    let bytes = int_str.as_bytes();
    let len = bytes.len();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            with_commas.push(',');
        }
        with_commas.push(b as char);
    }

    format!("{sign}{with_commas}.{frac_str}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord_scale_and_crs_labels() {
        let c20 = DocCoord::new(20.0, 5186, 148_000.0, 98_150.0);
        assert_eq!(c20.scale_label(), "1:20");
        assert_eq!(c20.crs_label(), "5186 · 중부원점");
        assert_eq!(c20.summary_label(), "1:20 · 작업 5186 · 중부원점");

        let c1 = DocCoord::new(1.0, 5187, 200_000.0, 600_000.0);
        assert_eq!(c1.scale_label(), "1:1");
        assert_eq!(c1.crs_label(), "5187 · 동부원점");

        let c100 = DocCoord::new(100.0, 5186, 0.0, 0.0);
        assert_eq!(c100.scale_label(), "1:100");
    }

    #[test]
    fn test_pt_and_tm_round_trip() {
        let coord = DocCoord {
            scale: 20.0,
            epsg: 5186,
            origin_x: 147_999.305,
            origin_y: 98_157.620,
            anchor_x: 100.0,
            anchor_y: 200.0,
            elevation_datum: "해발".to_string(),
        };

        // Test anchor matches origin
        let (x0, y0) = coord.pt_to_tm(100.0, 200.0);
        assert!((x0 - 147_999.305).abs() < 1e-9);
        assert!((y0 - 98_157.620).abs() < 1e-9);

        // Test round trip at an arbitrary point
        let pt = (350.5, 620.25);
        let (tm_x, tm_y) = coord.pt_to_tm(pt.0, pt.1);
        let (back_x, back_y) = coord.tm_to_pt(tm_x, tm_y);
        assert!((back_x - pt.0).abs() < 1e-6, "back_x diff: {}", back_x - pt.0);
        assert!((back_y - pt.1).abs() < 1e-6, "back_y diff: {}", back_y - pt.1);
    }

    #[test]
    fn test_format_tm_coord() {
        assert_eq!(format_tm_coord(147_999.305), "147,999.305");
        assert_eq!(format_tm_coord(98_157.620), "98,157.620");
        assert_eq!(format_tm_coord(0.0), "0.000");
        assert_eq!(format_tm_coord(1_234_567.890), "1,234,567.890");
    }

    #[test]
    fn test_stroke_conversion() {
        let mm = DocCoord::stroke_pt_to_paper_mm(0.8);
        assert!((mm - 0.2822).abs() < 0.001);
        let s = DocCoord::stroke_conversion_label(0.8, "굴광선");
        assert_eq!(s, "0.8 pt 굴광선 = 종이 실측 0.28 mm");
    }
}
