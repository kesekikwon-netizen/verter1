# J010: 단면·토층 화면 (Section & Soil Stratum View)

## 1. 개요 및 배경
한국 매장유산 발굴조사에서 유구(주거지, 수혈, 분묘, 성벽 등)의 **단면(Section)과 토층(Stratigraphy)** 실측도는 층위학적 퇴적 과정과 시기 구분의 절대적 근거가 되는 핵심 도면이다.
본 작업(J010)은 Strata(hgis) 연계 조용한 도판(Quiet Plate) 기준에 맞춰, 현업 발굴조사원이 단면 실측과 토층 주기를 신속하고 정확하게 완성할 수 있는 전용 제도 시스템을 구축한다.

---

## 2. 세부 기능 요구사항

### 2.1 기준 표고선 및 거리 축 (Datum Line & Distance Axis)
1. **기준 표고선 (Base Datum Line)**:
   - 단면 실측의 절대적 기준선(EL. m, 예: `152.50 m`).
   - 1.5 pt 선 굵기, `#C46686` (또는 지정 참조색), 좌측 시작점에 굵은 텍스트 `기준선 EL. {el:.2} m`.
   - 엔진 명령: `section.setProps`, `section.getProps`.
2. **해발선 연동 (Level Lines)**:
   - J015 `drawing.makeLevelLines`와 연동하여 기준 표고선 위아래로 50 cm(0.5 pt) 및 10 cm(0.2 pt 점선/쇄선) 간격 해발선과 `EL. {el} m` 표고 라벨 자동 배치.
3. **수평 거리 눈금자 (Distance Axis)**:
   - 단면 하단에 1.0 m 단위 주 눈금선 및 텍스트(`0.0 m`, `1.0 m`, `2.0 m`...), 0.5 pt 선 굵기.

### 2.2 굴광 절단면 (Cut Face) 및 토층선 (Stratum Boundary)
1. **굴광선 및 절단면 (Excavation Cut Face)**:
   - 트렌치/유구 굴착 경계선 0.8 pt 외곽선.
   - J014 잘린 면 빗금 해칭 연동: 45° 각도, 1.0 mm 간격, 0.1 pt 빗금선.
2. **토층 분할선 (Stratum Boundary Line)**:
   - 토층 경계를 나누는 0.3 pt 실선 (`drawing.addStratumLine`).
   - 기본 타깃 레이어: `"토층선"` (weight=0.3 pt).
3. **소토면 및 특수 층위 (Burnt Clay & Special Inclusions)**:
   - 소토면/노지 단면: 클레이 색상(`#F3DEDA` 채움, 점토 굵기 선, 텍스트 `소토면`).

### 2.3 토층 번호 및 속성 배지 (Stratum Badge)
1. **토층 번호 원형 마크**:
   - 원형 배지(반경 9.0 pt, 흰색 채움, 1.0 pt 먹색 테두리).
   - 중앙 원문자(①, ②, ③, ④... 및 20 초과 시 아라비아 숫자 1, 2, 3).
   - 우측에 층위 기본 명칭(예: `① 표토`, `② 암갈색 사질점토`, `③ 주거지 바닥 다짐토`, `④ 생토층`).
   - 속성 메타데이터 연동:
     - `stratum:number` (1, 2, 3...)
     - `stratum:name` (표토, 암갈색 사질점토...)
     - `stratum:munsell` (Munsell 토색: 예 `10YR 4/3 갈색`)
     - `stratum:texture` (토성: 사질점토, 점토질모래, 점토...)
     - `stratum:inclusions` (포함유물 및 퇴적 특징)
   - 엔진 명령: `drawing.addStratumBadge`.

### 2.4 하단 삼선표 토층 해설표 (Three-line Stratum Table)
1. **표준 학술 삼선표 양식**:
   - 상단 테두리선: 1.5 pt.
   - 헤더 구분선: 1.0 pt.
   - 하단 테두리선: 1.5 pt.
   - 배경: `#FAF9F5` (패널 서피스) 또는 투명.
2. **열 구성**:
   - `층위` (Stratum No.): 원문자 `①`, `②`, `③`...
   - `토색 (Munsell)`: 예 `10YR 4/3 갈색`, `10YR 3/2 암갈색`
   - `토성`: 예 `사질점토`, `점토질모래`, `점토`
   - `포함유물 및 퇴적 특징`: 예 `표토 및 현대 식생 부식토, 소량의 자기편`, `주거지 바닥 다짐토, 목탄편 집중, 갈판 완형 출토`
3. **엔진 명령**:
   - `drawing.generateStrataTable`: 문서 내 토층 배지들을 수집하여 규격 삼선표 그룹 노드를 자동 생성/갱신.

### 2.5 단면 수치 계산 (Section Metrics)
1. **단면 너비 및 최대 심도**:
   - 단면 너비(m): $W = (x_{\max} - x_{\min}) / \text{ppm}$.
   - 최대 심도(m): 기준 표고선(Datum Y) 대비 가장 깊은 굴광 바닥점까지의 수직 거리 $D = (y_{\text{bottom}} - y_{\text{datum}}) / \text{ppm}$.
   - 엔진 명령: `section.calculateMetrics`.

---

## 3. UI 및 컨트롤 바 연동
1. **단면 모드 컨트롤 바**:
   - `단면명`: `A-A' 북벽` 드롭다운/입력.
   - `기준 표고`: `152.50 m` 스피너/입력.
   - `해발선`: 클릭 시 자동 생성 (`drawing.makeLevelLines`).
   - `잘린 면`: 45° 1.0 mm 빗금 적용 (`palette-cut`).
   - `토층표`: 토층표 갱신 및 내보내기 단추.
2. **속성 패널 (Properties)**:
   - 단면 너비 및 최대 심도 표시 (`단면 너비 4.60 m · 최대 심도 0.65 m`).
   - 토층 선택 시 Munsell 토색, 토성, 퇴적 특징 편집 필드 제공.

---

## 4. 검증 기준
1. `cargo fmt --all -- --check` 통과.
2. `cargo clippy --workspace --all-targets -- -D warnings` 통과.
3. `cargo test -p vectorcraft-engine cmd::stratatools` 통과.
4. `cargo test -p vectorcraft-tools stratatools` 통과.
5. `cargo test -p vectorcraft-ui-egui trowel::` 전체 테스트 통과.
6. `cargo xtask ci` 7개 전체 관문 통과.
7. Git 커밋, 원격 푸시, GitHub Actions CI 확인.
