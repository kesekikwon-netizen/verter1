# J008 — 밑그림 맞추기 (2점·4점 정합, 상사/아핀 왜곡 보정, 오차 잔차 계산 및 대화상자)

상태: done (2026-10-14) · 기준 커밋 `68e6ff09` · 검증: `cargo xtask ci` 7개 전체 품질 게이트 통과 (fmt, clippy, test, assets, brands, layers, wasm) · **스캔 실측도 2점/4점 정합 및 잔차 오차 RMSE <= 2.0 mm 완벽 검증**

## 목표

청사진(`docs/design/trowel-design-blueprint.html` 6절 Dialog 2) 및 백로그 항목 10에 따라:
현장 방안지 실측도 스캔본(300 DPI) 또는 트렌치 수직 사진을 **밑그림(Underlay)** 레이어에 배치하고, 2점(상사 변환) 또는 4점(아핀 변환)의 기준점(GCP)을 매칭하여 실제 도면 축척(1:20 등) 및 국가 TM 좌표계에 맞추고 오차 잔차(RMSE)를 계산하는 기능을 구현한다.

1. **정합 수학 알고리즘 (Registration Mathematics)**:
   - **2점 상사 정합 (2-Point Similarity)**:
     - 회전(Rotation), 등방 축척(Uniform Scale), 평행 이동(Translation) 4개 자유도 해석.
     - 두 점 사이의 길이 비와 회전각을 계산하여 아핀 변환 행렬 도출.
   - **4점 아핀 정합 (4-Point Affine)**:
     - 4개 GCP 짝을 이용한 최소자승법(Least Squares) 최적 아핀 변환 행렬($3 \times 3$) 추정.
     - 방안지의 온·습도 불균일 신축(X축/Y축 독립 신축 및 전단 왜곡) 완벽 보정.
   - **오차 잔차(Residual Error) 및 RMSE 계산**:
     - 각 GCP별 잔차 오차 $r_i = \|M p_i - q_i\|$ 계산.
     - 종합 평균 제곱근 오차 $\text{RMSE} = \sqrt{\frac{1}{n}\sum r_i^2}$ 계산.
     - 도면 축척(예: 1:20)을 반영한 **종이 실측 오차(mm)** 환산 및 국가유산 발굴조사 규격(RMSE $\le 2.0\text{ mm}$) 적합성 판정.

2. **엔진 명령 (Engine Commands)**:
   - `underlay.align`: `{ id?: NodeId, method: "twoPoint" | "affine", gcps: [{ scan: [x, y], target: [x, y] }] }` -> 밑그림 이미지 노드의 변환 행렬 갱신 및 잔차 결과 반환. 단일 실행 취소(Undo) 지원.
   - `underlay.estimate`: 계산 결과 및 잔차만 반환하는 무부작용 쿼리 명령.

3. **대화상자 카드 UI (`crates/ui-egui/src/trowel/underlay_card.rs`)**:
   - 스캔 밑그림 파일 정보 표시.
   - 정합 방식 선택 세그먼트: `2점 정합 (회전·축척)` vs `4점 아핀 정합 (왜곡 보정)`.
   - GCP 삼선표(Three-line table): GCP 번호, 스캔 좌표(px), 국가 TM 좌표/도면 좌표, 잔차 오차(mm).
   - 종합 오차 분석: RMSE 및 허용치 2.0 mm 이내 만족 여부(`t.done` 녹색 / 초과 시 경고).
   - 주 동작 버튼: `밑그림 정합 실행` (Clay 스타일) 및 `취소`.

4. **리본 및 메뉴 연동**:
   - `crates/ui-egui/src/trowel/ribbon.rs`: `tile("align", "tw-align", tw!("Align"), Act::Align)` 활성화.
   - `crates/ui-egui/src/trowel/menu.rs`: 도면(Drawing) 메뉴의 `밑그림 맞추기…` 활성화.

## 검증 계획

1. 2점 상사 정합 및 4점 아핀 정합 수치 수학 단위 테스트 (오차 < 1e-9).
2. 일직선 상의 특이점(Collinear Degenerate) 입력 시 무패닉 및 적절한 에러 반환 검증.
3. 엔진 커맨드 `underlay.align`의 변환 행렬 적용 및 Undo/Redo 트랜잭션 롤백 테스트.
4. Trowel UI 하네스를 통한 리본 단추 클릭 -> 대화상자 카드 열림 -> 정합 실행 연동 테스트.
5. 다국어 TSV 카탈로그 11개 언어 등록.
6. `cargo xtask ci` 7개 전체 품질 게이트 통과.
