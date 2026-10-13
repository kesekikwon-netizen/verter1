# J002b — 누르고 있는 동안만 Z 돋보기 (스프링 로드 줌)

상태: done (2026-10-13) · 기준 커밋 `7b4281ff` · 검증: `cargo xtask ci` 전체 7개 관문 통과 · 커밋 1개 (`J002b: …`)

## 목표

도구 사용 중 단축키 Z를 길게 누르거나(0.3초 초과) 누른 채로 드래그하여 줌한 뒤 Z를 놓으면 원래 사용하던 이전 도구로 복귀하는 스프링 로드 돋보기(Spring-loaded Z Loupe)를 구현한다. 반면 Z를 0.3초 이내로 짧게 누르고 떼면 기존과 동일하게 줌 도구로 영구 전환된다.

1. 이전 도구가 줌 도구가 아닐 때 Z를 누르면:
   - 현재 도구(`previous_tool`)와 누른 시간(`pressed_time`)을 기록하고 줌 도구(`zoom`)로 전환한다.
2. Z를 누르고 있는 동안:
   - 캔버스에서 드래그 조작이 발생하면 `dragged = true`로 기록한다.
3. Z 키를 뗄 때(Release):
   - 누르고 있던 시간이 0.3초를 초과했거나 드래그가 발생한 경우:
     - 드래그가 진행 중이면 드래그 완료 시점에, 아니면 즉시 이전 도구(`previous_tool`)로 복귀한다.
   - 누르고 있던 시간이 0.3초 이하이고 드래그가 없었던 경우:
     - 줌 도구 상태를 그대로 유지한다(단축키 탭 동작).
4. 이전 도구가 이미 줌 도구인 경우에는 도구를 빌리지 않고 줌 도구를 유지한다.
5. 텍스트 입력 중이거나 수정 중일 때는 스프링 로드가 발동하지 않는다.

## 완료 기준

1. `crates/ui-egui/src/shortcuts.rs`:
   - `SpringZoom` 상태 구조체 정의:
     ```rust
     #[derive(Clone, Debug)]
     pub struct SpringZoom {
         pub previous_tool: String,
         pub pressed_time: f64,
         pub dragged: bool,
     }
     ```
   - `VectorcraftApp`에 `pub(crate) spring_zoom: Option<SpringZoom>` 및 `pub(crate) spring_zoom_revert: Option<String>` 필드 추가.
2. Z 키 누름/뗌 감지:
   - `shortcuts::handle`:
     - 단일 키 도구 단축키 처리 시 `zoom` 도구 선택 시점:
       - 이전 도구가 `"zoom"`이 아니면 `app.spring_zoom`에 이전 도구와 현재 시간(`ctx.input(|i| i.time)`)을 저장.
     - 키 뗌(`Event::Key { key: Key::Z, pressed: false, .. }` 또는 물리 키 `Key::Z`):
       - `app.spring_zoom`이 있으면 경과 시간 및 드래그 여부 판정:
         - `dragged || (now - pressed_time) > 0.3`인 경우:
           - 현재 캔버스 드래그 중이면 `app.spring_zoom_revert = Some(previous_tool)`에 보관 후 드래그 종료 시 복귀.
           - 아니면 `app.select_tool(&previous_tool)`.
         - 짧은 탭(<= 0.3s, no drag)인 경우: `zoom` 도구 유지.
         - `app.spring_zoom = None`.
3. 캔버스 드래그 연동:
   - `crates/ui-egui/src/canvas.rs`:
     - 줌 드래그(`Drag::ZoomScrub`, `Drag::ZoomBox`) 발생 시 `app.spring_zoom`의 `dragged`를 `true`로 설정.
     - 드래그 종료(`PointerKind::Up` 또는 마우스 버튼 해제) 시 `app.spring_zoom_revert`가 있으면 해당 도구로 복귀하고 초기화.
4. 예외 및 안전 처리:
   - 창 포커스 해제(`Event::WindowFocused(false)`) 또는 명시적 다른 도구 선택 시 `spring_zoom` 초기화.
   - 텍스트 입력 필드나 Type 도구 텍스트 편집 중에는 동작하지 않음.
5. 단위 및 통합 시험:
   - `crates/ui-egui/src/shortcuts.rs` 및 `canvas.rs`에 시험 추가:
     - 0.3초 미만 탭: 줌 도구 유지
     - 0.3초 초과 홀드 후 뗌: 이전 도구 복귀
     - 누른 채 드래그 후 뗌: 이전 도구 복귀
     - 줌 도구에서 Z 누름: 줌 도구 유지
6. 품질 검증:
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test -p vectorcraft-ui-egui`
   - `cargo xtask ci` 전체 관문 통과.

## 변경 범위

- `crates/ui-egui/src/lib.rs`: `VectorcraftApp`에 `spring_zoom` 필드 추가.
- `crates/ui-egui/src/shortcuts.rs`: `SpringZoom` 정의, 키 누름/해제 처리, 단위 시험.
- `crates/ui-egui/src/canvas.rs`: 줌 드래그 시 `dragged` 플래그 갱신 및 드래그 종료 시 복귀 처리.
- `factory/backlog.md`: J002b 상태 갱신.
