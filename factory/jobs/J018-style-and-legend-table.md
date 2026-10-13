# J018: 양식 파일과 범례표 저절로 (내 기본값, .trowel-style 내보내기·받기, 쓴 줄만 범례표, 사용자 굵기 추가)

상태: spec → build · 기준 커밋 `05f75563` · 검증: `cargo test` 및 `cargo xtask ci` 전체 관문 통과 목표

## 1. 개요 및 배경
발굴조사 도면 표준화와 보고서 출판의 편의를 위해 양식 파일(`*.trowel-style`), 사용자 맞춤 기본값(내 기본값), 사용자 정의 선 굵기 동적 추가, 그리고 도면에 실제로 객체가 그려진 레이어만 선별하여 도판에 자동 배치하는 "쓴 줄만 범례표" 기능을 구현한다.

---

## 2. 세부 기능 요구사항

### 2.1 C1: 양식 파일 규격 (`.trowel-style`) 및 내보내기/가져오기 (`style.*`)
1. **파일 형식**:
   - JSON 기반의 자체 완비형 양식 규격 (`"format": "trowel-style"`, `"version": 1`).
   - 포함 정보: 양식명(`name`), 설명(`description`), 기본 축척(`scale`), 굵기 사다리(`weights`), 레이어 목록(`layers`: 이름, 선 굵기, 선종, 대시 배열, 빗금 종류, 색상 RGB, 즐겨찾기 여부, 고고학 묶음).
2. **엔진 명령**:
   - `style.exportTrowelStyle`: 현재 열린 도면의 레이어 및 범례 설정을 `.trowel-style` JSON 구조로 직렬화하여 반환하거나 파일로 저장.
   - `style.importTrowelStyle`: `.trowel-style` 데이터를 읽어 현재 도면과 비교(Diff 분석):
     - `preview_only: true`: 새로 추가될 레이어(`added`), 굵기/선종이 변경될 레이어(`modified`), 동일한 레이어(`unchanged`)를 사전 비교하여 보고.
     - `apply: true`: 비교 결과를 바탕으로 신규 레이어 생성 및 기존 레이어 범례 속성 일괄 동기화.

### 2.2 C2: 「내 기본값」 (My Defaults) 환경설정 연동
1. **환경설정 저장**:
   - `Prefs`에 `trowel_default_style: String` (JSON 직렬화된 사용자 기본 레이어 양식) 추가.
2. **엔진 명령**:
   - `style.setDefaults`: 현재 도면의 레이어 설정을 사용자의 글로벌 "내 기본값"으로 영구 저장.
   - `style.applyDefaults`: 현재 작업 중인 도면에 "내 기본값" 레이어 세트를 즉시 적용.
3. **UI 연동**:
   - 줄 고치기 카드(`rowedit.rs`)의 `saved_in` 영역에서 「내 기본값 (`defaults`)」 단추를 활성화하여 현재 레이어를 기본값으로 즉시 동기화.

### 2.3 C3: 그리기 팔레트 사용자 굵기 추가 (`+` 단추)
1. **동적 굵기 지원**:
   - 기본 굵기 사다리(`0.1`, `0.2`, `0.3`, `0.5`, `0.8`, `1.0`) 외에 현장 실측자가 자주 쓰는 굵기(예: `0.15`, `0.4`, `0.7`, `1.2`, `1.5`)를 동적으로 팔레트에 등록.
2. **환경설정**:
   - `Prefs`에 `trowel_custom_weights: Vec<f64>` 추가 (`trowelCustomWeights`).
3. **팔레트 UI (`palette.rs`)**:
   - `weight-add` `+` 단추를 활성화하여 클릭 시 다음 추천 굵기를 사다리에 즉시 추가하거나 커스텀 굵기를 토글 선택 가능하도록 구현.

### 2.4 C4: 범례표 저절로 (쓴 줄만 / Automatic Legend Table with Used Layers Only)
1. **동작 원리**:
   - 도판 출판 시 20~30개의 템플릿 레이어 중 실제로 유구선이나 토층선이 그려진 레이어만 자동 판별.
   - `sheet.generateLegendTable`에 `used_only: bool` (기본값: `true`) 매개변수 적용.
   - `used_only == true`인 경우, 도면 레이어 중 하위 자식 노드가 1개 이상 존재하는 레이어(`children().len() > 0`)만 필터링하여 범례표 행으로 작성.
   - 각 범례표 행에 해당 레이어의 실제 모양(선 굵기, 색상, 실선/파선 대시, 빗금 견본)을 정밀하게 벡터 그래픽으로 렌더링.

---

## 3. 검증 기준
1. `cargo fmt --all -- --check` 통과.
2. `cargo clippy --workspace --all-targets -- -D warnings` 통과.
3. `cargo test -p vectorcraft-engine` 양식 파일 직렬화/역직렬화 및 비교(Diff) 테스트 통과.
4. `cargo test -p vectorcraft-ui-egui trowel::` 전체 테스트 통과 (J018 통합 테스트 포함).
5. `cargo test -p vectorcraft-ui-egui i18n` 전체 37개 테스트 통과.
6. `cargo xtask ci` 7개 전체 관문 통과.
7. Git 커밋, 원격 푸시, GitHub Actions CI 확인.
