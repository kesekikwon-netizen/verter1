# J017: 펜 2단계 (옆버튼, 지우개 끝, 손바닥 거르기, 펜 기울기 연동)

상태: done · 기준 커밋 `6130b1c7` · 검증: `cargo test` 및 `cargo xtask ci` 7개 전 관문 100% 통과

## 1. 개요 및 배경
와콤(Wacom), 엑스피펜(XP-Pen), 애플펜슬 등 발굴 현장 및 연구실 제도 환경에서 사용되는 전문 스타일러스 펜 타블렛의 2단계 고도화 작업(B1–B4)이다.
1단계(J016)에서 Windows Ink 플릭/길게 누르기 해제, 133~240Hz 서브프레임 점 수집, 3pt 탭·8px 닫기 보정, 캔버스 단추줄, 대화형 시험장 및 균일/필압 모드를 구축한 데 이어, 2단계에서는 **스타일러스 하드웨어 제어의 완전성**을 달성한다.

---

## 2. 세부 기능 요구사항 (B1–B4)

### 2.1 B1: 펜 지우개 끝 자동 반전 (Stylus Eraser Tip Inversion)
1. **동작 원리**:
   - 와콤 및 호환 스타일러스의 뒤쪽 지우개 끝(Eraser Tip)이 태블릿 디지타이저에 닿았을 때(`DeviceKind::Eraser`, 보조 버튼 또는 지우개 입력 감지), 현재 손에 쥔 도구(펜, 연필 등)를 영구적으로 바꾸지 않고 **지우는 동안만 임시 지우개(Eraser) 도구로 자동 전환**하여 획을 지움.
   - 지우개 끝을 떼면 직전에 쓰던 도구(예: 펜 도구, 토층선 도구)로 즉시 복귀.
2. **환경설정**:
   - `pen_eraser_invert: bool` (기본값: `true`, `penEraserInvert`).

### 2.2 B2: 펜 옆버튼 사용자 할당 (Stylus Barrel Buttons)
1. **동작 원리**:
   - 스타일러스 자루에 위치한 1번/2번 사이드 버튼(Barrel Button 1 & 2) 입력을 가로채 사용자가 지정한 도구/명령으로 즉시 실행.
   - 1번 옆버튼 (`pen_barrel_button_1`): 기본값 `"pan"` (화면 이동 / 손 도구 임시 활성화, 누른 채 드래그 시 캔버스 팬).
   - 2번 옆버튼 (`pen_barrel_button_2`): 기본값 `"menu"` (우클릭 컨텍스트 메뉴 또는 실행 취소 `"undo"`, 임시 돋보기 `"zoom"`).
2. **지원 동작 옵션**:
   - `"pan"`: Hand / Pan View (스페이스바와 동일).
   - `"undo"`: 즉각 실행 취소 (`edit.undo`).
   - `"eraser"`: 임시 지우개 모드.
   - `"zoom"`: 임시 돋보기 스크러빙 줌.
   - `"menu"`: 우클릭 상황 메뉴.
   - `"none"`: 무동작.
3. **환경설정**:
   - `pen_barrel_button_1: String` (기본: `"pan"`).
   - `pen_barrel_button_2: String` (기본: `"menu"`).

### 2.3 B3: 손바닥 거르기 (Palm Rejection Filtering)
1. **동작 원리**:
   - 스타일러스 펜이 화면에 닿아 있거나 펜 입력이 감지된 직후(500 ms 윈도우)에는 터치스크린에 닿는 손바닥(Palm)이나 손가락 접촉으로 인한 캔버스 터치 제스처, 오터치 획 생성을 완벽하게 차단.
   - 펜 획 작업 중 손바닥이 화면을 스치더라도 도면이 튀거나 원치 않는 앵커가 찍히는 문제를 근본적으로 차단.
2. **환경설정**:
   - `pen_palm_rejection: bool` (기본값: `true`, `penPalmRejection`).

### 2.4 B4: 펜 기울기(Tilt) 및 실시간 스타일러스 상태 감지
1. **동작 원리**:
   - 스타일러스의 경사각($\text{Tilt } X, Y$ 각도)을 감지하고 보정.
   - 타블렛 감도 설정 카드(`tablet_card.rs`)의 대화형 펜 시험장(`pen_test_area`)에 실시간 스타일러스 상태 표시:
     - 현재 펜 끝 상태 (일반 펜촉 vs 뒤쪽 지우개).
     - 옆버튼 클릭 상태 감지 인디케이터.
     - 기울기(Tilt) 각도 수치 표시.
     - 손바닥 거르기(Palm Rejection Active) 상태 표시.

---

## 3. 검증 기준
1. `cargo fmt --all -- --check` 통과.
2. `cargo clippy --workspace --all-targets -- -D warnings` 통과.
3. `cargo test -p vectorcraft-engine tests_prefs` 및 펜 환경설정 테스트 통과.
4. `cargo test -p vectorcraft-ui-egui trowel::` 전체 테스트 통과.
5. `cargo test -p vectorcraft-ui-egui i18n` 전체 37개 테스트 통과.
6. `cargo xtask ci` 7개 전체 관문 통과.
7. Git 커밋, 원격 푸시, GitHub Actions CI 확인.
