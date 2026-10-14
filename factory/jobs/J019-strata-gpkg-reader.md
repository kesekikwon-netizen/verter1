# J019: Strata 조사 파일(GPKG) 읽기 ◆

## 1. 개요 및 목적
- **작업명**: J019 - Strata 조사 파일(GPKG) 읽기 ◆
- **목적**: 국가유산 및 매장유산 현장조사 표준 소프트웨어인 Strata (hgis)의 현장 조사 파일인 OGC GeoPackage(`.gpkg`, SQLite 3 데이터베이스)를 읽어와 Trowel(VectorCraft) 벡터 도면 레이어와 1:1로 결합하고 도면화하는 기능 구현.
- **성격**: 읽기 전용(Read-Only), 데스크톱 네이티브 SQLite 연동(`rusqlite bundled`), WKB(Well-Known Binary) 표준 지오메트리 파서, 좌표계(EPSG:5186/5187) 투영 변환, 유구배치도 종류별 해치 자동 부여.

---

## 2. 세부 규격 및 아키텍처

### 2.1 GeoPackage 포맷 및 WKB 지오메트리 파싱
- **SQLite 3 컨테이너 구조**:
  - `gpkg_contents`: 메타데이터 테이블 (`table_name`, `data_type`, `srs_id`, `min_x`, `min_y`, `max_x`, `max_y`).
  - `gpkg_geometry_columns`: 지오메트리 컬럼 정의 (`table_name`, `column_name`, `geometry_type_name`, `srs_id`).
  - 피처 테이블:
    - `survey_area` (조사구역)
    - `trial_trench` (시굴격자/트렌치)
    - `feature_poly` (유구 면: `kind`/`종류`, `era`/`시대`, `no`/`번호`, `uid`, `hatch`)
    - `feature_line` (유구 선: `kind`, `no`, `uid`)
    - `section_line` (단면선: `name`, `no`)
    - `control_points` (기준점: `name`, `x`, `y`, `z`)
    - `artifact_point` (유물 출토 위치: `no`, `kind`, `era`, `elevation`)
- **GeoPackage Binary Header (OGC 12-128r15)**:
  - Magic: `0x47, 0x50` ('GP')
  - Version: `0`
  - Flags: Endianness(0: BE, 1: LE), Envelope Type(0: none, 1: 32B minX/maxX/minY/maxY, 2: 48B, 3: 48B, 4: 64B)
  - SRS ID: 4 bytes int
  - Envelope Bytes (플래그에 따라 0, 32, 48, 64 바이트)
  - OGC Standard WKB (Well-Known Binary) Stream:
    - Endianness (1 = Little Endian)
    - Geometry Type (1: Point, 2: LineString, 3: Polygon, 4: MultiPoint, 5: MultiLineString, 6: MultiPolygon)
    - Coordinates: IEEE 754 f64 pairs `(X, Y)` (Easting, Northing).

### 2.2 좌표계 매핑 및 종이 좌표 변환
- 실세계 TM 좌표(m)를 도면 종이 좌표(pt)로 변환:
  $$\text{pt\_x} = (X - \text{origin\_x}) \times \text{MM} \times \frac{1000}{\text{scale}} + \text{anchor\_x}$$
  $$\text{pt\_y} = \text{anchor\_y} - (Y - \text{origin\_y}) \times \text{MM} \times \frac{1000}{\text{scale}}$$
  (종이 Y축은 아래로 증가하고, TM Northing은 북쪽/위로 증가하므로 부호 반전 적용)

### 2.3 레이어 키 짝맞춤 (Matching by Layer Key)
- 문서에 이미 존재하는 레이어 중 `legend.strata == Some(key)`인 레이어에 우선 매핑.
- 없는 경우 표준 디자인 2.1 §7 범례 그룹 규칙에 따라 새 상위 레이어 자동 생성:
  - `survey_area` -> `"조사구역 경계"` (1.0 pt, DashDot)
  - `trial_trench` -> `"시굴격자"` (0.5 pt, Solid)
  - `feature_poly` -> `"굴광선 (유구 면)"` (0.8 pt, Solid)
  - `feature_line` -> `"유구 선 (구·벽구)"` (0.5 pt, Solid)
  - `section_line` -> `"단면 위치선"` (0.2 pt, DashDot)
  - `control_points` -> `"기준점"` (△ 마크)
  - `artifact_point` -> `"유물 출토 위치"` (③ 원형 배지 마크)

### 2.4 유구배치도 종류별 빗금 (Hatch by Feature Type)
- `feature_poly` 유구 면의 `kind`(종류) 및 `hatch` 값에 따라 고고학 도면 표준 해치 자동 적용:
  - 주거지/수혈: 45° 빗금 (`Hatch::Deg45`)
  - 석축/고분/무덤: 135° 빗금 (`Hatch::Deg135`)
  - 노지/소토: 교차/번갈아 빗금 (`Hatch::Alternate`)
  - 기타/자연구: 외곽선 유지 (`Hatch::None`)

### 2.5 엔진 명령 명세
- `strata.inspectGpkg`:
  - 파라미터: `{path?: string, data?: string}`
  - 반환: `{ok: true, srs_id: u32, bounds: [min_x, min_y, max_x, max_y], tables: [{name, key, geom_type, count}]}`
- `strata.importGpkg`:
  - 파라미터: `{path?: string, data?: string, tables?: string[], apply_hatch?: bool}`
  - 반환: `{imported_features: usize, layers: string[], bounds: {...}}`

### 2.6 UI 연동
- `rowedit.rs`: `card-strata` 필드를 상호작용 가능한 선택 메뉴로 활성화하여 7개 Strata 레이어 키 할당 지원.
- `menu.rs`: 도면 메뉴의 `Send to Strata…` 항목을 연결하거나 GPKG 임포트 명령 연동.
- 10개 전체 다국어 번역 (`ko`, `ja`, `zh-hans`, `zh-hant`, `fr`, `de`, `it`, `es`, `ru`, `uk`).

---

## 3. 검증 계획
1. 단위 테스트:
   - GPKG SQLite 생성 및 WKB 바이너리 파서 무결성 테스트.
   - 좌표 투영 변환 및 폴리곤/라인/포인트 변환 검증.
   - 레이어 키 매핑 및 유구 종류별 해치 자동 부여 검증.
2. UI 통합 테스트:
   - `rowedit.rs` Strata 레이어 선택 및 `legend.strata` 반영 검증.
   - `strata.inspectGpkg` 및 `strata.importGpkg` 실행 후 도면 노드 및 속성 검증.
3. 무결함 게이트:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test -p vectorcraft-ui-egui i18n`
   - `cargo test -p vectorcraft-ui-egui trowel::`
   - `cargo xtask ci` 전체 7개 관문 통과.
