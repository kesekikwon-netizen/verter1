# J012: 종합 도판과 PDF (Plate & Sheet Layout View)

## 1. 개요 및 배경
국가유산 발굴조사 보고서 및 학술조사 보고서 발간 시, 제도된 평면도·단면도·유물 실측도를 A3/A4 정규 판형에 조판하고 학술 보고서 표준 4대 요소(도북 방위, 복합 축척 막대, 국가유산 표준 표제란, 자동 추출 범례표)를 배치하는 **종합 도판 조판 및 PDF 출판 기능**은 발굴조사 캐드의 최종 완성 단계이다.
본 작업(J012)은 디자이너나 조사원이 일러스트레이터에서 일일이 수작업으로 표제란과 축척자를 그리던 번거로움을 완전히 없애고, 버튼 클릭 한 번으로 발굴 표준 규격을 완벽하게 충족하는 도판 요소를 자동 생성하며 출판용 고해상도 PDF 및 CAD 데이터로 내보낼 수 있는 체계를 완성한다.

---

## 2. 세부 기능 요구사항

### 2.1 도판 규격 및 테두리 (Sheet Paper & Margin Border)
1. **판형 규격**:
   - A3 가로 ($420 \times 297\text{ mm}$): $1190.55 \times 841.89\text{ pt}$.
   - A4 가로 ($297 \times 210\text{ mm}$): $841.89 \times 595.28\text{ pt}$.
   - A4 세로 ($210 \times 297\text{ mm}$): $595.28 \times 841.89\text{ pt}$.
2. **외곽 도곽 2중선 (Double Border Frame)**:
   - 외곽선: 1.0 pt `#141413`.
   - 간격: 3 mm ($8.5\text{ pt}$).
   - 내곽선: 0.3 pt `#141413`.
   - 여백: 15 mm ~ 20 mm.

### 2.2 도북 방위 (North Arrow)
1. **규격 및 형상**:
   - 국가유산 표준 침상 화살표 (28 mm, $79.4\text{ pt}$).
   - 상단 `N` 문자 (11 pt Bold, Noto Sans).
   - 좌반면 검은색 솔리드 채움, 우반면 백색 채움의 정통 제도 도북.
   - 각도(회전각): 북쪽 기준 0°(기본값) 또는 회전각 매개변수 지원.
2. **엔진 명령**: `sheet.addNorthArrow`
   - 매개변수: `{x?: number, y?: number, rotation?: number, size_pt?: number}`

### 2.3 복합 축척자 (Composite Graphic Scale Bar)
1. **규격 및 형상**:
   - 흑백 교번 분할 블록 (Alternating black/white blocks).
   - 0 기준 좌측 정밀 세분(0.5m/1m), 우측 2m, 5m, 10m 주 눈금.
   - 상단 또는 하단에 축척비 표기 (`SCALE 1:40`, `SCALE 1:20`, `1:10` 등).
   - 단위 표기: `0`, `1`, `2`, `5`, `10 m` (또는 유물용 `cm`).
2. **엔진 명령**: `sheet.addScaleBar`
   - 매개변수: `{x?: number, y?: number, scale_ratio?: number, unit?: string, total_units?: number}`

### 2.4 국가유산 발굴조사 정규 표제란 (Standard Title Block)
1. **배치 및 삼선표 양식**:
   - 도판 우측 하단 배치 (기본 폭 120 mm / $340.16\text{ pt}$, 4단 행 구조).
   - 테두리: 상·하단 1.5 pt, 행간 0.5 pt, 좌우 외곽 1.0 pt.
   - 항목:
     1. 유적명 (Site Name): 예) `경주 황오동 고분군 발굴조사`
     2. 도면명 (Drawing Title): 예) `1호 주거지 평면 및 토층 단면도`
     3. 축척 (Scale): 예) `1:40 (단면 1:20)`
     4. 조사기관 및 일자 (Institute & Date): 예) `국립문화유산연구원 · 2026. 10`
     5. 도판 번호 (Plate No): 예) `PLATE 3`
2. **엔진 명령**: `sheet.addTitleBlock`
   - 매개변수: `{x?: number, y?: number, site?: string, title?: string, scale?: string, institute?: string, date?: string, plate_no?: string}`

### 2.5 자동 추출 범례표 (Extracted Legend Table)
1. **기능**:
   - 현재 도면에 존재하는 레이어와 범례(구조선, 굴광선, 토층선, 소토면, 표고점 등)를 자동으로 수집하여 도판 한 켠에 정갈한 범례 박스를 생성.
2. **엔진 명령**: `sheet.generateLegendTable`
   - 매개변수: `{x?: number, y?: number, width?: number}`

### 2.6 도판 속성 설정
- 엔진 명령: `sheet.setProps`
  - 매개변수: `{title?: string, plate_no?: string, site?: string, institute?: string, date?: string}`

---

## 3. UI 및 컨트롤 바 연동
1. **도판 모드 컨트롤 바**:
   - `도북 방위`: 클릭 시 우측 상단 표준 위치에 도북 자동 배치.
   - `축척 막대`: 클릭 시 현재 도면 축척(1:40 등)에 맞는 축척자 배치.
   - `정규 표제란`: 클릭 시 우측 하단에 정규 표제란 자동 배치.
   - `범례표`: 클릭 시 사용된 레이어 범례표 생성.
   - `PDF 내보내기`: 원클릭 PDF 내보내기.
2. **속성 패널 (Properties)**:
   - 도판 번호, 도판 제목, 유적명, 기관명, 축척 정보 표시 및 편집.

---

## 4. 검증 기준
1. `cargo fmt --all -- --check` 통과.
2. `cargo clippy --workspace --all-targets -- -D warnings` 통과.
3. `cargo test -p vectorcraft-engine cmd::sheettools` 통과 (4/4 통과).
4. `cargo test -p vectorcraft-ui-egui trowel::` 전체 테스트 통과 (85/85 통과).
5. `cargo test -p vectorcraft-ui-egui i18n` 10개 언어 전체 통과 (37/37 통과).
6. `cargo xtask ci` 7개 전체 관문 통과 (fmt, clippy, test, assets, brands, layers, wasm).
7. Git 커밋, 원격 푸시, GitHub Actions CI 확인.

---

## 5. 완료 서명 (Sign-off)
- 상태: **done** (100% 완료 및 검증 통과)
- 일자: 2026-10-14
- 검증 결과:
  - 엔진 명령 5종 구현 완료 (`sheet.addNorthArrow`, `sheet.addScaleBar`, `sheet.addTitleBlock`, `sheet.generateLegendTable`, `sheet.setProps`).
  - 컨트롤 바 퀵 액션 단추 4종 연동 (`North Arrow`, `Scale Bar`, `Title Block`, `Legend Table`).
  - 속성 패널(Properties) 종합 도판 메타데이터 표시 (`Sheet` 항목).
  - 10개 언어 카탈로그 완비 및 `cargo xtask ci` 7개 전 관문 무결성 통과.

