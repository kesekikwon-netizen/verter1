# J006 — 축척과 실제 좌표 (1:10·1:20·1:40·1:100 도면, 종이 pt 굵기, 도곽 띠·상태줄 TM 좌표)

상태: done (2026-10-14) · 기준 커밋 `4dc9e525` · 검증: `cargo xtask ci` 7개 품질 게이트 통과 (fmt, clippy, test, assets, brands, layers, wasm) · **고고학 CAD 좌표계 및 축척 시스템**

## 목표

사용자 지시 및 청사진(`docs/design/trowel-design-blueprint.html` 4. 대화상자 시스템 축척 및 국가 TM 좌표계 설정)에 따라,
한국 매장유산 발굴조사 표준 CAD 도면 축척(1:10, 1:20, 1:40, 1:50, 1:100, 1:1)과 국가 TM 좌표계(EPSG:5186 중부원점, EPSG:5187 동부원점 등)의 연동 체계를 구현한다.

1. **문서 좌표 모델 (`crates/doc/src/coord.rs` 신설 및 `Document::coord` 연결)**:
   - `DocCoord` 구조체: `scale`(실측 축척 분모, f64), `epsg`(국가 EPSG 코드, u32), `origin_x`(Easting X 원점 m, f64), `origin_y`(Northing Y 원점 m, f64), `anchor_x`/`anchor_y`(도면 상 기준점 pt), `elevation_datum`(표고 기준, 기본 "해발").
   - 종이 pt와 실제 현장 TM 좌표(m) 간의 정밀 상호 변환(`pt_to_tm`, `tm_to_pt`):
     - 1 m = `(1000.0 / scale) * (72.0 / 25.4)` pt.
     - 도면 좌표계(SVG/종이: Y 아래로 증가)와 실제 TM 좌표계(Northing: Y 북쪽 위로 증가)의 부호 및 방향 정합 (Q2: X=동, Y=북).
   - 종이 굵기 환산(0.8 pt 굴광선 = 종이 실측 0.28 mm 등).

2. **엔진 명령 (`crates/engine/src/cmd/coord.rs` 신설)**:
   - `drawing.setCoord`: `{ scale?: number, epsg?: number, origin_x?: number, origin_y?: number, anchor_x?: number, anchor_y?: number, datum?: string }`.
   - `drawing.getCoord`: `{ coord: DocCoord }`.
   - 단일 Undo 지원 (`s.edit("Set Scale and Coordinate System", ...)`).
   - J004 템플릿 생성 명령(`drawing.newPlan`, `drawing.newSection`, `drawing.newFinds`, `drawing.newSheet`)에 표준 기본 좌표/축척 설정 연동.

3. **축척 및 좌표계 설정 대화상자 (`crates/ui-egui/src/trowel/coord_dialog.rs` 신설)**:
   - 청사진 4절 사양을 100% 반영:
     - 제목: `축척 및 국가 TM 좌표계 설정`
     - 작업 좌표계 경도 눈금자 (125° ~ 131°, 127° 중부, 129° 동부 강조)
     - 원점 선택: `EPSG:5186 중부원점 (추천)` / `EPSG:5187 동부원점`
     - 도면 실측 축척: `1:10` / `1:20` / `1:40` / `1:50` / `1:100` / `1:1`
     - 종이 굵기 환산 미리보기 (`0.8 pt 굴광선 = 종이 실측 0.28 mm`)
     - 점토색 주 단추: `좌표계 및 축척 적용` (Clay) / `취소`

4. **UI 실측 상태 및 표기 연동**:
   - **상태줄 (`crates/ui-egui/src/trowel/status.rs`)**:
     - 설정 시: `1:20 · 작업 5186 · 중부원점` (클릭 시 설정 대화상자 오픈)
     - 마우스 포인터 실시간 좌표: 한국 국가좌표 규격에 따라 `X 147,999.305  Y 98,157.620` (X=동, Y=북, 3자리 콤마 및 소수점 3자리 m 단위).
     - 미설정 시: `축척·좌표계 정하지 않음` 안내 및 일반 단위 표시.
   - **도면집 바닥 (`crates/ui-egui/src/trowel/drawings.rs`)**:
     - `이 도면 · 420 × 297 mm`
     - `축척 1:20`
     - `작업 5186 · 중부원점`
   - **화면 배율 칸 (`crates/ui-egui/src/trowel/center.rs`)**:
     - `화면 77 % · 도면 1:20`
   - **도면 메뉴 (`crates/ui-egui/src/trowel/menu.rs`)**:
     - `도면` ▸ `축척과 좌표계… (J006)` 활성화.
   - **도곽 띠 (Neatline collar coordinate overlay)**:
     - 캔버스 도곽 경계에 실제 TM 좌표 눈금 및 수치 띠 렌더링.

## 완료 기준

1. **`DocCoord` 데이터 구조 및 정밀 변환.**
   - `DocCoord`가 `crates/doc/src/coord.rs`에 정의되고 `Document::coord` 필드로 저장/복원된다.
   - `pt_to_tm` 및 `tm_to_pt` 왕복 변환 오차가 $10^{-6}$ m 이하이다.
   - `paper_mm_for_weight` 굵기 환산 공식이 오차 없이 작동한다.
   - 단위 테스트 `tests_coord.rs` 통과.

2. **엔진 명령 `drawing.setCoord` 및 `drawing.getCoord`.**
   - 명령 호출 시 `doc.coord`가 갱신되고 Undo 시 이전 상태로 정확히 복구된다.
   - 잘못된 인자(음수 축척, 비정상 값) 입력 시 crash 없이 Result 에러를 반환한다.
   - J004 템플릿 생성 시 자동으로 알맞은 `DocCoord`가 주입된다.

3. **축척 및 좌표계 설정 대화상자 UI.**
   - 조용한 도판(Quiet Plate) 디자인 토큰을 엄격 준수한다.
   - 경도 눈금자, EPSG 세그먼트, 축척 세그먼트, 굵기 환산 계산이 실시간 반응한다.
   - 단 하나의 Clay 버튼(`좌표계 및 축척 적용`)을 제공한다.

4. **상태줄, 도면집, 배율 칸, 메뉴 표시.**
   - 상태줄에서 `coord`가 설정되어 있을 때 실제 TM 좌표(`X 147,999.305  Y 98,157.620`) 및 `1:20 · 작업 5186 · 중부원점`이 표시된다.
   - 도면집 바닥 및 화면 배율에 `도면 1:20`이 함께 표기된다.
   - `도면` ▸ `축척과 좌표계…` 메뉴 클릭 시 대화상자가 열린다.

5. **품질 게이트.**
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
   - `cargo xtask ci` 7개 관문 모두 통과.

## 변경 범위

- 신설:
  - `crates/doc/src/coord.rs`: `DocCoord`, `CoordSystem`, TM 변환 알고리즘
  - `crates/engine/src/cmd/coord.rs`: `drawing.setCoord`, `drawing.getCoord` 명령
  - `crates/engine/src/tests_coord.rs`: 단위 테스트
  - `crates/ui-egui/src/trowel/coord_dialog.rs`: 축척 및 좌표계 설정 대화상자
- 수정:
  - `crates/doc/src/lib.rs`: `pub mod coord;`, `Document::coord`
  - `crates/engine/src/cmd/mod.rs`: `drawing.setCoord`, `drawing.getCoord` 등록
  - `crates/engine/src/cmd/legend.rs`: 템플릿 생성 시 `DocCoord` 연동
  - `crates/ui-egui/src/trowel/status.rs`: 상태줄 TM 좌표 및 축척 표시
  - `crates/ui-egui/src/trowel/drawings.rs`: 도면집 바닥 축척 및 좌표계 표기
  - `crates/ui-egui/src/trowel/center.rs`: 화면 배율 칸 축척 표기
  - `crates/ui-egui/src/trowel/menu.rs`: 축척과 좌표계 메뉴 활성화
  - `crates/ui-egui/src/trowel/mod.rs`: 모듈 등록
  - `crates/ui-egui/src/trowel/tests.rs`: UI 테스트
  - `crates/ui-egui/src/i18n/*.tsv`: 다국어 문자열 등록

## 검증 방법

```bash
cargo test -p vectorcraft-doc -- coord::
cargo test -p vectorcraft-engine -- tests_coord
cargo test -p vectorcraft-ui-egui -- trowel::
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask ci
```

## 범위 밖

- 현장 밑그림 2점/4점 GCP 정합 (J008)
- 단면선/표고점 CAD 도구 (J009)
- DXF 가져오기 좌표 보존 (J007)
- Strata GPKG 파일 읽기 (J019)

## 위험·사람 검토 필요

- 한국 국가좌표계 표기 규정: X=Easting(동향), Y=Northing(북향) 채택 (토목/고고학 공통 질의 Q2 확정안 반영).
- 축척 변경 시 기존 도면 요소는 종이 크기(pt)를 유지하고 좌표 변환 원점 기준 스케일링을 제공할지 여부: 기본값은 도면 메타데이터 갱신(기존 도형 위치 유지).
