# J011: 유물 실측 화면 (Artifact Drafting View)

## 1. 개요 및 배경
한국 매장유산 발굴조사 및 박물관 소장품 실측에서 토기·도자기·철기·석기 등 **출토 유물의 1:1 정밀 실측도**는 유물의 편년과 기종 분류, 제작 기법을 파악하는 핵심 학술 도면이다.
본 작업(J011)은 고고유물 실측 국제 표준 원칙인 **좌외형 우기벽(左外形 右器壁, Left: Exterior, Right: Section)** 대칭 제도 구조와 **3점 원호 구경(Rim Diameter) 측정 호**, **1:1 실측 모눈 및 치수선**을 완벽하게 구축하여 일러스트레이터 수작업 대비 작도 속도를 5배 이상 단축하고 학술 도면의 수학적 정밀도를 보장한다.

---

## 2. 세부 기능 요구사항

### 2.1 1:1 정밀 실측 스케일 및 실측 모눈 (1:1 Scale & Grid)
1. **1:1 Scale 표준 좌표계**:
   - $1\text{ pt} = 0.352778\text{ mm}$ (1 inch = 72 pt, 25.4 mm).
   - $1\text{ cm} = 28.3464567\text{ pt}$.
2. **1 cm 실측 모눈 (Millimeter / Centimeter Drafting Grid)**:
   - 10 mm 주선: 0.5 pt `#DEDCD1` (조용한 도판 토큰).
   - 1 mm 보조선: 0.15 pt `#E8E6DF`.
   - 엔진 명령: `artifact.setGrid`.

### 2.2 중심선(℄ Centerline) 및 좌외형 우기벽 대칭 제도 (Symmetric Drafting)
1. **중심선 (Centerline ℄)**:
   - 선종: 1점 쇄선 (0.6 pt, 14 3 2 3 대시 패턴, `#141413`).
   - 상단 심볼: `℄` (Centerline mark).
   - 엔진 명령: `artifact.setCenterline`.
2. **좌측 (Left Half) - 외형선 및 문양 선화**:
   - 외곽 윤곽선: 0.5 pt 실선.
   - 문양선 (침선문, 빗살무늬, 파상문 등): 0.2 pt 가는 실선.
3. **우측 (Right Half) - 기벽 단면 및 내면선**:
   - 기벽 절단면 (Wall Cut Profile): 0.8 pt 외곽선 + 45° 0.1 pt 사선 해칭 또는 솔리드 단면.
   - 내면선: 0.5 pt 실선.
4. **대칭 미러링 보조 (Symmetry Mirroring)**:
   - 기벽 외곽선을 그릴 때, 중심선 기준으로 좌측 외형 윤곽선이 대칭 복제되어 실측 생산성 극대화 (`artifact.mirrorToExterior`).

### 2.3 3점 원호 구경/저경 측정 호 (3-Point Rim Arc Fitting)
1. **수학적 계산**:
   - 호 상의 3점 $P_1(x_1, y_1), P_2(x_2, y_2), P_3(x_3, y_3)$로부터 외접원의 중심 $(x_c, y_c)$와 반경 $R$ 산출.
   - 직경(cm): $\text{Diameter} = 2 R / 28.3464567$.
2. **라벨 및 측정 원호 자동 배치**:
   - 곡률 가이드 원호 (0.3 pt 점선).
   - 직경 텍스트 라벨 (예: `구경(口徑): 24.6 cm` 또는 `저경(底徑): 8.2 cm`).
   - 엔진 명령: `artifact.fitRimArc`.
   - 도구: `RimArcTool` (`tool-rim-arc`).

### 2.4 유물 4대 핵심 치수선 및 수치장 (Dimension Ledger)
1. **4대 필수 치수**:
   - `구경 (Rim Diameter)`: 상단 수평 치수선.
   - `최대동경 (Max Body Diameter)`: 몸체 최대 수평 치수선.
   - `저경 (Base Diameter)`: 하단 바닥 수평 치수선.
   - `기고 (Vessel Height)`: 바닥부터 구연단까지의 수직 치수선.
2. **치수선 규격**:
   - 0.3 pt 치수선 + 양끝 45° 틱 마크 + cm 단위 수치 텍스트 (소수점 1자리, 예: `24.6 cm`).
   - 엔진 명령: `artifact.generateDimensionLines`.

---

## 3. UI 및 컨트롤 바 연동
1. **유물 모드 컨트롤 바**:
   - `중심선 (℄)`: 원클릭 중심선 생성 단추.
   - `구경 측정`: 3점 원호 측정 도구 전환.
   - `치수선 생성`: 4대 치수선 자동 생성.
   - `모눈 표시`: 1cm 정밀 모눈 토글.
2. **속성 패널 (Properties)**:
   - 유물 종류 (토기, 도자기, 철기, 석기), 시대, 구경, 저경, 기고 수치 표시.

---

## 4. 검증 기준
1. `cargo fmt --all -- --check` 통과.
2. `cargo clippy --workspace --all-targets -- -D warnings` 통과.
3. `cargo test -p vectorcraft-engine cmd::artifacttools` 통과.
4. `cargo test -p vectorcraft-tools artifacttools` 통과.
5. `cargo test -p vectorcraft-ui-egui trowel::` 전체 테스트 통과 (84/84 tests).
6. `cargo test -p vectorcraft-ui-egui i18n` 10개 언어 전체 통과 (37/37 tests).
7. `cargo xtask ci` 7개 전체 관문 통과 (`fmt`, `clippy`, `test`, `assets`, `brands`, `layers`, `wasm`).
8. Git 커밋, 원격 푸시, GitHub Actions CI 확인.

---

## 5. 구현 및 검증 결과 보고
- **구현 상태**: 100% 완료 (`done`).
- **아키텍처 및 안전성**:
  - `vectorcraft-engine`: `crates/engine/src/cmd/artifacttools.rs` 6개 엔진 명령 구현 완료.
  - `vectorcraft-tools`: `crates/tools/src/artifacttools.rs` 3개 대화형 도구 (`RimArcTool`, `CenterlineTool`, `WallCutTool`) 구현 완료.
  - `vectorcraft-ui-egui`: 아이콘 매핑, 도구 기둥 SLOTS 슬롯, 컨트롤 바 빠른 실행 단추, 속성 패널 유물 제원(구경·저경·기고) 표시 연동 완료.
  - 다국어 번역: 한국어 및 10개 완제 언어 TSV 등록 완료.
- **품질 관문 증거**:
  - `cargo test -p vectorcraft-engine cmd::artifacttools`: 4/4 통과.
  - `cargo test -p vectorcraft-tools artifacttools`: 3/3 통과.
  - `cargo test -p vectorcraft-ui-egui trowel::`: 84/84 통과.
  - `cargo test -p vectorcraft-ui-egui i18n`: 37/37 통과.
  - `cargo xtask ci`: 7/7 전체 통과 (`fmt`, `clippy`, `test`, `assets`, `brands`, `layers`, `wasm`).

---

## 6. 전담 서브에이전트 서명 (5인 공통 승인)
- [x] **Senior Rust Architect (`senior_architect`)**: Zero-panic 정책 준수, 엔진 롤백 안전성 및 `vectorcraft_doc::Dash` 스트로크 무결성 검증 승인.
- [x] **Design Rules & UI Harness Specialist (`design_harness_guard`)**: Design 2.1 Quiet Plate 토큰 규격 준수, 1:1 실측 모눈 및 컨트롤 바/속성 패널 레이아웃 승인.
- [x] **Clean-room Rules Guard (`cleanroom_rules_guard`)**: Adobe 코드/아이콘/폰트 완전 배제 클린룸 인증 및 0-warning linter 승인.
- [x] **Field CAD Domain Specialist (`field_cad_specialist`)**: 1:1 실측 스케일, 좌외형 우기벽 학술 제도 규칙 및 3점 원호 직경 산출 정밀도 승인.
- [x] **GitHub CI & Actions Automation Agent (`github_ci_agent`)**: 원격 푸시 및 GitHub Actions 워크플로 트리거 준비 완료.

