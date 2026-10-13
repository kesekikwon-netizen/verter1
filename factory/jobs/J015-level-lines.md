# J015 — 해발선 만들기 (빈 곳에만, 50cm 0.5pt, 10cm 0.2pt, 「152.5m」 왼쪽 끝, 단면선 수정 시 재절단)

상태: done (2026-10-14) · 기준 커밋 `80cccb63` · 검증: `cargo xtask ci` 7개 품질 게이트 통과 (fmt, clippy, test, assets, brands, layers, wasm) · **고고학 CAD 단면 해발선 자동 생성**

## 목표

사용자 지시, 백로그 항목 7, 및 청사진(`docs/design/trowel-design-blueprint.html` 단면·토층 도면 화면 사양)에 따라:
한국 매장유산 발굴조사 표준 토층단면도(Section Profile)의 기준 표고선 및 50 cm / 10 cm 해발선(Elevation level contours)을 단면선과 토층·유구 면을 제외한 빈 공간(배경 및 토층 공백)에 자동으로 제도하는 엔진 및 UI 체계를 구현한다.

1. **규격 및 도면 레이어 체계**:
   - `해발선 50 cm`: 0.5 pt 실선 (주선, LegendGroup::Section 기본 레이어).
   - `해발선 10 cm`: 0.2 pt 실선 (보조선, LegendGroup::Section 기본 레이어).
   - `해발 숫자`: 굵기 없음 기호/텍스트 (`해발 숫자` 레이어).
     - 50 cm 주선의 왼쪽 끝에 `152.5m` 형식(EL 표고 소수점 1자리 m 표기)으로 배치.
   - 대상 레이어가 문서에 없으면 자동으로 Section 범례 속성(`group: Some(LegendGroup::Section)`)으로 생성.

2. **표고 계산 및 생성 범위 ("가장 낮은 곳부터 가장 높은 곳 위 첫 50 cm 선까지")**:
   - `doc.coord`의 축척(`scale`) 및 원점 표고(`origin_y`, `elevation_datum`)와 연동.
   - 단면선(`단면선 (잘린 땅)`) 및 단면 유구의 경계 상자에서 최저점(`y_max`, paper coords = lowest elevation)과 최고점(`y_min`, paper coords = highest elevation) 산출.
   - 시작 표고(`min_elev`): 가장 낮은 곳을 10 cm (0.1 m) 단위로 내림(floor).
   - 종료 표고(`max_elev`): 가장 높은 곳 위의 첫 50 cm (0.5 m) 배수로 올림(ceil).
   - 0.1 m (10 cm) 간격으로 수평선을 전개:
     - 0.5 m (50 cm) 배수: 0.5 pt 선 + 좌측 끝 `152.5m` 레이블.
     - 그 외: 0.2 pt 보조선.

3. **장애물 영역 제외 ("빈 곳에만: 땅·잘린 것·보이는 돌·토층 면 안 제외")**:
   - 장애물 대상 레이어: `단면선 (잘린 땅)`, `잘린 돌·유물·기와`, `보이는 돌·유물`, `교란`, `생토`, `토층선` 등 지중/토층/석재 폐합 객체.
   - 수평선 $y$와 장애물 폐합 곡선 간의 교차 구간($[x_{in}, x_{out}]$)을 계산하고 합집합을 구한 뒤, 수평 전체 구간 $[x_{start}, x_{end}]$에서 장애물 구간을 차감(subtract).
   - 오직 빈 공간에만 선분을 생성하여 토층 내부나 돌을 침범하지 않음.

4. **멱등성 및 재실행 ("단면선을 고치면 다시 자름")**:
   - `drawing.makeLevelLines` 실행 시 이전 생성된 `해발선 50 cm`, `해발선 10 cm`, `해발 숫자` 레이어 내의 기존 해발선 객체를 깨끗이 비우고 새로 계산하여 삽입.
   - 단면선이나 토층을 수정 후 다시 실행하면 즉시 재절단/재배치 완료. 단일 Undo 지원.

5. **엔진 명령 및 UI 연동**:
   - 엔진 명령: `drawing.makeLevelLines`
     - 파라미터: `{ base_elevation?: number, interval_cm?: number, min_elevation?: number, max_elevation?: number, left_x?: number, right_x?: number }`
     - 반환값: `{ lines_50cm: number, lines_10cm: number, labels: number, min_elevation: number, max_elevation: number }`
   - UI:
     - 메뉴 `도면` ▸ `해발선 만들기` 활성화 (Item::LevelLines).
     - 실행 후 상태줄 안내: `해발선 생성 완료: 50 cm N개, 10 cm M개 (EL. X.X m ~ Y.Y m)`.
     - 10개 언어 번역 동기화.

6. **품질 검증**:
   - `vectorcraft-geom` 수평선-장애물 차감 알고리즘 단위 테스트.
   - `vectorcraft-engine` 해발선 생성, 레이어 배치, 치수 및 라벨 검증 테스트.
   - `vectorcraft-ui-egui` 메뉴 활성화 및 실행 테스트.
   - `cargo xtask ci` 7개 전체 관문 통과.
