# J009 — 평면 도구 (단면선, 표고점, 유물 번호, 주혈 심도, 경사 기호, 폐합 유구 면적·둘레·장축 계산)

상태: done (2026-10-14) · 기준 커밋 `8de26a8d` · 검증: `cargo xtask ci` 7개 전체 품질 게이트 통과 (fmt, clippy, test, assets, brands, layers, wasm) · **고고학 발굴 현장 실측 평면도 전용 5대 도구 및 다각형 폐합 계측 시스템 완벽 구현**

## 목표

청사진(`docs/design/trowel-design-blueprint.html` 2절 평면 도면 화면 `p-plan`) 및 백로그 14번째 항목(J009)에 따라:
발굴조사 현장 실측 평면도(1:20 등) 제도에 필수적인 5대 전문 도구(단면선, 표고점, 유물번호, 주혈심도, 경사기호)를 구현하고, 다각형 유구 폐합 시 실세계 미터($\text{m}, \text{m}^2$) 기준 면적·둘레·장축·단축을 자동 산출하여 컨트롤 바 및 속성 패널에 연동한다.

### 1. 단면 위치선 도구 (Section Line Tool, 단축키 `Shift+A`)
- **레이어 자동 배치**: Trowel 고고학 범례의 `"단면 위치선"` 레이어 (`section_line`, 0.2 pt, 일점쇄선 `LineType::DashDot`, `stroke-dasharray="6.0 2.5 0.9 2.5"`).
- **시선 및 절단 표식**:
  - 시작점($P_1$)과 끝점($P_2$)을 잇는 기준 일점쇄선.
  - 양 끝점에서 시선 방향으로 뻗는 수직 절단 틱(Cut tick, 길이 12 pt, 1.0 pt 굵기).
  - 틱 끝에 토층 벽면을 바라보는 방향을 나타내는 시선 화살표(Sight arrow).
  - 단면 레이블 문자: 시작점에 $A$, 끝점에 $A'$ (알파벳 자동 증가: $A-A'$, $B-B'$, $C-C'$...).
  - 단축키 `Space` 또는 `Alt`로 시선 방향(좌/우) 즉각 반전(Flip).
- **엔진 명령**: `drawing.addSectionLine`
  - 인자: `{ x1: f64, y1: f64, x2: f64, y2: f64, label?: string, flip?: bool, tick_length?: f64 }`

### 2. 해발 표고점 도구 (Spot Height Tool, 단축키 `Shift+Z`)
- **레이어 자동 배치**: Trowel 고고학 범례의 `"표고점"` 레이어 (`spot_height`).
- **도호 및 수치 표기**:
  - 실측 원점: 검은색 채운 원점(Solid black dot, 반경 1.6 pt).
  - 표고 수치: 9 pt 고정폭 폰트(`m9`), 예: `152.36`, 원점 우상단 3 pt 오프셋 배치.
  - 현장 실측 높이(Z, m)를 입력하거나 활성 기준 표고(`DocCoord.elevation_datum`)와 연동.
- **엔진 명령**: `drawing.addSpotHeight`
  - 인자: `{ x: f64, y: f64, elevation: f64, label?: string }`

### 3. 유물 출토 번호 도구 (Find Number Tool)
- **레이어 자동 배치**: Trowel 고고학 범례의 `"유물 출토 위치"` 레이어 (`artifact_find`).
- **원형 배지 심볼**:
  - 배경 흰색 채움(#FFFFFF), 외곽선 0.6 pt 검은색, 반경 6 pt 원.
  - 내부에 정렬된 8 pt 아라비아 숫자 번호 (1, 2, 3... 또는 ①, ②, ③...).
  - 도구 클릭 시 번호 자동 증가(Auto-incrementing).
- **엔진 명령**: `drawing.addFindNumber`
  - 인자: `{ x: f64, y: f64, number?: u32 }`

### 4. 주혈 심도 도구 (Posthole Depth Tool)
- **레이어 자동 배치**: Trowel 고고학 범례의 `"내부 시설"` 또는 `"주혈"` 레이어.
- **주혈 원 및 심도 표기**:
  - 주혈 윤곽: 반경 $R$ (기본 13 pt), 1.25 pt 실선 원.
  - 주혈 레이블: `P1`, `P2`, `P3` 또는 굴착 깊이 병기 `P1 (-25)` (cm 단위).
  - 도구 클릭 시 번호 자동 카운트.
- **엔진 명령**: `drawing.addPosthole`
  - 인자: `{ x: f64, y: f64, radius?: f64, number?: u32, depth_cm?: i32 }`

### 5. 경사 / 법면 기호 도구 (Slope Hachure Tool)
- **레이어 자동 배치**: Trowel 고고학 범례의 `"경사 표시"` 레이어 (`slope_hachure`, 0.2 pt 실선).
- **법면 빗금 생성**:
  - 유구 경사면의 어깨선(Crest/Upper ridge)을 따라 일정 간격(예: 6 pt)으로 경사 방향 수직선 생성.
  - 장·단 빗금 교대(Alternating long and short ticks, 예: 장선 12 pt, 단선 6 pt)의 표준 고고학 법면 표기.
- **엔진 명령**: `drawing.addSlopeTicks`
  - 인자: `{ points: [[x, y], ...], tick_length?: f64, spacing?: f64, flip?: bool }`

### 6. 다각형 폐합 시 실세계 면적·둘레·장축 계측 (Closed Polygon Metrics)
- **수학 알고리즘**:
  - 신발끈 공식(Green's Shoelace Theorem)에 의한 다각형 면적 $A = \frac{1}{2} |\sum (x_i y_{i+1} - x_{i+1} y_i)|$ 계산.
  - 유클리드 선분 합에 의한 둘레 $P = \sum \|p_{i+1} - p_i\|$ 계산.
  - 다각형 정점 간 최대 유클리드 거리 장축 $L_{\text{major}} = \max_{i,j} \|p_i - p_j\|$ 및 직교 단축 $L_{\text{minor}}$ 계산.
  - 활성 축척(`DocCoord.points_per_meter()`)을 통해 도면 pt를 실세계 미터 단위($\text{m}, \text{m}^2$)로 정확히 변환.
- **UI 및 속성 연동**:
  - 컨트롤 바: `닫으면: 면적·둘레 계산` 드롭다운 활성화 (J009 해제).
  - 우측 하단 속성 창(`props.rs`):
    - 종류(Kind): 주거지, 수혈, 수구, 유구 등 드롭다운.
    - 번호(No.): 1호, 2호 등 입력.
    - 시대(Period): 청동기, 원삼국, 삼국, 조선 등 선택.
    - 계측 장부(Ledger): 면적(`32.91 ㎡`), 둘레(`24.60 m`), 장축/단축 실시간 표출.
- **엔진 명령**: `drawing.calculateMetrics`, `feature.setProps`

---

## 작업 단위 및 순서

1. **엔진 레이어 (`crates/engine/src/cmd/plantools.rs`)**:
   - `drawing.addSectionLine`, `drawing.addSpotHeight`, `drawing.addFindNumber`, `drawing.addPosthole`, `drawing.addSlopeTicks`, `drawing.calculateMetrics`, `feature.setProps` 명령 구현.
   - 단일 트랜잭션 편집(`s.edit`) 및 완벽한 Undo/Redo 보장.
   - `crates/engine/src/cmd/mod.rs`에 명령 등록.
2. **도구 레이어 (`crates/tools/src/plantools.rs` & `catalog.rs`)**:
   - 도구 등록: `section-line` (단축키 `Shift+A`), `spot-height` (단축키 `Shift+Z`), `find-number`, `posthole`, `hachure`.
   - `crates/tools/src/lib.rs`의 `create(id)`에 도구 생성자 연결.
3. **UI 레이어 (`crates/ui-egui/src/trowel/`)**:
   - `tools.rs`: archaeology 도구 목록의 `soon_widget`을 실제 도구 활성화(`app.select_tool(key)`)로 전환.
   - `ribbon.rs`: `section-line`, `spot-height` 타일을 활성 액션으로 연결.
   - `control.rs`: `When Closed`를 실제 작동 드롭다운으로 전환.
   - `props.rs`: `field_soon`을 실제 유구 속성 및 계측 장부(면적, 둘레, 장축)로 전환.
4. **i18n 다국어 카탈로그**:
   - 한국어(`ko`), 영어, 일본어, 독일어 등 11개 언어 번역 등록.
5. **품질 검증**:
   - 수학 단위 테스트: 면적, 둘레, 장축 계산 정확도 검증.
   - 도구 동작 테스트 및 UI 하네스 테스트 (81개 전원 통과 확인).
   - `cargo xtask ci` 7대 관문(포맷, 클리피, 테스트, 에셋, 브랜드, 레이어, wasm) 통과.
