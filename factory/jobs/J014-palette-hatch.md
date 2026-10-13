# J014 — 그리기 팔레트와 빗금, 잘린 면

상태: done (2026-10-13) · 기준 커밋 `af91965ea12e4d2a8b221a049757a6c6ad1b15ec` · 검증: `cargo xtask ci` 7개 관문 모두 통과 · **◆ 파일 형식**(레이어 범례 `hatch` 필드, 너그러운 읽기) · **◆ 수치**(종이 간격 1.0 mm, 0.8 pt 외곽 테두리 + 0.1 pt 빗금 선, 45°/135° 각도)

## 목표

디자인 2.1 7절 및 고고학 발굴도면 제도 표준에 따라, 그리기 팔레트의 빗금(Hatch 45°·135°·번갈아·없음)과 「잘린 면」(Cut Face, 0.8 pt 테두리 + 0.1 pt 빗금), 그리고 컨트롤 바의 「추정(파선)」을 실제 동작하는 기능으로 구현한다.

- 그리기 팔레트(`palette.rs`)의 빗금 칩 4종(45°, 135°, 번갈아, 없음)과 「잘린 면」 단추를 활성화한다.
- 컨트롤 바(`control.rs`)의 「추정 (파선)」(`ctl-estimate`) 토글을 활성화한다.
- 행 편집 카드(`rowedit.rs`)의 빗금 세그먼트를 활성화하여 레이어 범례(`Legend`)에 빗금(`Hatch`) 속성을 저장하고 연동한다.
- 「잘린 면」 명령(`drawing.cutFace`): 선택된 닫힌 도형에 대해 외곽선 0.8 pt를 부여하고, 내부를 종이 간격 1.0 mm(약 2.835 pt)의 0.1 pt 빗금 벡터 선으로 채운다.
- 인접한 두 면이 맞닿아 있으면 빗금 각도를 45°와 135°로 엇갈리게 자동 배치하고, 3개 면이 서로 맞닿아 2색 엇갈림이 불가능한 경우 주의(!) 메시지를 제공한다.
- 생성된 빗금은 실제 벡터 패스(`NodeKind::Path`)로 추가되어 SVG 및 PDF 내보내기 시 실제 인쇄 선으로 출력된다.
- 메뉴 줄 `도면 › 잘린 면`을 활성화한다.

## 완료 기준

1. **범례 모형과 파일 형식 (`crates/doc/src/legend.rs`).**
   - `Hatch` 열거형 추가: `None`, `Deg45` ("45"), `Deg135` ("135"), `Alternate` ("alt").
   - `Legend` 구조체에 `pub hatch: Hatch` 필드 추가 (`#[serde(default, skip_serializing_if = "crate::skip::is_default")]`).
   - 기본값이 아닐 때만 직렬화되며, 기존 파일 역직렬화 시 `Hatch::None`으로 안전하게 호환됨.
   - `legend.set` 명령에 `hatch?: none|45|135|alt` 매개변수 지원.

2. **빗금 기하학 엔진 (`crates/tools/src/builder.rs` 또는 `crates/engine/src/cmd/drawing.rs`).**
   - 닫힌 패스에 대해 임의의 각도(45°, 135°) 및 지정된 간격(1.0 mm = 2.8346 pt)으로 빗금 선분을 계산하는 알고리즘 구현.
   - 0.1 pt stroke, 검정색 먹선, fill none의 실제 패스 객체 생성.

3. **잘린 면 명령 (`drawing.cutFace`).**
   - 선택된 객체의 외곽선을 0.8 pt로 설정.
   - 내부 0.1 pt 빗금 선 생성 (그룹으로 묶거나 레이어에 배치).
   - 인접 판정 및 각도 엇갈림 (45° vs 135°).
   - 3개 상호 인접 시 `warning` 안내 반환.
   - Undo/Redo(실행 취소) 1단계 보장.

4. **UI 그리기 팔레트 (`crates/ui-egui/src/trowel/palette.rs`).**
   - 빗금 칩(45, 135, alt, none) 클릭 시 활성화 표시(`row_selected`) 및 선택된 객체(또는 다음 그릴 선)에 빗금 적용.
   - 「잘린 면」 단추(`palette-cut`) 클릭 시 `drawing.cutFace` 실행.

5. **컨트롤 바 (`crates/ui-egui/src/trowel/control.rs`).**
   - 「추정 (파선)」(`ctl-estimate`) 토글 클릭 시 파선(Dashed) 선종 적용/해제.

6. **메뉴 줄 (`crates/ui-egui/src/trowel/menu.rs`).**
   - `도면 › 잘린 면` 항목 활성화 및 실행.

7. **행 편집 카드 (`crates/ui-egui/src/trowel/rowedit.rs`).**
   - `card-hatch` 세그먼트 활성화 및 `legend.set` 연동.

8. **품질 검증.**
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test -p vectorcraft-ui-egui trowel::`
   - `cargo xtask ci` 통과.

## 변경 범위

- `crates/doc/src/legend.rs`: `Hatch` enum, `Legend` 필드, 직렬화/역직렬화 및 시험.
- `crates/engine/src/cmd/legend.rs` & `crates/engine/src/cmd/`: `legend.set`의 `hatch` 처리 및 `drawing.cutFace` 명령.
- `crates/ui-egui/src/trowel/palette.rs`: 빗금 칩 활성화, 잘린 면 단추 연결.
- `crates/ui-egui/src/trowel/control.rs`: 추정(파선) 토글 활성화.
- `crates/ui-egui/src/trowel/rowedit.rs`: 빗금 세그먼트 활성화.
- `crates/ui-egui/src/trowel/menu.rs`: 도면 › 잘린 면 메뉴 연결.
- `crates/ui-egui/src/trowel/tests.rs`: UI 시험 갱신 및 신규 시험 추가.

## 기준 커밋
`af91965ea12e4d2a8b221a049757a6c6ad1b15ec`
