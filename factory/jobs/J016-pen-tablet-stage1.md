# J016 — 펜 1단계 (와콤/XP-Pen 윈도우 제스처 해제, 서브프레임 점 수집, 3pt/8px 탭·닫기, 캔버스 단추줄, 펜 시험장, 필압 제어)

상태: done (2026-10-14) · 기준 커밋 `fe82c724` · 검증: `cargo test` (엔진 1702, 도구 290, trowel 80 전체 통과) 및 `cargo clippy -D warnings` 통과 · **와콤 / XP-Pen / 애플펜슬 타블렛 1단계 연동 완료**

## 목표

사용자 지시, 백로그 항목 9(A1–A7), 및 청사진(`docs/design/trowel-design-blueprint.html` 6절 주요 대화상자)에 따라:
와콤(Wacom), 엑스피펜(XP-Pen), 애플펜슬 등 고고학 발굴 실측 제도용 스타일러스 펜 타블렛의 지연과 오류를 방지하고, 윈도우 제스처 무력화, 한 프레임 내 다중 펜 샘플 수집, 정밀 탭(3 pt) 및 닫기(8 px) 보정, 캔버스 단추줄, 대화형 펜 시험장, 컨트롤 바 필압 모드(균일 굵기 / 필압 따라)를 구현한다.

1. **A1: Windows 펜 몸짓 끄기 (Windows Pen Flicks & Press-and-Hold Disable)**:
   - Windows Ink 환경에서 펜을 길게 누를 때 생기는 우클릭 원형 애니메이션(Press-and-Hold Ring)과 플릭 제스처는 0.3~0.5초의 지연과 획 끊김을 초래함.
   - 윈도우 창 생성 및 핸들 바인딩 시 `MicrosoftTabletPenServiceProperty` 속성을 적용하여 `TABLET_DISABLE_PRESSANDHOLD | TABLET_DISABLE_PENTAPFEEDBACK | TABLET_DISABLE_FLICKS` 활성화.
   - 환경설정 `pen_disable_gestures: bool` (기본: `true`)과 연동.

2. **A2: 한 프레임의 펜 점 모두 받기 (Sub-frame Coalesced Pen Events)**:
   - 고주파(133 Hz ~ 240 Hz) 펜 타블렛 샘플링 시 60 Hz 화면 주사율 사이에서 발생하는 중간 점들의 유실을 방지.
   - `crates/ui-egui/src/canvas.rs`의 드래그 루프에서 `ui.input(|i| &i.events)`에 누적된 `PointerMoved` 및 `Touch` 이벤트를 순회하여, 모든 중간 좌표와 압력(pressure)을 보존한 상태로 도구에 순차 디스패치.
   - 빠른 펜 스트로크에서도 선이 각지게 꺾이지 않고 매끄러운 곡선 유지.

3. **A3: 탭 허용치 3 pt · 닫기 허용 반경 8 px**:
   - `pen_tap_tolerance: f64` = 3.0 pt: 손떨림으로 인한 미세 드래그를 방지하고 정확한 단일 앵커 포인트 생성.
   - `pen_close_radius: f64` = 8.0 px: 유구 면, 토층 경계 등 다각형/패스 폐합 시 시작점 부근 8 px 이내에서 자동으로 스냅 닫기.

4. **A4: 더블탭은 시스템 설정 연동**:
   - OS 기본 더블클릭/더블탭 시간 및 거리 임계값 준수.

5. **A5: 캔버스 단추 줄 (Canvas Button Row for Tablet)**:
   - 키보드 접근이 어려운 액정/판 타블렛 작업자를 위한 캔버스 플로팅 퀵 버튼 바.
   - 실행 취소(Undo), 다시 실행(Redo), 손 도구 토글(Hand/Pan), 줌 인(+), 줌 아웃(-), 대지 맞춤(Fit), 펜 감도 설정(Tablet Settings).
   - Quiet Plate 디자인 토큰 적용 및 환경설정 `canvas_button_row` (기본: `true`) 연동.

6. **A6: 펜 시험 칸 및 타블렛 감도 설정 대화상자**:
   - 대화상자 / 카드: `와콤/XP-Pen 펜 타블렛 감도 설정`.
   - 인식된 타블렛 장치 정보 표시.
   - 필압 감도 곡선: `부드러움 (Soft)` ($p = f^{0.7}$) | `보통 (Normal, 표준 고고학 제도)` ($p = f$) | `단단함 (Firm, 균일 선)` ($p = f^{1.4}$).
   - 정밀 탭 허용치(3 pt), 닫기 허용치(8 px), 손바닥 거르기(Palm Rejection) 토글.
   - **대화형 펜 시험 칸 (Interactive Pen Test Canvas)**:
     - 대화상자 내에서 직접 스타일러스로 선을 그어보며 실시간 필압 게이지와 굵기 변화를 체감.
     - 시험선 지우기(Clear) 기능 제공.

7. **A7: 컨트롤 바 펜 필압 모드 (`ctl-pressure`)**:
   - `crates/ui-egui/src/trowel/control.rs`의 `segments_soon`을 실제 동작하는 세그먼트 버튼으로 전환.
   - `Same Weight` (균일 굵기, 기본): 고고학 실측 규격(0.8 pt, 0.5 pt, 0.2 pt 등)을 필압에 무관하게 엄격 유지.
   - `Follow Pressure` (필압 따라): 필압 감도 곡선에 따라 스트로크 두께 가변.

8. **A8: 안내 가이드**:
   - 타블렛 사용 팁 및 고고학 제도 가이드라인 툴팁/안내문 제공.

## 검증 계획

1. 환경설정 엔진 커맨드(`prefs.set` / `prefs.get`)의 펜 관련 7종 키 단위 테스트.
2. 캔버스 단추줄 인터랙션 및 커맨드 실행 단위 테스트.
3. 펜 감도 곡선 변환 수치 계산 단위 테스트.
4. 타블렛 설정 대화상자 및 펜 시험 칸 UI 렌더링 테스트.
5. 컨트롤 바 `ctl-pressure` 토글 연동 테스트.
6. `cargo xtask ci` 7개 전체 품질 게이트 통과.
