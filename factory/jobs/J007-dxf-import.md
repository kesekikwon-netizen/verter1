# J007 — 좌표를 지키는 DXF 가져오기 및 desca 단면 정합 (IMAGE 읽기, CP949, desca 레이어 짝)

상태: done (2026-10-14) · 커밋 대상 · 검증: `cargo xtask ci` 7개 전체 품질 관문 통과 (fmt, clippy, test, assets, brands, layers, wasm) · **고고학 CAD DXF 임포터 및 desca 단면 체계**

## 목표

사용자 지시, 백로그 항목 8, 및 한국 매장유산 발굴조사 CAD 실무 표준에 따라:
기존 DXF 가져오기가 도면을 대지 중앙으로 강제 이동시켜 TM 좌표를 유실하던 문제를 해결하고, 한국 발굴조사 표준 토층단면 프로그램인 `desca` DXF 파일의 레이어·표고·이미지·한글 인코딩(CP949)을 완벽하게 수용하여 Trowel 도면으로 정합하는 시스템을 구현한다.

1. **좌표 보존 및 실제 좌표계 연동 (`center: false`, `preserve_coord: true`)**:
   - 기존 CAD 임포터는 `ImportOptions::default()`에서 `center: true`로 설정되어 중심점을 임의 오프셋으로 이동시켜 실제 국가 TM 및 단면 표고 좌표를 소실함.
   - `preserve_coord: bool` 옵션 추가:
     - `preserve_coord: true` 또는 고고학 도면 모드 시 원본 도면의 실세계 좌표계를 유지.
     - 도면의 실제 바운딩 박스를 기준으로 적절한 도곽/대지 크기를 설정하고, 원본 원점을 `DocCoord`(J006)에 등록하여 종이 pt와 현장 실측 좌표(m)가 정확히 일치하도록 연결.

2. **한국어 인코딩(CP949 / EUC-KR) 자동 복원**:
   - 한국 발굴조사 AutoCAD 및 desca에서 출력된 DXF는 대부분 Windows 한국어 코드페이지 `CP949 (ANSI_949)`로 작성됨.
   - `$DWGCODEPAGE` 헤더가 `ANSI_949`, `EUC-KR`, `949`인 경우 또는 UTF-8이 아닌 한글 바이트열이 포함된 경우, `encoding_rs::EUC_KR`을 통해 완전한 11,172자 한글로 디코딩하여 레이어명과 텍스트의 깨짐(mojibake)을 원천 방지.

3. **desca 단면 레이어 자동 짝맞춤 (Layer Mapping)**:
   - `SECTION_PROFILE` (또는 `단면선`): Trowel 표준 `단면선 (잘린 땅)` (0.8 pt 실선)으로 매핑.
   - `LEVEL_*` (예: `LEVEL_50`, `LEVEL_10`, `LEVEL_LINE` 등 desca 해발선 원본):
     - 숨긴 템플릿 레이어인 `desca 원본` (template: true, visible: false)으로 보존.
     - 동시에 J015 해발선 자동 생성기(`drawing.makeLevelLines`)를 연계하여 깨끗한 50 cm / 10 cm 표준 해발선으로 재구성.
   - `LEVEL_TEXT` (표고 숫자):
     - `해발 숫자` 레이어로 매핑.
     - 텍스트 값 끝에 단위가 없으면 `"m"`을 자동 부가 (예: `"152.5"` → `"152.5m"`).
   - 기타 단면 레이어:
     - `STONE`, `ROCK`, `보이는 돌` → `보이는 돌·유물` (0.5 pt)
     - `CUT_STONE`, `잘린 돌` → `잘린 돌·유물·기와` (0.8 pt)
     - `STRATA`, `토층선` → `토층선` (0.5 pt)
     - `DISTURB`, `교란` → `교란` (0.5 pt)

4. **DXF `IMAGE` 및 `IMAGEDEF` 엔티티 읽기**:
   - AutoCAD DXF의 래스터 이미지(`IMAGE` 엔티티 및 `IMAGEDEF` 객체) 파싱:
     - 삽입점(10, 20), U/V 벡터(11/21, 12/22), 픽셀 크기(13, 23).
     - `IMAGEDEF` 파일 경로/파일명(1)을 읽어 `NodeKind::Image(ImageObject)` 생성.
     - 현장 단면 사진 또는 스캔 밑그림이 도면 내 정확한 위치에 래스터 객체로 배치되도록 지원.

5. **품질 검증**:
   - CP949 인코딩 DXF 디코딩 단위 테스트 (`crates/cad/src/import/reader.rs`).
   - `IMAGE` 및 `IMAGEDEF` 엔티티 파싱 및 `ImageObject` 생성 테스트.
   - `desca` 단면 DXF 레이어 매핑 및 좌표 보존 테스트.
   - `cargo xtask ci` 7개 전체 품질 관문 통과.

## 검증 결과 및 증거 (Evidence)

1. **단위 및 통합 테스트**:
   - `cargo test -p vectorcraft-cad`: 50개 테스트 전체 통과 (0 failed).
     - `import::reader::tests::euc_kr_cp949_korean_decoding`: 통과.
     - `import::tests::preserve_coord_and_desca_section_mapping`: 통과.
     - `import::tests::image_entity_parsing`: 통과.
   - `cargo test -p vectorcraft-engine -- open_with_preserve_coord_and_desca_section`: 통과.
   - `cargo test -p vectorcraft-ui-egui -- trowel::`: 77개 테스트 전체 통과.

2. **품질 게이트 (cargo xtask ci)**:
   - Step 1: cargo fmt --all -- --check (Pass)
   - Step 2: cargo clippy --workspace --all-targets -- -D warnings (Pass, 0 warnings)
   - Step 3: cargo test --workspace (Pass, 856 tests all ok)
   - Step 4: cargo xtask assets (Pass)
   - Step 5: cargo xtask brands (Pass)
   - Step 6: cargo xtask layers (Pass)
   - Step 7: cargo xtask wasm (Pass)
   - 종합 결과: **ci passed in 53s**

