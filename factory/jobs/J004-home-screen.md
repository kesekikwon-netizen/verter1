# J004 — 홈 화면과 새 도면 견본

상태: done (2026-10-14) · 기준 커밋 `b63240dc` · 검증: `cargo xtask ci` 7개 품질 게이트 전원 통과 (fmt, clippy -D warnings, test, assets, brands, layers, wasm) · 75/75 trowel UI 테스트 및 legend 단위 테스트 통과

## 목표

사용자 지시 및 디자인 2.1 2절, 청사진(`docs/design/trowel-design-blueprint.html` 1. 홈 화면)에 따라, 한국 매장유산 발굴도면 전문 CAD Trowel의 홈 화면과 4대 표준 새 도면 견본(평면도 1:20, 단면·토층도 1:20, 유물 실측도 1:1, 보고서 종합도판 A3)을 구현한다.

- `crates/ui-egui/src/trowel/home.rs` 신설: Strata hgis의 조용한 도판(Quiet Plate) 디자인 토큰을 100% 준수하는 홈 화면 대시보드 구현.
  - **상단 연동 바**: 조사 연동(광령리1 Strata GPKG), 작업 좌표계(EPSG:5186 중부원점), 도면집 경로, 연결 상태.
  - **브랜드 히어로**: Trowel 타이틀, 고고학 전문 CAD 설명, 「새 도면 만들기 (⌘N)」 점토색(Clay/Action) 단추, 「도면집 열기 (⌘O)」 보조 단추.
  - **이어서 작업 카드**: 최근 작업 도면명 원클릭 「이어서 열기 →」, 메타데이터 5열(조사명, 종류, 축척, 좌표계, 작업일).
  - **최근 도면집 테이블**: 도면명, 조사명, 종류, 축척, 좌표계, 상태, 마지막 수정 컬럼 및 검색창. 행 클릭 시 도면 즉시 로드.
  - **도면 작성 3단계 순서 레일**: ① 현장 실측 & 밑그림 정합(완료) → ② CAD 실측 제도(다음/진행중) → ③ 도판 배치 & 보고서 출력(예정).
  - **새 도면 견본 4종 카드**: 평면도(1:20), 단면·토층도(1:20), 유물 실측도(1:1), 보고서 종합도판(A3). 클릭 시 해당 규격의 신규 도면 생성 및 레이어/범례 세팅.
  - **하단 발굴 검수 상태줄**: Strata GIS 연결, EPSG:5186, 타블렛 감지, Noto Serif/Sans 글꼴 적용, 국가유산 발굴조사 유의사항 v1 통과 표시.
- `crates/engine/src/cmd/drawing.rs` (또는 `legend.rs`): `drawing.newTemplate` 명령 구현 (`{ kind: "plan" | "section" | "finds" | "sheet", name?: string }`).
  - 평면도(plan): 1:20 축척, EPSG:5186, Plan 범례 15개 레이어 세팅.
  - 단면·토층도(section): 1:20 축척, Section 범례 12개 레이어 세팅.
  - 유물 실측도(finds): 1:1 축척, Finds 범례 7개 레이어 세팅.
  - 보고서 종합도판(sheet): A3 가로 규격, 도판 도곽/표제란/방위축척 레이어 세팅.
- 리본(`ribbon.rs`)의 새 도면 묶음 타일 4종(평면, 단면, 유물, 도판) 활성화 및 연결.
- `canvas.rs`의 `home()`에서 `app.ui.trowel`일 때 `crate::trowel::home::show` 호출로 연결.

## 완료 기준

1. **엔진 명령 `drawing.newTemplate`.**
   - 인자 `{ kind: "plan" | "section" | "finds" | "sheet", name?: string }`.
   - 각 종류별 적합한 도면 크기 및 전용 범례/레이어 구성 생성.
   - 단일 실행 취소(Undo) 지원.
   - 단위 테스트 작성 (`crates/engine/src/cmd/fileio/` 또는 `legend.rs` 내).

2. **홈 화면 UI (`crates/ui-egui/src/trowel/home.rs`).**
   - Quiet Plate 색상 토큰 체계(`Tokens`) 엄격 준수 (Ground, Wash, Surface, Oat, Ink, Hand, Clay).
   - 상단 연동 바, 히어로 영역, 이어서 작업 카드, 최근 도면집 목록 테이블, 3단계 가이드 레일, 4종 견본 카드, 하단 상태줄 배치.
   - 최근 작업 도면 목록(`recent_files`)이 있을 경우 해당 파일 목록을 바인딩하고, 없거나 초기 상태일 때는 표준 견본 도면 목록 표시.
   - 각 카드 및 행 클릭 시 해당 도면 열기 또는 템플릿 생성 실행.

3. **리본 연동 (`crates/ui-egui/src/trowel/ribbon.rs`).**
   - "새 도면" 그룹 내 `plan`, `section`, `finds`, `sheet` 타일 모두 활성화되어 각각의 템플릿 생성 명령을 실행.

4. **다국어 및 번역.**
   - 신규 `tw!("...")` 문자열에 대해 `crates/ui-egui/src/i18n/ko.tsv` 한국어 번역 추가.
   - 신규 메뉴/명령 라벨에 대해 10개 언어 카탈로그 검증 통과.

5. **품질 검증.**
   - `cargo fmt --all -- --check`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test -p vectorcraft-ui-egui trowel::`
   - `cargo xtask ci` 통과.
