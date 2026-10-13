#!/usr/bin/env python3
"""Build docs/design/trowel-design-blueprint.html — Trowel's finished-target design board.

Complete architectural and interface blueprint of the finished Trowel application:
1. 홈 화면 (Home Screen & Dashboard, J004)
2. 평면 도면 화면 (Plan CAD View, J003 / J005 / J006 / J009 / J014)
3. 단면·토층 도면 화면 (Section & Soil Stratum View, J010 / J014 / J015)
4. 유물 실측 도면 화면 (Artifact Drafting View, J011)
5. 보고서 종합 도판 화면 (Plate & Sheet Layout View, J012)
6. 주요 대화상자 시스템 (Core Dialogs: 축척/TM좌표계 J006, 밑그림 정합 J008, DXF 임포트 J007, 펜 설정 J016)
7. 디자인 시스템 규격서 (Quiet Plate System Tokens & Rules from Strata hgis)
8. 지금 앱 대조 (Current Running App Screenshot)
9. 공장 개발 순서 (Factory Backlog J001 ~ J019)

Strict adherence to Strata's binding design 「조용한 도판」 (Quiet Plate) from hgis.
"""

import base64
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ICONS = ROOT / "assets" / "icons"
OUT = ROOT / "docs" / "design" / "trowel-design-blueprint.html"


def ic(name: str, size: int = 24) -> str:
    path = ICONS / f"{name}.svg"
    if not path.exists():
        return f'<svg width="{size}" height="{size}"><rect width="{size}" height="{size}" fill="none" stroke="#141413"/></svg>'
    svg = path.read_text(encoding="utf-8").strip()
    return svg.replace('width="24" height="24"', f'width="{size}" height="{size}"', 1)


T = {
    "ground": "#FAF9F5", "surface": "#FFFFFF", "wash": "#F5F4ED", "desk": "#F0EEE6",
    "press": "#E8E6DC", "oat": "#E3DACC", "hand": "#30302E", "ink": "#141413",
    "ink2": "#3D3D3A", "muted": "#5E5D59", "faint": "#73726C", "line": "#DEDCD1",
    "edge": "#C2C0B6", "ring": "#B0AEA5", "action": "#B5573A", "clay": "#D97757",
    "clayText": "#9C4A2F", "block": "#A33B3B", "caution": "#7A5A00", "done": "#3F6B31",
}

PX = {1.0: 2.5, 0.8: 2.0, 0.5: 1.25, 0.3: 0.85, 0.2: 0.6, 0.1: 0.45}


def dash(kind, w=1.0):
    w = max(w, 0.8)
    return {
        "dashdot": f' stroke-dasharray="{6*w:.1f} {2.5*w:.1f} {0.9:.1f} {2.5*w:.1f}"',
        "dash": f' stroke-dasharray="{4*w:.1f} {2.5*w:.1f}"',
        "dot": f' stroke-dasharray="0.9 {2.4*w:.1f}"',
    }.get(kind, "")


def tile(icon, label, look):
    return f'<div class="tile {look}"><div class="well">{ic(icon, 22)}</div><span>{label}</span></div>'


# ------------------------------------------------------------------ RIBBONS
RIBBON_PLAN = [
    ("파일", [("tw-open", "열기", ""), ("tw-save", "저장", "save"), ("tw-import", "가져오기", "")]),
    ("새 도면", [("tw-plan", "평면", "on"), ("tw-section", "단면", ""), ("tw-finds", "유물", ""), ("tw-sheet", "도판", "")]),
    ("밑그림", [("tw-align", "맞추기", ""), ("tw-underlay", "밑그림", "shown")]),
    ("그리기", [("tw-pen", "펜", "on"), ("tw-pencil", "연필", ""), ("tw-section-line", "단면선", ""), ("tw-spot-height", "표고", ""), ("tw-bucket", "칠통", "")]),
    ("도면 요소", [("tw-north", "방위", ""), ("tw-scale-bar", "축척", ""), ("tw-legend", "범례표", ""), ("tw-title-block", "표제란", "")]),
    ("보기", [("tw-grid", "격자", ""), ("tw-magnet", "자석", "shown")]),
    ("내보내기", [("tw-pdf", "PDF", ""), ("tw-dxf", "DXF", ""), ("tw-strata", "Strata로", "")]),
]

RIBBON_SECTION = [
    ("파일", [("tw-open", "열기", ""), ("tw-save", "저장", "save"), ("tw-import", "가져오기", "")]),
    ("새 도면", [("tw-plan", "평면", ""), ("tw-section", "단면", "on"), ("tw-finds", "유물", ""), ("tw-sheet", "도판", "")]),
    ("단면 도구", [("tw-section-line", "단면선", ""), ("tw-pen", "토층선", "on"), ("tw-spot-height", "해발선", "shown"), ("tw-hachure", "빗금", ""), ("tw-bucket", "칠통", "")]),
    ("도면 요소", [("tw-scale-bar", "표고눈금", "shown"), ("tw-legend", "토층표", "shown"), ("tw-title-block", "표제란", "")]),
    ("보기", [("tw-grid", "수평격자", "shown"), ("tw-magnet", "자석", "shown")]),
    ("내보내기", [("tw-pdf", "PDF", ""), ("tw-dxf", "DXF", ""), ("tw-strata", "Strata로", "")]),
]

RIBBON_ARTIFACT = [
    ("파일", [("tw-open", "열기", ""), ("tw-save", "저장", "save"), ("tw-import", "실측사진", "")]),
    ("새 도면", [("tw-plan", "평면", ""), ("tw-section", "단면", ""), ("tw-finds", "유물", "on"), ("tw-sheet", "도판", "")]),
    ("유물 실측", [("tw-line", "중심선", "shown"), ("tw-pen", "윤곽선", "on"), ("tw-pencil", "문양선", ""), ("tw-hachure", "단면해치", ""), ("tw-dropper", "태토", "")]),
    ("측정 도구", [("tw-rotate", "구경측정", "shown"), ("tw-scale", "저경측정", ""), ("tw-scale-bar", "1:1축척자", "shown")]),
    ("보기", [("tw-grid", "1cm모눈", "shown"), ("tw-magnet", "정밀스냅", "shown")]),
    ("내보내기", [("tw-pdf", "PDF", ""), ("tw-dxf", "DXF", ""), ("tw-sheet", "도판으로", "")]),
]

RIBBON_SHEET = [
    ("파일", [("tw-open", "열기", ""), ("tw-save", "저장", "save"), ("tw-import", "도면배치", "")]),
    ("새 도면", [("tw-plan", "평면", ""), ("tw-section", "단면", ""), ("tw-finds", "유물", ""), ("tw-sheet", "도판", "on")]),
    ("도판 구성", [("tw-north", "도북방위", "shown"), ("tw-scale-bar", "복합축척자", "shown"), ("tw-legend", "자동범례표", "shown"), ("tw-title-block", "정규표제란", "shown")]),
    ("용지 규격", [("tw-rect", "A3 가로", "shown"), ("tw-align", "도면정렬", ""), ("tw-grid", "여백격자", "")]),
    ("출판 내보내기", [("tw-pdf", "PDF출판", "on"), ("tw-dxf", "CAD내보내기", ""), ("tw-strata", "Strata제출", "")]),
]


def render_ribbon(ribbon_spec):
    return "".join(
        f'<div class="rgroup"><div class="rcap">{cap}</div><div class="tiles">{"".join(tile(*t) for t in tiles)}</div></div>'
        for cap, tiles in ribbon_spec
    )


# ------------------------------------------------------------------ TOOLS
TOOLS_DATA = [
    [("tw-select", "V"), ("tw-direct", "A")],
    [("tw-pen", "P"), ("tw-pencil", "N"), ("tw-line", "\\"), ("tw-rect", "M"), ("tw-type", "T")],
    [("tw-rotate", "R"), ("tw-scale", "S"), ("tw-scissors", "C")],
    [("tw-bucket", "K"), ("tw-dropper", "I")],
    [("tw-hand", "H"), ("tw-zoom", "Z")],
]
ARCH_DATA = ["tw-section-line", "tw-spot-height", "tw-find-number", "tw-posthole", "tw-hachure"]


def render_tools(active_icon="tw-pen"):
    out = []
    for i, group in enumerate(TOOLS_DATA):
        for icon, key in group:
            out.append(f'<div class="tool{" on" if icon == active_icon else ""}">{ic(icon, 19)}<b>{key}</b></div>')
        out.append('<div class="tsep"></div>' if i < len(TOOLS_DATA) - 1 else "")
    out.append('<div class="tcap">고고학</div>')
    out += [f'<div class="tool{" on" if icon == active_icon else ""}">{ic(icon, 19)}</div>' for icon in ARCH_DATA]
    out.append('<div class="proxy"><i class="fb"></i><i class="sb"></i></div>')
    return "".join(out)


# ------------------------------------------------------------------ LEGEND ROWS
EYE = '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/></svg>'
OPEN = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" opacity=".35"><rect x="5" y="11" width="14" height="10" rx="1"/><path d="M8 11V7a4 4 0 0 1 7.5-2"/></svg>'
SHUT = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="5" y="11" width="14" height="10" rx="1"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg>'

STRATA_ROWS = [
    ("굴광선 (유구 면)", 0.8, None, "poly", True, 6, "sel"),
    ("단면 위치선", 0.2, "dashdot", "line", True, 2, "apart"),
    ("조사구역 경계", 1.0, "dashdot", "poly", False, 1, ""),
    ("시굴격자", 0.5, None, "poly", False, 6, ""),
    ("유구 선 (구·벽구)", 0.5, None, "line", False, 1, ""),
    ("기준점", None, "△", "mark", False, 4, ""),
    ("유물 출토 위치", 0.2, "③", "mark", False, 4, ""),
]
TROWEL_ROWS = [
    ("바닥선", 0.5, None, "poly", True, 4, ""),
    ("내부 시설", 0.5, None, "poly", False, 5, ""),
    ("돌", 0.5, None, "poly", False, 9, ""),
    ("경사 표시", 0.2, None, "poly", False, 3, ""),
    ("표고점", None, "·152", "mark", False, 7, ""),
    ("격자 · 도곽", 0.1, None, "line", False, 1, ""),
    ("글자 · 번호", None, "가", "mark", False, 12, ""),
    ("밑그림", None, None, "under", False, 1, "locked"),
]

SECTION_ROWS = [
    ("단면 굴광선 (외곽)", 0.8, None, "line", True, 1, "sel"),
    ("잘린 면 빗금 (45°)", 0.1, "hatch", "mark", True, 1, ""),
    ("기준 표고선 (EL. 152.5)", 0.5, "dash", "line", True, 1, ""),
    ("해발 보조선 (10cm)", 0.2, "dot", "line", False, 8, ""),
    ("토층 분할선", 0.3, None, "line", True, 4, ""),
    ("소토 및 탄화면", 0.2, None, "poly", False, 2, ""),
    ("토층 번호", None, "①", "mark", False, 4, ""),
    ("토층 해설표", None, "표", "mark", False, 1, ""),
]

ARTIFACT_ROWS = [
    ("유물 중심선 (℄)", 0.2, "dashdot", "line", True, 1, ""),
    ("외형 윤곽선", 0.5, None, "line", True, 2, "sel"),
    ("단면 잘린면 (태토)", 0.8, None, "poly", True, 2, ""),
    ("태토 빗금 해칭", 0.1, "hatch", "mark", True, 1, ""),
    ("문양 및 침선", 0.2, None, "line", False, 6, ""),
    ("구경 측정 가이드", 0.2, "dot", "line", False, 1, ""),
    ("치수선 및 수치", 0.2, None, "line", False, 3, ""),
    ("1:1 실측 축척자", 0.5, None, "line", False, 1, ""),
]


def look(weight, d, kind):
    ink = T["ink"]
    if kind == "mark":
        return f'<span class="mk">{d}</span>'
    if kind == "under":
        return f'<svg width="22" height="12"><rect x="1.5" y="1.5" width="19" height="9" fill="{T["wash"]}" stroke="{T["edge"]}"/><path d="M4 9l4-4 3 3 3-3 4 4" fill="none" stroke="{T["faint"]}"/></svg>'
    w = PX.get(weight, 1.0) if weight else 1
    if kind == "poly":
        return f'<svg width="22" height="12"><rect x="{1+w/2:.2f}" y="{1+w/2:.2f}" width="{20-w:.2f}" height="{10-w:.2f}" fill="none" stroke="{ink}" stroke-width="{w}"{dash(d, w)}/></svg>'
    return f'<svg width="22" height="12"><line x1="1" y1="6" x2="21" y2="6" stroke="{ink}" stroke-width="{w}"{dash(d, w)}/></svg>'


def rows(data):
    out = []
    for name, weight, d, kind, star, count, flag in data:
        cls = "lrow" + (" sel" if flag == "sel" else "") + (" fav" if star else "")
        out.append(
            f'<div class="{cls}"><span class="tw">▸</span><span class="ico">{EYE}</span>'
            f'<span class="ico">{SHUT if flag == "locked" else OPEN}</span><span class="look">{look(weight, d, kind)}</span>'
            f'<span class="star">{"★" if star else ""}</span><span class="ln">{name}</span>'
            f'{"<span class=apart>따로 1</span>" if flag == "apart" else ""}'
            f'<span class="lw">{f"{weight:.1f}" if weight is not None else ""}</span><span class="lc">{count}</span></div>'
        )
    return "".join(out)


def weight_chips(chosen=0.8):
    out = [f'<div class="chip{" chosen" if w == chosen else ""}"><i style="height:{max(PX[w],1):.2f}px"></i><span>{w:.1f}</span></div>'
           for w in (0.1, 0.2, 0.3, 0.5, 0.8, 1.0)]
    return "".join(out) + '<div class="chip plus">＋</div>'


def hatch_icon(kind):
    lines = ""
    for k in range(-2, 5):
        x = k * 5
        if kind in ("45", "alt"):
            lines += f'<line x1="{x}" y1="12" x2="{x+12}" y2="0"/>'
        if kind in ("135", "alt"):
            lines += f'<line x1="{x}" y1="0" x2="{x+12}" y2="12"/>'
    return (f'<svg width="16" height="12"><clipPath id="h{kind}"><rect x="1" y="1" width="14" height="10"/></clipPath>'
            f'<g clip-path="url(#h{kind})" stroke="{T["ink"]}" stroke-width=".7">{lines}</g>'
            f'<rect x="1" y="1" width="14" height="10" fill="none" stroke="{T["ink"]}"/></svg>')


def hatch_chips(chosen="45"):
    spec = [("45", "45°", chosen == "45"), ("135", "135°", chosen == "135"), ("alt", "번갈아", chosen == "alt"), ("none", "없음", chosen == "none")]
    return "".join(f'<div class="hchip{" chosen" if c else ""}">{hatch_icon(k) if k != "none" else ""}<span>{l}</span></div>' for k, l, c in spec)


# ------------------------------------------------------------------ DRAWINGS
def plan_svg():
    ink = T["ink"]
    g = []
    for i, x in enumerate(range(60, 721, 66)):
        g.append(f'<line x1="{x}" y1="40" x2="{x}" y2="520" stroke="{ink}" stroke-width="{PX[0.1]}" opacity=".5"/>')
        g.append(f'<text x="{x}" y="33" class="m9" text-anchor="middle">{147980 + i*2:,}</text>')
    for i, y in enumerate(range(40, 521, 60)):
        g.append(f'<line x1="60" y1="{y}" x2="720" y2="{y}" stroke="{ink}" stroke-width="{PX[0.1]}" opacity=".5"/>')
        g.append(f'<text x="54" y="{y+3}" class="m9" text-anchor="end">{98170 - i*2:,}</text>')
    g.append(f'<rect x="78" y="58" width="624" height="444" fill="none" stroke="{ink}" stroke-width="{PX[1.0]}"{dash("dashdot", 2.5)}/>')
    for k in range(9):
        y = 172 + k * 24
        g.append(f'<line x1="{213 - (k % 2) * 4}" y1="{y}" x2="238" y2="{y + 4}" stroke="{ink}" stroke-width="{PX[0.2]}"/>')
    g.append(f'<polygon points="210,150 548,132 590,318 556,452 236,470 196,300" fill="none" stroke="{ink}" stroke-width="{PX[0.8]}" stroke-linejoin="round"/>')
    g.append(f'<polygon points="246,184 520,170 556,318 528,424 262,438 232,302" fill="none" stroke="{ink}" stroke-width="{PX[0.5]}" stroke-linejoin="round"/>')
    for cx, cy, n in ((290, 226, "P1"), (480, 216, "P2"), (488, 380, "P3"), (300, 392, "P4")):
        g.append(f'<circle cx="{cx}" cy="{cy}" r="13" fill="none" stroke="{ink}" stroke-width="{PX[0.5]}"/><text x="{cx+16}" y="{cy-11}" class="m9">{n}</text>')
    g.append(f'<ellipse cx="392" cy="318" rx="40" ry="28" fill="none" stroke="{ink}" stroke-width="{PX[0.5]}"/><text x="392" y="336" class="s10" text-anchor="middle">노지</text>')
    for sx, sy in ((618, 200), (642, 238), (614, 280), (128, 404), (150, 436)):
        g.append(f'<path d="M{sx} {sy} l10 -6 l12 3 l3 11 l-9 8 l-13 -2 z" fill="none" stroke="{ink}" stroke-width="{PX[0.5]}"/>')
    g.append(f'<line x1="150" y1="300" x2="650" y2="300" stroke="{ink}" stroke-width="{PX[0.2]}"{dash("dashdot")}/>')
    g.append(f'<path d="M150 294v12M650 294v12" stroke="{ink}"/><text x="140" y="290" class="s12">A</text><text x="652" y="290" class="s12">A′</text>')
    for px, py, n in ((96, 78, "GCP1"), (684, 78, "GCP2"), (684, 486, "GCP3"), (96, 486, "GCP4")):
        g.append(f'<path d="M{px} {py-6} l6 10 h-12 z" fill="none" stroke="{ink}"/><text x="{px+9}" y="{py+3}" class="m9">{n}</text>')
    for fx, fy, n in ((350, 250, "1"), (436, 372, "2"), (330, 346, "3"), (452, 262, "4")):
        g.append(f'<circle cx="{fx}" cy="{fy}" r="6" fill="#fff" stroke="{ink}" stroke-width="{PX[0.2]}"/><text x="{fx}" y="{fy+3}" class="m8" text-anchor="middle">{n}</text>')
    for ex, ey, v in ((222, 140, "152.36"), (560, 124, "152.41"), (604, 330, "152.18"), (420, 282, "151.74")):
        g.append(f'<circle cx="{ex}" cy="{ey}" r="1.6" fill="{ink}"/><text x="{ex+4}" y="{ey-3}" class="m9">{v}</text>')
    for i, (vx, vy) in enumerate(((210, 150), (548, 132), (590, 318), (556, 452), (236, 470), (196, 300))):
        g.append(f'<rect x="{vx-3.5}" y="{vy-3.5}" width="7" height="7" fill="{T["action"] if i == 2 else "#fff"}" stroke="{ink}"/>')
    g.append(f'<polyline points="612,384 664,372 676,426" fill="none" stroke="{ink}" stroke-width="{PX[0.8]}"/>')
    g.append(f'<line x1="676" y1="426" x2="642" y2="470" stroke="{T["action"]}" stroke-dasharray="3 3"/>')
    g.append(f'<rect x="609" y="381" width="6" height="6" fill="#fff" stroke="{ink}"/><rect x="661" y="369" width="6" height="6" fill="#fff" stroke="{ink}"/><rect x="673" y="423" width="6" height="6" fill="{ink}"/>')
    # Scale bar & North
    g.append(f'<g transform="translate(60 540)"><rect width="66" height="5" fill="{ink}"/><rect x="66" width="66" height="5" fill="#fff" stroke="{ink}"/><rect x="132" width="66" height="5" fill="{ink}"/>'
             f'<text y="-4" class="m9">0</text><text x="62" y="-4" class="m9">2</text><text x="128" y="-4" class="m9">4</text><text x="190" y="-4" class="m9">6 m</text><text x="214" y="6" class="s10">축척 1 : 20</text></g>')
    g.append(f'<g transform="translate(742 500)"><path d="M0 -26 l7 22 -7 -5 -7 5 z" fill="{ink}"/><text y="12" class="s10" text-anchor="middle">도북</text></g>')
    return "".join(g)


def section_svg():
    ink = T["ink"]
    clay = T["clay"]
    g = []
    # Elevation axes
    for i, el in enumerate(("153.5", "153.0", "152.5", "152.0", "151.5")):
        y = 60 + i * 55
        g.append(f'<line x1="70" y1="{y}" x2="720" y2="{y}" stroke="{ink}" stroke-width="{PX[0.1]}" opacity=".4" stroke-dasharray="2 4"/>')
        g.append(f'<text x="64" y="{y+4}" class="m9" text-anchor="end">EL. {el} m</text>')
    # Distance ticks
    for k in range(6):
        x = 90 + k * 120
        g.append(f'<line x1="{x}" y1="340" x2="{x}" y2="348" stroke="{ink}" stroke-width="{PX[0.5]}"/>')
        g.append(f'<text x="{x}" y="362" class="m9" text-anchor="middle">{k}.0 m</text>')
    g.append(f'<line x1="90" y1="340" x2="690" y2="340" stroke="{ink}" stroke-width="{PX[0.5]}"/>')
    # Base Reference line (EL. 152.50)
    g.append(f'<line x1="80" y1="170" x2="710" y2="170" stroke="#C46686" stroke-width="1.5" stroke-dasharray="6 3"/>')
    g.append(f'<text x="82" y="164" fill="#C46686" font-size="11px" font-weight="700">기준선 EL. 152.50 m</text>')

    # Excavated profile cut polygon
    profile_pts = "100,90 140,110 210,135 240,230 360,248 480,245 580,225 610,120 670,95"
    # Cut-face hatch pattern definition inside
    g.append('<defs><pattern id="cutHatch" width="8" height="8" patternTransform="rotate(45 0 0)" patternUnits="userSpaceOnUse"><line x1="0" y1="0" x2="0" y2="8" stroke="#141413" stroke-width="0.8" /></pattern></defs>')
    # Fill trench cut boundary with 0.8pt and pattern
    g.append(f'<path d="M100,90 L140,110 L210,135 L240,230 L360,248 L480,245 L580,225 L610,120 L670,95 L670,290 L100,290 Z" fill="url(#cutHatch)" opacity="0.15"/>')
    g.append(f'<path d="M100,90 L140,110 L210,135 L240,230 L360,248 L480,245 L580,225 L610,120 L670,95" fill="none" stroke="{ink}" stroke-width="{PX[0.8]}" stroke-linejoin="round"/>')

    # Stratum division lines (0.3pt)
    g.append(f'<path d="M105,115 Q220,150 360,160 T665,118" fill="none" stroke="{ink}" stroke-width="{PX[0.3]}"/>')
    g.append(f'<path d="M140,145 Q260,185 420,190 T630,140" fill="none" stroke="{ink}" stroke-width="{PX[0.3]}"/>')
    g.append(f'<path d="M225,205 Q350,222 470,220 T590,195" fill="none" stroke="{ink}" stroke-width="{PX[0.3]}"/>')

    # Hearth (노지) pit cross section
    g.append(f'<path d="M370,240 Q410,258 450,238" fill="#F3DEDA" stroke="{clay}" stroke-width="1.2"/>')
    g.append(f'<text x="410" y="246" font-size="9px" fill="{clay}" text-anchor="middle">소토면</text>')

    # Soil Layer Labels ①, ②, ③, ④
    for cx, cy, num, label in ((200, 115, "①", "표토"), (320, 160, "②", "퇴적토"), (360, 205, "③", "주거면"), (420, 270, "④", "생토층")):
        g.append(f'<circle cx="{cx}" cy="{cy}" r="9" fill="#FFFFFF" stroke="{ink}" stroke-width="1"/>')
        g.append(f'<text x="{cx}" y="{cy+4}" class="m9" text-anchor="middle" font-weight="700">{num}</text>')
        g.append(f'<text x="{cx+14}" y="{cy+3}" class="s10">{label}</text>')

    # Table of strata (삼선표) at bottom
    g.append(f'<g transform="translate(70, 380)">'
             f'<rect width="640" height="130" fill="#FAF9F5" stroke="{T["line"]}"/>'
             f'<line x1="0" y1="0" x2="640" y2="0" stroke="{ink}" stroke-width="1.5"/>'
             f'<line x1="0" y1="24" x2="640" y2="24" stroke="{T["line"]}" stroke-width="1"/>'
             f'<text x="12" y="16" font-size="11px" font-weight="700" fill="{ink}">층위</text>'
             f'<text x="60" y="16" font-size="11px" font-weight="700" fill="{ink}">토색 (Munsell)</text>'
             f'<text x="180" y="16" font-size="11px" font-weight="700" fill="{ink}">토성</text>'
             f'<text x="280" y="16" font-size="11px" font-weight="700" fill="{ink}">포함유물 및 퇴적 특징</text>'
             f'<text x="12" y="44" class="m9">①</text><text x="60" y="44" class="s10">10YR 4/3 갈색</text><text x="180" y="44" class="s10">사질점토</text><text x="280" y="44" class="s10">표토 및 현대 식생 부식토, 소량의 자기편</text>'
             f'<text x="12" y="68" class="m9">②</text><text x="60" y="68" class="s10">10YR 3/2 암갈색</text><text x="180" y="68" class="s10">점토질모래</text><text x="280" y="68" class="s10">암적갈색 소토립 다량 함유, 무문토기 구연부편</text>'
             f'<text x="12" y="92" class="m9">③</text><text x="60" y="92" class="s10">10YR 2/2 흑갈색</text><text x="180" y="92" class="s10">점토</text><text x="280" y="92" class="s10">주거지 바닥 다짐토, 목탄편 집중, 갈판 완형 출토</text>'
             f'<text x="12" y="116" class="m9">④</text><text x="60" y="116" class="s10">7.5YR 5/6 명황갈색</text><text x="180" y="116" class="s10">점토</text><text x="280" y="116" class="s10">기반 풍화암반 잔류 점토층 (무유구 기반층)</text>'
             f'<line x1="0" y1="130" x2="640" y2="130" stroke="{ink}" stroke-width="1.5"/>'
             f'</g>')
    return "".join(g)


def artifact_svg():
    ink = T["ink"]
    g = []
    # 1cm grid
    for x in range(80, 701, 30):
        g.append(f'<line x1="{x}" y1="40" x2="{x}" y2="480" stroke="{T["line"]}" stroke-width="0.5"/>')
    for y in range(40, 481, 30):
        g.append(f'<line x1="80" y1="{y}" x2="700" y2="{y}" stroke="{T["line"]}" stroke-width="0.5"/>')

    # Centerline
    g.append(f'<line x1="390" y1="30" x2="390" y2="460" stroke="{ink}" stroke-width="{PX[0.2]}" stroke-dasharray="14 3 2 3"/>')
    g.append(f'<text x="390" y="24" class="m9" text-anchor="middle">℄ 중심선</text>')

    # Vessel Rim arc guide (Diameter 24.6 cm -> radius ~ 150px)
    g.append(f'<path d="M240,80 A150,150 0 0,1 540,80" fill="none" stroke="{T["faint"]}" stroke-dasharray="2 3"/>')
    g.append(f'<line x1="240" y1="80" x2="540" y2="80" stroke="{ink}" stroke-width="{PX[0.5]}"/>')
    g.append(f'<text x="390" y="74" class="m9" text-anchor="middle">구경 24.6 cm</text>')

    # Left: Outer contour line (0.5pt) + decoration lines (0.2pt)
    g.append(f'<path d="M240,80 C210,130 190,220 220,330 C235,385 270,410 320,415 L390,415" fill="none" stroke="{ink}" stroke-width="{PX[0.5]}" stroke-linejoin="round"/>')
    for dy in range(120, 240, 16):
        g.append(f'<line x1="228" y1="{dy}" x2="275" y2="{dy-15}" stroke="{ink}" stroke-width="{PX[0.2]}"/>')
        g.append(f'<line x1="234" y1="{dy+8}" x2="280" y2="{dy-7}" stroke="{ink}" stroke-width="{PX[0.2]}"/>')
    g.append(f'<text x="200" y="260" class="s10" text-anchor="end">즐목문(빗살) 선화</text>')

    # Right: Vessel wall section (0.8pt outer + 0.1pt diagonal hatching for clay fabric)
    g.append('<defs><pattern id="pasteHatch" width="5" height="5" patternTransform="rotate(45 0 0)" patternUnits="userSpaceOnUse"><line x1="0" y1="0" x2="0" y2="5" stroke="#141413" stroke-width="0.7"/></pattern></defs>')
    g.append(f'<path d="M390,80 L540,80 C570,130 590,220 560,330 C545,385 510,410 460,415 L390,415 L390,403 L450,403 C495,398 528,375 540,325 C568,220 548,135 522,92 L390,92 Z" fill="url(#pasteHatch)" stroke="{ink}" stroke-width="{PX[0.8]}" stroke-linejoin="round"/>')
    g.append(f'<text x="580" y="260" class="s10">기벽 단면 (0.8pt + 0.1pt 빗금)</text>')

    # Bottom Base guide (Diameter 9.0cm)
    g.append(f'<line x1="320" y1="415" x2="460" y2="415" stroke="{ink}" stroke-width="{PX[0.5]}"/>')
    g.append(f'<text x="390" y="432" class="m9" text-anchor="middle">저경 9.0 cm</text>')

    # Height dimension line (18.2 cm)
    g.append(f'<line x1="160" y1="80" x2="160" y2="415" stroke="{ink}" stroke-width="{PX[0.2]}"/>')
    g.append(f'<line x1="154" y1="80" x2="166" y2="80" stroke="{ink}" stroke-width="{PX[0.2]}"/>')
    g.append(f'<line x1="154" y1="415" x2="166" y2="415" stroke="{ink}" stroke-width="{PX[0.2]}"/>')
    g.append(f'<text x="146" y="250" class="m9" text-anchor="end">기고 18.2 cm</text>')

    # 1:1 Scale Bar
    g.append(f'<g transform="translate(240, 455)">'
             f'<rect width="30" height="6" fill="{ink}"/>'
             f'<rect x="30" width="30" height="6" fill="#fff" stroke="{ink}"/>'
             f'<rect x="60" width="30" height="6" fill="{ink}"/>'
             f'<rect x="90" width="30" height="6" fill="#fff" stroke="{ink}"/>'
             f'<rect x="120" width="30" height="6" fill="{ink}"/>'
             f'<text x="0" y="16" class="m9">0</text><text x="30" y="16" class="m9">2</text><text x="60" y="16" class="m9">4</text><text x="90" y="16" class="m9">6</text><text x="120" y="16" class="m9">8</text><text x="150" y="16" class="m9">10 cm</text>'
             f'<text x="180" y="16" class="s10">실측 축척 1 : 1</text></g>')
    return "".join(g)


def sheet_svg():
    ink = T["ink"]
    g = []
    # A3 Sheet boundary (420 x 297 mm representation)
    g.append(f'<rect x="20" y="20" width="740" height="520" fill="#FFFFFF" stroke="{ink}" stroke-width="1.5"/>')
    g.append(f'<rect x="35" y="35" width="710" height="490" fill="none" stroke="{ink}" stroke-width="0.8"/>')

    # Plate title
    g.append(f'<text x="390" y="62" font-family="var(--serif)" font-size="16px" font-weight="700" text-anchor="middle" fill="{ink}">【도판 3】 1호 주거지 평면 및 단면 실측도</text>')

    # Block 1: Plan (Left, 1:40)
    g.append(f'<rect x="50" y="80" width="400" height="340" fill="#FAF9F5" stroke="{T["line"]}"/>')
    g.append(f'<text x="60" y="100" class="s10" font-weight="700">1호 주거지 평면도 (축척 1 : 40)</text>')
    g.append(f'<g transform="translate(60, 100) scale(0.48)">{plan_svg()}</g>')

    # Block 2: Section (Right Top, 1:20)
    g.append(f'<rect x="465" y="80" width="265" height="190" fill="#FAF9F5" stroke="{T["line"]}"/>')
    g.append(f'<text x="475" y="98" class="s10" font-weight="700">1호 주거지 A–A′ 단면도 (축척 1 : 20)</text>')
    g.append(f'<g transform="translate(470, 85) scale(0.34)">{section_svg()}</g>')

    # Block 3: Artifact (Right Bottom, 1:2)
    g.append(f'<rect x="465" y="280" width="265" height="140" fill="#FAF9F5" stroke="{T["line"]}"/>')
    g.append(f'<text x="475" y="298" class="s10" font-weight="700">출토 토기 1호 실측도 (축척 1 : 2)</text>')
    g.append(f'<g transform="translate(470, 275) scale(0.33)">{artifact_svg()}</g>')

    # North arrow
    g.append(f'<g transform="translate(70, 460)">'
             f'<path d="M0 -22 l6 18 -6 -4 -6 18 z" fill="{ink}"/>'
             f'<text y="10" class="s10" text-anchor="middle">도북</text></g>')

    # Dual scale bar
    g.append(f'<g transform="translate(130, 460)">'
             f'<rect width="60" height="4" fill="{ink}"/><rect x="60" width="60" height="4" fill="#fff" stroke="{ink}"/>'
             f'<text x="0" y="-4" class="m9">0</text><text x="60" y="-4" class="m9">2</text><text x="120" y="-4" class="m9">4 m</text>'
             f'<text x="135" y="4" class="s10">축척 1 : 40 (평면)</text>'
             f'<rect y="16" width="30" height="4" fill="{ink}"/><rect x="30" y="16" width="30" height="4" fill="#fff" stroke="{ink}"/>'
             f'<text x="0" y="30" class="m9">0</text><text x="30" y="30" class="m9">1</text><text x="60" y="30" class="m9">2 m</text>'
             f'<text x="75" y="20" class="s10">축척 1 : 20 (단면)</text>'
             f'</g>')

    # Title block (우측 하단 표준 표제란, r0, 1px ink rule)
    g.append(f'<g transform="translate(435, 435)">'
             f'<rect width="295" height="75" fill="#FFFFFF" stroke="{ink}" stroke-width="1.2"/>'
             f'<line x1="0" y1="25" x2="295" y2="25" stroke="{ink}" stroke-width="1"/>'
             f'<line x1="0" y1="50" x2="295" y2="50" stroke="{ink}" stroke-width="1"/>'
             f'<line x1="60" y1="0" x2="60" y2="75" stroke="{ink}" stroke-width="1"/>'
             f'<line x1="200" y1="0" x2="200" y2="75" stroke="{ink}" stroke-width="1"/>'
             f'<text x="8" y="17" class="s10" font-weight="700">유적명</text><text x="68" y="17" class="s10">제주 광령리 유적</text>'
             f'<text x="206" y="17" class="s10" font-weight="700">도판번호</text><text x="250" y="17" class="m9" font-weight="700">PLATE 3</text>'
             f'<text x="8" y="42" class="s10" font-weight="700">도면명</text><text x="68" y="42" class="s10">1호 주거지 평면 및 단면도</text>'
             f'<text x="206" y="42" class="s10" font-weight="700">축척</text><text x="245" y="42" class="m9">1:20 · 1:40</text>'
             f'<text x="8" y="67" class="s10" font-weight="700">조사단</text><text x="68" y="67" class="s10">한국고고학연구원</text>'
             f'<text x="206" y="67" class="s10" font-weight="700">조사일</text><text x="242" y="67" class="m9">2026. 10.</text>'
             f'</g>')
    return "".join(g)


# ------------------------------------------------------------------ TABLES & DATA
def tokens_table():
    spec = [
        ("ground", "붙은 판 모두, 홈, 창 몸체", "panel · app_bar"), ("surface", "입력칸, 종이, 뜬 것", "input"),
        ("wash", "줄 위에 올림, 꺼진 칸, 눈금자", "panel_darker · ruler"), ("desk", "종이 뒤 책상, 리본 칸", "pasteboard · button"),
        ("press", "눌린 면, 표의 줄 선", "pressed · hover"), ("oat", "켜짐 · 보임 · 고름", "selection_fill · row_selected"),
        ("hand", "손에 든 도구", "tool_active · icon"), ("ink", "글자, 종이 테두리, 탭 밑줄", "text"),
        ("muted", "보조 글자, 이름표", "text_dim"), ("faint", "단위, 꺼진 글자", "text_disabled"),
        ("line", "판 사이 가는 선", "border · divider"), ("edge", "입력칸 · 보조 단추 테두리", "input_border"),
        ("ring", "켜진 귀리 칸의 테두리", "ring"), ("action", "한 화면에 하나: 저장 안 된 저장 칸 · 주 단추", "action"),
        ("clay", "6 px 점: 저장 안 됨 · 다음", "clay"), ("clayText", "자주 쓰는 줄 · 「다음」", "clay_text"),
        ("block", "막는 오류", "error"), ("caution", "주의", "caution"),
    ]
    return "".join(f'<tr><td><i class="sw" style="background:{T[k]}"></i></td><td class="mono">{k}</td><td class="mono r">{T[k]}</td><td>{r}</td><td class="mono">{f}</td></tr>' for k, r, f in spec)


def looks():
    spec = [("", "쉼", "tw-pencil", "연필"), ("on", "손에 든 도구 · hand", "tw-pen", "펜"),
            ("shown", "보임 · 켜짐 · oat + ring", "tw-magnet", "자석"), ("save", "저장 안 됨 · action, 하나뿐", "tw-save", "저장")]
    return "".join(f'<div class="lookcell">{tile(i, l, k)}<div class="cap">{c}</div></div>' for k, c, i, l in spec)


BACKLOG = [
    ("J001", "Trowel 이름 · 흙손 아이콘 · 조용한 도판 테마", "끝남"), ("J002", "줌 도구: 왼쪽으로 끌면 확대", "끝남"),
    ("J013", "깃허브 액션 CI (Linux & Windows)", "끝남"), ("J003", "메인 창 짜임 (1600 × 988 규격)", "끝남"), ("J005", "범례 = 레이어 (선종·굵기 연동)", "끝남"),
    ("J014", "그리기 팔레트와 빗금, 잘린 면", "다음"), ("J002b", "누르고 있는 동안만 Z 돋보기", ""),
    ("J004", "홈 화면과 새 도면 견본", ""), ("J006", "축척과 실제 좌표 (5186 · 5187)", ""), ("J015", "해발선 만들기", ""),
    ("J007", "좌표를 지키는 DXF 가져오기, desca 단면", ""), ("J016", "펜 1단계 (필압/탭 감도)", ""), ("J008", "밑그림 맞추기 (2점 · 4점 정합)", ""),
    ("J009", "평면 도구: 단면선 · 표고 · 유물 번호 · 주혈 · 경사", ""), ("J010", "단면 · 토층 화면", ""), ("J011", "유물 화면", ""),
    ("J012", "도판과 PDF: 방위 · 축척 막대 · 범례표 · 표제란", ""), ("J017", "펜 2단계 (옆버튼/손바닥 거르기)", ""), ("J018", "양식 파일과 범례표 자동 정리", ""),
    ("J019", "Strata 조사 파일(GPKG) 읽기", ""),
]


def backlog():
    def state(s):
        if s == "끝남":
            return '<span class="muted">끝남</span>'
        if s == "다음":
            return '<i class="cdot"></i>다음'
        return '<span class="faint">대기</span>'
    return "".join(f'<tr><td class="mono r">{i}</td><td class="mono">{j}</td><td>{t}</td><td>{state(s)}</td></tr>' for i, (j, t, s) in enumerate(BACKLOG, 1))


def page(shot: str) -> str:
    rep = {
        "{RIBBON_PLAN}": render_ribbon(RIBBON_PLAN),
        "{RIBBON_SECTION}": render_ribbon(RIBBON_SECTION),
        "{RIBBON_ARTIFACT}": render_ribbon(RIBBON_ARTIFACT),
        "{RIBBON_SHEET}": render_ribbon(RIBBON_SHEET),
        "{TOOLS_PLAN}": render_tools("tw-pen"),
        "{TOOLS_SECTION}": render_tools("tw-section-line"),
        "{TOOLS_ARTIFACT}": render_tools("tw-line"),
        "{STRATA}": rows(STRATA_ROWS),
        "{TROWEL}": rows(TROWEL_ROWS),
        "{SECTION_ROWS}": rows(SECTION_ROWS),
        "{ARTIFACT_ROWS}": rows(ARTIFACT_ROWS),
        "{WEIGHTS_PLAN}": weight_chips(0.8),
        "{WEIGHTS_SECTION}": weight_chips(0.3),
        "{WEIGHTS_ARTIFACT}": weight_chips(0.5),
        "{HATCH_PLAN}": hatch_chips("none"),
        "{HATCH_SECTION}": hatch_chips("45"),
        "{HATCH_ARTIFACT}": hatch_chips("45"),
        "{PLAN}": plan_svg(),
        "{SECTION}": section_svg(),
        "{ARTIFACT}": artifact_svg(),
        "{SHEET}": sheet_svg(),
        "{IC_PEN16}": ic("tw-pen", 16),
        "{IC_PLAN14}": ic("tw-plan", 14),
        "{IC_SECTION14}": ic("tw-section", 14),
        "{IC_FINDS14}": ic("tw-finds", 14),
        "{IC_SHEET14}": ic("tw-sheet", 14),
        "{TOKENS}": tokens_table(),
        "{LOOKS}": looks(),
        "{BACKLOG}": backlog(),
        "{SHOT}": shot,
    }
    html = HTML
    for k, v in rep.items():
        html = html.replace(k, v)
    return html


HTML = r"""<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="UTF-8">
<title>Trowel 완성 목표 전체 설계도 · 조용한 도판</title>
<style>
:root{--ground:#FAF9F5;--surface:#FFFFFF;--wash:#F5F4ED;--desk:#F0EEE6;--press:#E8E6DC;--oat:#E3DACC;--hand:#30302E;--ink:#141413;--ink2:#3D3D3A;--muted:#5E5D59;--faint:#73726C;--line:#DEDCD1;--edge:#C2C0B6;--ring:#B0AEA5;--action:#B5573A;--action-h:#A44D32;--clay:#D97757;--clay-text:#9C4A2F;--block:#A33B3B;--caution:#7A5A00;--caution-edge:#D9C27A;--caution-wash:#F7EFD6;
--serif:"Noto Serif KR","AppleMyungjo",Batang,serif;--sans:"Noto Sans KR","Apple SD Gothic Neo","Malgun Gothic",sans-serif;--mono:"JetBrains Mono",Consolas,"SF Mono",Menlo,monospace;--overlay:0 2px 12px rgba(20,20,19,.12)}
*{box-sizing:border-box;margin:0;padding:0}
body{background:var(--desk);color:var(--ink);font:400 13px/20px var(--sans);word-break:keep-all}
.mono{font-family:var(--mono)} .r{text-align:right} .muted{color:var(--muted)} .faint{color:var(--faint)}
.cdot{display:inline-block;width:6px;height:6px;border-radius:50%;background:var(--clay);margin-right:6px;vertical-align:1px}
/* viewer */
.vhead{background:var(--ground);border-bottom:1px solid var(--line);padding:24px 32px 0}
.vhead .top{display:flex;align-items:center;gap:16px}
.vhead h1{font:500 28px/36px var(--serif)}
.vhead .rule{width:1px;height:28px;background:var(--edge)}
.vhead .lead{font-size:15px;line-height:24px;color:var(--muted)}
.vhead p{color:var(--muted);font-size:12px;line-height:18px;margin-top:8px;max-width:1100px}
.vtabs{display:flex;margin-top:16px;height:38px;overflow-x:auto}
.vtab{padding:0 16px;display:flex;align-items:center;color:var(--muted);cursor:pointer;border-bottom:2px solid transparent;margin-bottom:-1px;white-space:nowrap;font-size:13px}
.vtab:hover{color:var(--ink)} .vtab.cur{color:var(--ink);border-bottom-color:var(--ink);font-weight:700}
.vpage{display:none;padding:32px} .vpage.cur{display:block}
.bcap{font-size:12px;line-height:18px;color:var(--muted);margin-top:12px;max-width:1600px}
.bcap b{color:var(--ink)}

/* ===== window 1600 x 988 ===== */
.win{width:1600px;height:988px;background:var(--ground);border:1px solid var(--edge);display:flex;flex-direction:column;overflow:hidden;position:relative;margin:0 auto}
.cap{height:32px;flex:none;display:flex;align-items:center;gap:8px;padding-left:76px;border-bottom:1px solid var(--line);font-size:13px;position:relative}
.cap .lights{position:absolute;left:14px;top:10px;display:flex;gap:8px}.cap .lights i{width:12px;height:12px;border-radius:50%;background:var(--edge)}
.cap .appicon{width:18px;height:18px;border-radius:4px;background:var(--ink);color:var(--ground);display:flex;align-items:center;justify-content:center}
.ribbon{height:96px;flex:none;display:flex;border-bottom:1px solid var(--line);padding:0 0 0 4px}
.rgroup{display:flex;flex-direction:column;padding:6px 10px 0;border-right:1px solid var(--line)}
.rcap{font-size:12px;line-height:16px;color:var(--ink2);margin-bottom:4px}
.tiles{display:flex;gap:11px}
.tile{width:36px;display:flex;flex-direction:column;align-items:center;gap:3px}
.tile .well{width:36px;height:36px;border-radius:10px;background:var(--desk);display:flex;align-items:center;justify-content:center;color:var(--hand);border:1px solid transparent;transition:background .12s ease-out}
.tile span{font-size:12px;line-height:16px;white-space:nowrap}
.tile:hover .well{background:var(--press)}
.tile.on .well{background:var(--hand);border-color:var(--hand);color:var(--ground)} .tile.on span{font-weight:700}
.tile.shown .well{background:var(--oat);border-color:var(--ring);color:var(--ink)} .tile.shown span{font-weight:700}
.tile.save .well{background:var(--action);color:#fff} .tile.save span{font-weight:700}
.find{margin:auto 8px auto auto;width:130px;height:26px;background:var(--surface);border:1px solid var(--edge);border-radius:8px;display:flex;align-items:center;gap:6px;padding:0 8px;color:var(--faint);font-size:12px}
.ctrl{height:40px;flex:none;display:flex;align-items:center;gap:8px;padding:0 10px;border-bottom:1px solid var(--line);font-size:12px}
.ctrl .lbl{color:var(--muted)}
.combo{height:24px;display:flex;align-items:center;gap:6px;padding:0 8px;background:var(--surface);border:1px solid var(--edge);border-radius:8px;white-space:nowrap}
.combo .chev{color:var(--ink2);font-size:9px;margin-left:auto}
.combo .m{font-family:var(--mono)}
.btn{height:24px;display:inline-flex;align-items:center;padding:0 8px;border:1px solid var(--edge);border-radius:8px;background:var(--surface);white-space:nowrap;cursor:pointer}
.btn.on{background:var(--oat);border-color:var(--ring)}
.seg{display:flex}.seg .btn{border-radius:0;margin-left:-1px}.seg .btn:first-child{border-radius:8px 0 0 8px;margin-left:0}.seg .btn:last-child{border-radius:0 8px 8px 0}
.seg .btn.on{background:var(--oat);border-color:var(--ink);z-index:1;position:relative}
.vr{width:1px;height:18px;background:var(--line);margin:0 4px}
.ctrl .hint{color:var(--muted)} .ctrl .right{margin-left:auto;display:flex;align-items:center;gap:8px}
.body{flex:1;display:flex;min-height:0}
.tcol{width:44px;flex:none;border-right:1px solid var(--line);display:flex;flex-direction:column;align-items:center;padding:6px 0;gap:3px}
.tool{width:34px;height:20px;border-radius:7px;display:flex;align-items:center;justify-content:center;color:var(--hand);position:relative}
.tool svg{width:16px;height:16px}
.tool.on{background:var(--hand);color:var(--ground)}
.tool b{position:absolute;right:-1px;bottom:-3px;font:400 7px/9px var(--mono);color:var(--faint)}
.tool.on b{color:var(--ring)}
.tsep{width:24px;height:1px;background:var(--line);margin:3px 0}
.tcap{font-size:9px;color:var(--muted);margin-top:4px}
.proxy{margin-top:auto;position:relative;width:28px;height:28px}
.proxy .fb{position:absolute;left:0;top:0;width:18px;height:18px;background:#fff;border:1px solid var(--ink)}
.proxy .sb{position:absolute;right:0;bottom:0;width:18px;height:18px;border:4px solid var(--ink)}
.set{width:172px;flex:none;border-right:1px solid var(--line);display:flex;flex-direction:column}
.ph{height:30px;display:flex;align-items:center;justify-content:space-between;padding:0 10px}
.q{height:22px;min-width:22px;padding:0 4px;display:inline-flex;align-items:center;justify-content:center;border-radius:7px;font-size:12px;cursor:pointer}
.sec{display:flex;align-items:center;gap:8px;font-size:11px;line-height:16px;color:var(--muted);padding:8px 10px 6px}
.sec::after{content:"";flex:1;height:1px;background:var(--line)}
.drow{height:26px;margin:0 6px;border-radius:6px;display:flex;align-items:center;gap:7px;padding:0 6px;font-size:12px;cursor:pointer}
.drow svg{color:var(--hand);flex:none}
.drow.cur{background:var(--oat)} .drow .d{margin-left:auto}
.setfoot{margin-top:auto;border-top:1px solid var(--line);padding:8px 10px;font-size:11px;line-height:17px;color:var(--muted)}
.setfoot .mono{color:var(--ink)}
.center{flex:1;display:flex;flex-direction:column;min-width:0}
.tabs{height:28px;flex:none;display:flex;border-bottom:1px solid var(--line);padding:0 8px;font-size:12px}
.tab{display:flex;align-items:center;padding:0 14px;color:var(--muted);border-bottom:2px solid transparent;margin-bottom:-1px;cursor:pointer}
.tab.cur{color:var(--ink);border-bottom-color:var(--ink);font-weight:700}
.tabs .corner{margin-left:auto;display:flex;align-items:center;padding-right:6px}
.canvas{flex:1;position:relative;background:var(--desk);overflow:hidden}
.rh{position:absolute;left:16px;right:0;top:0;height:16px;background:var(--wash);border-bottom:1px solid var(--line)}
.rv{position:absolute;left:0;top:16px;bottom:0;width:16px;background:var(--wash);border-right:1px solid var(--line)}
.rc{position:absolute;left:0;top:0;width:16px;height:16px;background:var(--wash);border:solid var(--line);border-width:0 1px 1px 0;font-size:10px;text-align:center;line-height:14px;color:var(--muted)}
.paper{position:absolute;left:92px;top:38px;width:684px;height:484px;background:#fff;border:1px solid var(--ink2)}
.paper-sheet{position:absolute;left:40px;top:20px;width:940px;height:520px;background:#fff;border:1px solid var(--ink2)}
.m9{font:400 9px var(--mono);fill:var(--muted)} .m8{font:400 8px var(--mono);fill:var(--ink)} .s10{font:400 10px var(--sans);fill:var(--ink)} .s12{font:400 12px var(--sans);fill:var(--ink)}
.ovl{position:absolute;background:var(--surface);border:1px solid var(--edge);border-radius:8px;box-shadow:var(--overlay)}
.guide{left:24px;top:24px;height:28px;display:flex;align-items:center;gap:8px;padding:0 10px;font-size:12px}
.guide .h{color:var(--ink2)}
.zoom{right:10px;bottom:46px;width:28px;display:flex;flex-direction:column}
.zoom div{height:28px;display:flex;align-items:center;justify-content:center;border-bottom:1px solid var(--line);font-size:14px;cursor:pointer}
.zoom div:last-child{border:0;font-size:10px}
.zpct{right:10px;bottom:10px;height:26px;padding:0 8px;display:flex;align-items:center;gap:4px;font-size:12px}
.zpct b{font:400 12px var(--mono)}
.rcol{width:244px;flex:none;border-left:1px solid var(--line);display:flex;flex-direction:column;min-height:0;font-size:12px}
.pal{padding:0 10px 10px;border-bottom:1px solid var(--line)}
.pal .ph{padding:0}
.ph .sub{color:var(--muted);font-size:11px;margin-left:6px}
.prow{display:flex;align-items:center;gap:4px;margin-top:6px}
.prow .k{width:26px;color:var(--muted);font-size:11px;flex:none}
.chip{width:24px;height:22px;border:1px solid var(--edge);border-radius:6px;background:#fff;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:2px}
.chip i{display:block;width:14px;background:var(--ink)}
.chip span{font:400 8px/9px var(--mono)}
.chip.chosen{background:var(--oat);border:2px solid var(--ink)} .chip.plus{color:var(--muted);border-color:var(--line)}
.hchip{height:22px;padding:0 5px;border:1px solid var(--edge);border-radius:6px;background:#fff;display:flex;align-items:center;gap:3px;font-size:10px}
.hchip.chosen{background:var(--oat);border:2px solid var(--ink)}
.dseg{display:flex}.dseg span{height:22px;padding:0 7px;border:1px solid var(--edge);margin-left:-1px;background:#fff;display:flex;align-items:center;font-size:11px}
.dseg span:first-child{border-radius:6px 0 0 6px;margin-left:0}.dseg span:last-child{border-radius:0 6px 6px 0}
.dseg span.on{background:var(--oat);border-color:var(--ink);position:relative;z-index:1}
.pbtn{display:flex;gap:4px;margin-top:8px}.pbtn .btn{height:22px;font-size:11px;padding:0 6px}
.leg{flex:1;display:flex;flex-direction:column;min-height:0;border-bottom:1px solid var(--line)}
.leg .ph{padding:0 6px 0 10px}
.lrows{flex:1;overflow:hidden}
.lrow{height:20px;display:flex;align-items:center;padding:0 6px;margin:0 4px;font-size:11px;border-radius:5px}
.lrow.sel{background:var(--oat)}
.lrow .tw{width:9px;color:var(--ink2);font-size:7px}
.lrow .ico{width:16px;display:flex;justify-content:center;color:var(--ink2)}
.lrow .look{width:26px;display:flex;justify-content:center}
.lrow .mk{font:400 10px var(--mono);width:22px;text-align:center}
.lrow .star{width:12px;color:var(--clay);font-size:10px;text-align:center}
.lrow .ln{flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.lrow.fav .ln{color:var(--clay-text);font-weight:600}
.lrow .apart{font-size:9px;line-height:12px;color:var(--caution);border:1px solid var(--caution-edge);background:var(--caution-wash);border-radius:4px;padding:0 3px;margin-right:4px}
.lrow .lw{width:22px;text-align:right;font:400 9px var(--mono)} .lrow .lc{width:16px;text-align:right;font:400 9px var(--mono);color:var(--muted)}
.lfoot{height:30px;display:flex;align-items:center;gap:4px;padding:0 8px;white-space:nowrap;border-top:1px solid var(--line);font-size:11px}
.lfoot .btn{height:20px;font-size:10px;padding:0 5px}
.props{padding:0 10px 10px;flex:none}
.props .ph{padding:0}
.frow{display:flex;align-items:center;gap:6px;margin-top:6px}
.frow .k{width:26px;color:var(--muted);font-size:11px}
.ledger{margin-top:8px;border-top:1px solid var(--ink);border-bottom:1px solid var(--line)}
.ledger .row{height:28px;display:flex;align-items:center}
.ledger .row .k{flex:1;font-size:11px;color:var(--muted)}
.ledger .row .v{font:400 16px var(--mono)} .ledger .row .u{width:16px;margin-left:4px;font-size:11px;color:var(--faint)}
.status{height:26px;flex:none;border-top:1px solid var(--line);display:flex;align-items:center;padding:0 10px;font-size:11px}
.status .say{flex:1;color:var(--ink2)}
.status .c{display:flex;align-items:center;gap:5px;padding:0 10px;border-left:1px solid var(--line);height:14px}
.status .m{font-family:var(--mono);display:inline-block;width:78px;text-align:right}
.status .k{color:var(--muted)}

/* ===== HOME SCREEN STYLES ===== */
.home-page{padding:32px;display:flex;flex-direction:column;gap:24px;overflow-y:auto;height:calc(988px - 32px)}
.home-hero{display:flex;gap:32px;padding:24px 0;border-top:1px solid var(--ink);border-bottom:1px solid var(--ink)}
.home-brand{flex:1;display:flex;flex-direction:column;gap:12px}
.home-brand h2{font:500 36px/42px var(--serif)}
.home-brand .sub{font-size:15px;color:var(--muted)}
.home-brand .lead{font-size:13px;line-height:22px;color:var(--ink2);margin-top:8px}
.home-brand .btn-row{display:flex;gap:12px;margin-top:14px}
.home-card{width:560px;background:#fff;border:1px solid var(--ink);border-radius:0;padding:20px;display:flex;flex-direction:column;gap:14px}
.home-card .head{display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid var(--line);padding-bottom:8px}
.home-card .head span{font-size:12px;color:var(--muted)}
.home-card .target{display:flex;justify-content:space-between;align-items:center}
.home-card .target h3{font:500 24px/32px var(--serif)}
.home-card .meta-row{display:grid;grid-template-columns:repeat(5, 1fr);gap:8px;border-top:1px solid var(--line);padding-top:12px}
.home-card .meta-col{display:flex;flex-direction:column;gap:2px}
.home-card .meta-col .k{font-size:11px;color:var(--muted)}
.home-card .meta-col .v{font:400 12px var(--mono)}
.home-layout{display:grid;grid-template-columns:1.4fr 1fr;gap:32px}
.home-sec-head{display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;font-size:15px;font-weight:700}
.rail{display:flex;flex-direction:column;gap:16px;position:relative;padding-left:24px;border-left:2px solid var(--edge);margin-left:8px}
.rail-stop{position:relative}
.rail-stop .dot{position:absolute;left:-31px;top:2px;width:12px;height:12px;border-radius:50%;background:#fff;border:2px solid var(--ink)}
.rail-stop.done .dot{background:var(--ink)}
.rail-stop.next .dot{background:var(--clay);border-color:var(--clay);width:14px;height:14px;left:-32px}
.rail-stop h4{font-size:14px;font-weight:700}
.rail-stop p{font-size:12px;color:var(--muted);line-height:18px;margin-top:2px}
.tmpl-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-top:12px}
.tmpl-card{background:#fff;border:1px solid var(--edge);border-radius:8px;padding:12px;display:flex;flex-direction:column;gap:6px;cursor:pointer}
.tmpl-card:hover{border-color:var(--ink)}
.tmpl-card h5{font-size:13px;font-weight:700}
.tmpl-card span{font-size:11px;color:var(--muted)}
.home-foot-status{border-top:1px solid var(--line);padding-top:14px;display:flex;gap:24px;font-size:12px;color:var(--muted)}

/* ===== DIALOGS STYLES ===== */
.dlg-grid{display:grid;grid-template-columns:1fr 1fr;gap:24px}
.dlg-box{background:var(--ground);border:1px solid var(--edge);border-radius:8px;padding:24px;display:flex;flex-direction:column;gap:16px;box-shadow:var(--overlay)}
.dlg-box h3{font:500 18px/24px var(--serif);border-bottom:1px solid var(--line);padding-bottom:10px}
.dlg-field{display:flex;align-items:center;gap:12px}
.dlg-field .k{width:110px;font-size:12px;color:var(--muted);flex:none}
.dlg-field .inp{flex:1;height:32px;background:#fff;border:1px solid var(--edge);border-radius:6px;padding:0 10px;display:flex;align-items:center;font-size:12px}
.long-ruler{height:48px;background:#fff;border:1px solid var(--edge);border-radius:6px;position:relative;display:flex;align-items:center;justify-content:space-between;padding:0 12px}
.long-ruler .mark{display:flex;flex-direction:column;align-items:center;gap:2px}
.long-ruler .mark i{width:1px;height:12px;background:var(--ink)}
.long-ruler .mark.major i{height:18px;background:var(--clay);width:2px}
.dlg-foot{display:flex;justify-content:flex-end;gap:8px;margin-top:10px}

/* weights card */
.card{position:absolute;right:246px;top:430px;width:300px;background:#fff;border:1px solid var(--edge);border-radius:8px;box-shadow:var(--overlay);padding:16px;display:none;z-index:5;font-size:12px}
.card.open{display:block}
.card .ct{display:flex;justify-content:space-between;align-items:center;margin-bottom:10px;font-size:13px}
.card .fr{display:flex;align-items:center;gap:8px;margin-bottom:8px}.card .fr .k{width:44px;color:var(--muted)}
.inp{height:28px;border:1px solid var(--edge);border-radius:8px;padding:0 10px;display:flex;align-items:center;justify-content:flex-end;font-family:var(--mono);flex:1}
table.t3{border-collapse:collapse;width:100%;border-top:1px solid var(--ink);border-bottom:1px solid var(--ink)}
table.t3 th{height:26px;font-size:11px;font-weight:400;color:var(--muted);text-align:left;border-bottom:1px solid var(--line);padding:0 6px}
table.t3 td{height:28px;border-top:1px solid var(--press);padding:0 6px}
table.t3 tr:nth-child(2) td{border-top:0}
.card .note{margin:8px 0 14px}
.card .foot{display:flex;justify-content:flex-end;gap:8px}
.b32{height:32px;padding:0 16px;border-radius:8px;border:1px solid var(--edge);background:#fff;font-size:13px;display:inline-flex;align-items:center;cursor:pointer}
.b32.primary{background:var(--action);border-color:var(--action);color:#fff;font-weight:700;min-width:88px;justify-content:center}
/* other boards */
.sheet{background:var(--ground);border:1px solid var(--edge);width:1600px;padding:32px;margin:0 auto}
.sheet h2{font:500 22px/30px var(--serif);margin-bottom:6px}
.sheet .lead{font-size:13px;line-height:20px;color:var(--muted);margin-bottom:20px;max-width:1000px}
.sheet table.t3 td{height:32px;font-size:13px;padding:0 8px} .sheet table.t3 th{padding:0 8px}
.sw{display:inline-block;width:64px;height:20px;border-radius:4px;border:1px solid var(--line);vertical-align:middle}
.cols{display:grid;grid-template-columns:1fr 1fr;gap:48px}
.looks{display:flex;gap:40px;margin:8px 0 24px}
.lookcell{display:flex;flex-direction:column;align-items:center;gap:8px}
.lookcell .cap{height:auto;border:0;padding:0;font-size:11px;color:var(--muted);text-align:center;width:130px;display:block}
.types div{display:flex;align-items:baseline;gap:16px;border-top:1px solid var(--press);padding:8px 0}.types div:first-child{border-top:0}
.types .k{width:130px;font-size:11px;color:var(--muted);flex:none}
.rules li{list-style:none;padding:8px 0;border-top:1px solid var(--press);font-size:13px;line-height:20px}.rules li:first-child{border-top:0}
.shot{width:1600px;border:1px solid var(--edge);display:block;margin:0 auto}
.pair-cap{display:flex;gap:8px;align-items:baseline;margin:0 0 8px;width:1600px;margin-left:auto;margin-right:auto}.pair-cap h3{font:400 15px/24px var(--sans)}
</style>
</head>
<body>
<header class="vhead">
  <div class="top"><h1>Trowel</h1><span class="rule"></span><span class="lead">한국 매장유산 발굴도면 · 유물 실측 CAD 완성 목표 설계도</span></div>
  <p>디자인 기준은 Strata(hgis)의 <b>「조용한 도판」 (Quiet Plate)</b> 기준서(DESIGN_SPEC.md)를 100% 동일하게 준수합니다. 하나의 상아색 대지(Ground), 먹색 선(Ink), 한 화면에 오직 하나의 흙색 채움(Clay action), 도면명과 유구명은 Serif(≥20px), 숫자는 정밀 고정폭 Mono를 적용합니다. 발굴 조사의 전 과정을 아우르는 5대 핵심 도면 화면과 주요 다이얼로그 시스템 전체를 포함합니다.</p>
  <nav class="vtabs">
    <div class="vtab cur" data-p="home">1. 홈 화면 (J004)</div>
    <div class="vtab" data-p="plan">2. 평면 도면 (J003/J005)</div>
    <div class="vtab" data-p="section">3. 단면·토층 (J010/J015)</div>
    <div class="vtab" data-p="artifact">4. 유물 실측 (J011)</div>
    <div class="vtab" data-p="sheet">5. 종합 도판 (J012)</div>
    <div class="vtab" data-p="dialogs">6. 주요 대화상자 (J006~J008)</div>
    <div class="vtab" data-p="system">7. 디자인 기준서</div>
    <div class="vtab" data-p="now">8. 지금 앱 대조</div>
    <div class="vtab" data-p="order">9. 공장 로드맵 (J001~J019)</div>
  </nav>
</header>

<!-- ========================= 1. HOME SCREEN ========================= -->
<section class="vpage cur" id="p-home">
<div class="win">
  <div class="cap"><span class="lights"><i></i><i></i><i></i></span><span class="appicon"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M6 15l9-11 3 3-11 9z"/><path d="M4 20h16"/></svg></span>Trowel - 한국 매장유산 발굴도면 전문 CAD</div>
  <div class="ctrl" style="background:var(--wash);border-bottom:1px solid var(--line)">
    <span class="lbl">조사 연동</span><span class="btn on">광령리1 (Strata GPKG)</span>
    <span class="lbl">좌표계</span><span class="combo"><span class="m">EPSG:5186</span> 중부원점</span>
    <span class="lbl">도면집 경로</span><span class="mono faint">바탕 화면 › 광령리1 › 도면</span>
    <span class="right"><span class="lbl">연결 상태</span><span class="btn on">정상 동기화</span></span>
  </div>
  <div class="home-page">
    <div class="home-hero">
      <div class="home-brand">
        <h2>Trowel</h2>
        <div class="sub">한국 매장유산 실측도면 · 유물 실측 전문 CAD</div>
        <div class="lead">오늘의 실측 도면을 새로 제도하거나, 발굴조사 도면집을 이어서 엽니다.<br>Strata(hgis) 현장 조사 데이터와 1:1로 결합하여 도면과 보고서 도판을 완성합니다.</div>
        <div class="btn-row">
          <span class="b32 primary">새 도면 만들기 (⌘N)</span>
          <span class="b32">도면집 열기 (⌘O)</span>
        </div>
      </div>
      <div class="home-card">
        <div class="head"><span>이어서 작업</span><span class="muted">저장됨</span></div>
        <div class="target">
          <h3>광령리1 · 1호 주거지 평면</h3>
          <span class="b32 primary">이어서 열기 →</span>
        </div>
        <div class="meta-row">
          <div class="meta-col"><span class="k">조사명</span><span class="v">광령리1</span></div>
          <div class="meta-col"><span class="k">종류</span><span class="v">평면도</span></div>
          <div class="meta-col"><span class="k">축척</span><span class="v">1:20</span></div>
          <div class="meta-col"><span class="k">좌표계</span><span class="v">EPSG:5186</span></div>
          <div class="meta-col"><span class="k">작업일</span><span class="v">오늘 07:04</span></div>
        </div>
      </div>
    </div>

    <div class="home-layout">
      <div>
        <div class="home-sec-head"><span>최근 도면집</span><span class="combo" style="width:200px">도면명·조사명 검색</span></div>
        <table class="t3">
          <tr><th>도면명</th><th>조사명</th><th>종류</th><th class="r">축척</th><th>좌표계</th><th>상태</th><th class="r">마지막 수정</th></tr>
          <tr><td><b>1호 주거지 평면</b></td><td>광령리1</td><td>평면도</td><td class="mono r">1:20</td><td class="mono">5186</td><td class="muted">저장됨</td><td class="mono r">오늘 07:04</td></tr>
          <tr><td><b>1호 주거지 A–A′ 단면</b></td><td>광령리1</td><td>단면·토층</td><td class="mono r">1:20</td><td class="mono">EL.152.5</td><td class="muted">저장됨</td><td class="mono r">오늘 06:45</td></tr>
          <tr><td><b>토기 1호 실측도</b></td><td>광령리1</td><td>유물 실측</td><td class="mono r">1:1</td><td class="mono">—</td><td class="muted">저장됨</td><td class="mono r">어제 18:30</td></tr>
          <tr><td><b>도판 3호 유구배치도</b></td><td>광령리1</td><td>종합 도판</td><td class="mono r">1:100</td><td class="mono">5186</td><td><i class="cdot"></i>저장 안 됨</td><td class="mono r">어제 16:15</td></tr>
          <tr><td><b>2호 주거지 평면</b></td><td>광령리1</td><td>평면도</td><td class="mono r">1:20</td><td class="mono">5186</td><td class="muted">저장됨</td><td class="mono r">10월 11일</td></tr>
          <tr><td><b>병산동 1호 주거지</b></td><td>병산동 시굴</td><td>평면도</td><td class="mono r">1:20</td><td class="mono">5186</td><td class="muted">저장됨</td><td class="mono r">10월 09일</td></tr>
        </table>
      </div>
      <div>
        <div class="home-sec-head"><span>도면 작성 3단계 순서</span><span class="muted" style="font-size:12px">광령리1 기준</span></div>
        <div class="rail">
          <div class="rail-stop done">
            <div class="dot"></div>
            <h4>① 현장 실측 & 밑그림 정합</h4>
            <p>스캔 방안지 실측도 2점·4점 정합, GPS GCP 기준점 좌표 매핑 (오차 0.82mm 검수 통과)</p>
          </div>
          <div class="rail-stop next">
            <div class="dot"></div>
            <h4>② CAD 실측 제도 <span style="color:var(--clay-text);font-size:12px">● 다음</span></h4>
            <p>0.1~1.0pt 규격 선 및 단면 빗금(잘린 면), 유구 굴광선·주혈·표고점 실측 배치 진행 중</p>
          </div>
          <div class="rail-stop">
            <div class="dot"></div>
            <h4>③ 도판 배치 & 보고서 출력</h4>
            <p>A3/A4 도판 대지 배치, 방위표, 복합 축척자, 정규 표제란, 국가유산청 규격 검수 및 출판</p>
          </div>
        </div>

        <div class="home-sec-head" style="margin-top:24px"><span>새 도면 견본 (템플릿 4종)</span></div>
        <div class="tmpl-grid">
          <div class="tmpl-card"><h5>평면도 견본 (1:20)</h5><span>도곽선, TM 좌표 격자, 유구/주혈/표고 범례</span></div>
          <div class="tmpl-card"><h5>단면·토층도 (1:20)</h5><span>해발 기준선(EL.), 10cm 보조선, 토층 해설표</span></div>
          <div class="tmpl-card"><h5>유물 실측도 (1:1)</h5><span>중심선, 대칭 단면, 3점 구경 측정 호, 1cm 모눈</span></div>
          <div class="tmpl-card"><h5>보고서 종합도판 (A3)</h5><span>공용 도북, 복합 축척 막대, 국가유산 정규 표제란</span></div>
        </div>
      </div>
    </div>

    <div class="home-foot-status">
      <span>Strata GIS (광령리1.gpkg) 연결됨</span>
      <span>작업 좌표계: EPSG:5186 중부원점 (정상)</span>
      <span>타블렛: Wacom Intuos Pro (인식됨)</span>
      <span>글꼴: Noto Serif KR / Noto Sans KR (적용됨)</span>
      <span style="margin-left:auto">국가유산 발굴조사 유의사항 v1 검수 통과</span>
    </div>
  </div>
</div>
<p class="bcap"><b>홈 화면 (J004)</b>: 발굴 도면 작성 작업의 시작점입니다. 최근 작업 도면을 원클릭으로 이어 열 수 있는 표제란, Strata 조사 폴더의 도면 파일 목록, 3단계 작업 순서 가이드 레일, 4대 표준 견본(평면, 단면, 유물, 도판)으로 구성됩니다.</p>
</section>

<!-- ========================= 2. PLAN SCREEN ========================= -->
<section class="vpage" id="p-plan">
<div class="win">
  <div class="cap"><span class="lights"><i></i><i></i><i></i></span><span class="appicon"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M6 15l9-11 3 3-11 9z"/><path d="M4 20h16"/></svg></span>1호 주거지 평면 * - Trowel</div>
  <div class="ribbon">{RIBBON_PLAN}<div class="find"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/></svg>명령 찾기</div></div>
  <div class="ctrl">
    <span class="lbl">펜 ›&nbsp; 범례</span>
    <span class="combo" style="width:210px"><svg width="18" height="10"><rect x="1" y="1" width="16" height="8" fill="none" stroke="#141413" stroke-width="2"/></svg>굴광선 (유구 면) <span class="m">0.8 pt</span><span class="chev">▼</span></span>
    <span class="btn">추정 (파선)</span><span class="vr"></span>
    <span class="lbl">닫으면</span><span class="combo">면적·둘레 계산<span class="chev">▼</span></span><span class="vr"></span>
    <span class="lbl">펜 필압</span><span class="seg"><span class="btn">굵기 그대로</span><span class="btn on">필압 따라</span></span><span class="vr"></span>
    <span class="hint">굵기 · 빗금 · 선종은 오른쪽 「그리기 팔레트」에서</span>
    <span class="right"><span class="btn on">자석</span><span class="lbl">허용치</span><span class="combo"><span class="m">2</span> px</span></span>
  </div>
  <div class="body">
    <aside class="tcol">{TOOLS_PLAN}</aside>
    <aside class="set">
      <div class="ph"><span>도면집</span><span class="q">＋</span></div>
      <div class="sec">열린 도면</div>
      <div class="drow cur">{IC_PLAN14}1호 주거지 평면<span class="d"><i class="cdot"></i></span></div>
      <div class="drow">{IC_SECTION14}1호 주거지 A–A′ 단면</div>
      <div class="drow">{IC_FINDS14}토기 1 실측</div>
      <div class="drow">{IC_SHEET14}도판 3 유구배치도</div>
      <div class="sec">광령리1 · 도면</div>
      <div class="drow">{IC_PLAN14}2호 주거지 평면</div>
      <div class="drow">{IC_SECTION14}Tr-3 북벽 토층</div>
      <div class="setfoot"><div>이 도면 · <span class="mono">420 × 297</span> mm</div><div>축척 <span class="mono">1:20</span></div><div>작업 5186 · 중부원점</div></div>
    </aside>
    <section class="center">
      <div class="tabs"><span class="tab">홈</span><span class="tab cur">1호 주거지 평면</span><span class="tab">1호 주거지 A–A′ 단면</span><span class="corner">1호 주거지 평면&nbsp;&nbsp;<i class="cdot"></i>저장 안 됨</span></div>
      <div class="canvas">
        <div class="rc">+</div>
        <div class="rh"><svg width="1000" height="16"><g stroke="#87867F"><path d="M76 16v-7M142 16v-7M208 16v-7M274 16v-7M340 16v-7M406 16v-7M472 16v-7M538 16v-7M604 16v-7M670 16v-7M736 16v-7"/></g><g class="m9"><text x="79" y="9" class="m9">0</text><text x="145" y="9" class="m9">2</text><text x="211" y="9" class="m9">4</text><text x="277" y="9" class="m9">6</text><text x="343" y="9" class="m9">8</text><text x="409" y="9" class="m9">10</text><text x="475" y="9" class="m9">12</text><text x="541" y="9" class="m9">14</text><text x="607" y="9" class="m9">16</text><text x="673" y="9" class="m9">18 m</text></g></svg></div>
        <div class="rv"></div>
        <div class="paper"><svg width="684" height="484" viewBox="0 0 780 560">{PLAN}</svg></div>
        <div class="ovl guide"><span style="display:flex;color:var(--hand)">{IC_PEN16}</span>펜 · 굴광선 (유구 면)<span class="vr"></span><span class="h">클릭 코너 점 추가</span></div>
        <div class="zoom ovl"><div>＋</div><div>－</div><div>맞춤</div></div>
        <div class="zpct ovl">화면 <b>68</b> %</div>
      </div>
    </section>
    <aside class="rcol">
      <div class="pal">
        <div class="ph"><span>그리기 팔레트<span class="sub">다음에 그릴 선 · pt</span></span><span class="q">⋯</span></div>
        <div class="prow"><span class="k">굵기</span>{WEIGHTS_PLAN}</div>
        <div class="prow"><span class="k">빗금</span>{HATCH_PLAN}</div>
        <div class="prow"><span class="k">선종</span><span class="dseg"><span class="on">실선</span><span>파선</span><span>점선</span><span>일점쇄선</span></span></div>
        <div class="pbtn"><span class="btn">잘린 면</span><span class="btn">범례에 더하기</span><span class="btn">범례대로</span></div>
      </div>
      <div class="leg">
        <div class="ph"><span>범례 · 레이어<span class="sub">평면용 ▾</span></span><span><span class="q">＋</span><span class="q">⋯</span></span></div>
        <div class="sec" style="padding-top:2px">조사 데이터 · Strata와 같은 레이어</div>
        <div class="lrows">{STRATA}<div class="sec">그리기 · Trowel</div>{TROWEL}</div>
        <div class="lfoot"><span class="muted">굵기 한 번에</span><span class="btn" id="ow">모든 굵기 ×0.8 …</span><span class="btn">따로 1 → 범례대로</span></div>
      </div>
      <div class="props">
        <div class="ph"><span>속성<span class="sub">1호 주거지</span></span><span class="muted">⋯&nbsp; ⌄</span></div>
        <div class="frow"><span class="k">종류</span><span class="combo" style="flex:1">주거지<span class="chev">▼</span></span><span class="combo"><span class="m">1</span>호</span></div>
        <div class="frow"><span class="k">시대</span><span class="combo" style="flex:1"><i style="width:14px;height:10px;background:rgba(122,87,55,.7);border:1px solid #141413;display:inline-block"></i>청동기<span class="chev">▼</span></span></div>
        <div class="ledger"><div class="row"><span class="k">면적</span><span class="v">32.91</span><span class="u">㎡</span></div><div class="row"><span class="k">둘레</span><span class="v">24.60</span><span class="u">m</span></div></div>
      </div>
    </aside>
  </div>
  <div class="status">
    <span class="say">클릭 코너 점 추가 · 드래그 부드러운 점 추가 · 시작점 클릭 닫기 · Enter 완료</span>
    <span class="c">펜</span><span class="c">자석 켬</span><span class="c"><i class="cdot"></i>저장 안 됨 1건</span>
    <span class="c"><span class="k">X</span><span class="m">147,999.305</span><span class="k">&nbsp;Y</span><span class="m">98,157.620</span></span>
    <span class="c"><span class="mono">1:20</span>&nbsp;· 작업 5186 · 중부원점</span>
  </div>
  <div class="card" id="wc">
    <div class="ct"><span>굵기 한 번에</span><span class="q" id="cw">✕</span></div>
    <div class="fr"><span class="k">어디</span><span class="seg"><span class="btn on">이 도면</span><span class="btn">고른 줄</span></span></div>
    <div class="fr"><span class="k">어떻게</span><span class="seg"><span class="btn on">비율</span><span class="btn">값</span><span class="btn">한 단계</span></span></div>
    <div class="fr"><span class="k">비율</span><span class="mono muted">×</span><span class="inp">0.8</span></div>
    <table class="t3"><tr><th>범례 줄</th><th class="r">지금</th><th class="r">바꾼 뒤</th></tr>
      <tr><td>굴광선 (유구 면)</td><td class="mono r">0.8</td><td class="mono r">0.64</td></tr>
      <tr><td>바닥선 · 유구 선 · 내부 시설 · 돌</td><td class="mono r">0.5</td><td class="mono r">0.4</td></tr>
      <tr><td>단면 위치선 · 유물 · 경사 표시</td><td class="mono r">0.2</td><td class="mono r">0.16</td></tr>
      <tr><td>격자 · 도곽</td><td class="mono r">0.1</td><td class="mono r">0.08</td></tr></table>
    <div class="note">바뀌는 선 14개. ⌘Z 한 번으로 모두 되돌립니다.</div>
    <div class="foot"><span class="b32 primary">바꾸기</span><span class="b32" id="cw2">닫기</span></div>
  </div>
</div>
<p class="bcap"><b>평면 도면 화면 (J003/J005)</b>: 1600 × 988 메인 실측 창입니다. 1:20 축척 격자, TM 국가좌표계(EPSG:5186), 굴광선(0.8pt)과 바닥선(0.5pt), 단면 위치선 A-A', 주혈(P1~P4), 표고점(EL. 152.36), 그리기 팔레트 및 범례=레이어 시스템이 유기적으로 연동됩니다.</p>
</section>

<!-- ========================= 3. SECTION SCREEN ========================= -->
<section class="vpage" id="p-section">
<div class="win">
  <div class="cap"><span class="lights"><i></i><i></i><i></i></span><span class="appicon"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M6 15l9-11 3 3-11 9z"/><path d="M4 20h16"/></svg></span>1호 주거지 A–A′ 단면 * - Trowel</div>
  <div class="ribbon">{RIBBON_SECTION}<div class="find"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/></svg>명령 찾기</div></div>
  <div class="ctrl">
    <span class="lbl">토층선 ›&nbsp; 범례</span>
    <span class="combo" style="width:190px"><svg width="18" height="10"><line x1="1" y1="5" x2="17" y2="5" stroke="#141413" stroke-width="1.2"/></svg>토층선 <span class="m">0.3 pt</span><span class="chev">▼</span></span>
    <span class="lbl">기준 표고</span><span class="combo"><span class="m">EL. 152.50</span> m</span><span class="vr"></span>
    <span class="lbl">해발선 간격</span><span class="seg"><span class="btn on">10 cm</span><span class="btn">50 cm</span></span><span class="vr"></span>
    <span class="lbl">잘린 면</span><span class="btn on">0.8pt + 빗금 45°</span>
    <span class="right"><span class="btn on">자석 켬</span><span class="lbl">허용치</span><span class="combo"><span class="m">2</span> px</span></span>
  </div>
  <div class="body">
    <aside class="tcol">{TOOLS_SECTION}</aside>
    <aside class="set">
      <div class="ph"><span>도면집</span><span class="q">＋</span></div>
      <div class="sec">열린 도면</div>
      <div class="drow">{IC_PLAN14}1호 주거지 평면</div>
      <div class="drow cur">{IC_SECTION14}1호 주거지 A–A′ 단면<span class="d"><i class="cdot"></i></span></div>
      <div class="drow">{IC_FINDS14}토기 1 실측</div>
      <div class="drow">{IC_SHEET14}도판 3 유구배치도</div>
      <div class="sec">광령리1 · 단면 목록</div>
      <div class="drow">{IC_SECTION14}1호 주거지 B–B′ 단면</div>
      <div class="drow">{IC_SECTION14}Tr-1 남벽 토층</div>
      <div class="drow">{IC_SECTION14}Tr-3 북벽 토층</div>
      <div class="setfoot"><div>단면 폭 · <span class="mono">4.60</span> m</div><div>기준 표고 · <span class="mono">EL. 152.50</span> m</div><div>축척 <span class="mono">1:20</span></div></div>
    </aside>
    <section class="center">
      <div class="tabs"><span class="tab">홈</span><span class="tab">1호 주거지 평면</span><span class="tab cur">1호 주거지 A–A′ 단면</span><span class="corner">A–A′ 단면&nbsp;&nbsp;<i class="cdot"></i>저장 안 됨</span></div>
      <div class="canvas">
        <div class="rc">+</div>
        <div class="rh"><svg width="1000" height="16"><g stroke="#87867F"><path d="M76 16v-7M196 16v-7M316 16v-7M436 16v-7M556 16v-7M676 16v-7"/></g><g class="m9"><text x="76" y="9" class="m9">0</text><text x="196" y="9" class="m9">1.0</text><text x="316" y="9" class="m9">2.0</text><text x="436" y="9" class="m9">3.0</text><text x="556" y="9" class="m9">4.0</text><text x="676" y="9" class="m9">5.0 m</text></g></svg></div>
        <div class="rv"></div>
        <div class="paper"><svg width="684" height="484" viewBox="0 0 760 540">{SECTION}</svg></div>
        <div class="ovl guide"><span style="display:flex;color:var(--hand)">{IC_PEN16}</span>토층선 그리기 · 0.3 pt<span class="vr"></span><span class="h">토층 경계 클릭 드래그</span></div>
        <div class="zoom ovl"><div>＋</div><div>－</div><div>맞춤</div></div>
        <div class="zpct ovl">화면 <b>85</b> %</div>
      </div>
    </section>
    <aside class="rcol">
      <div class="pal">
        <div class="ph"><span>단면 그리기 팔레트</span><span class="q">⋯</span></div>
        <div class="prow"><span class="k">굵기</span>{WEIGHTS_SECTION}</div>
        <div class="prow"><span class="k">빗금</span>{HATCH_SECTION}</div>
        <div class="prow"><span class="k">선종</span><span class="dseg"><span class="on">실선</span><span>파선</span><span>점선</span></span></div>
        <div class="pbtn"><span class="btn on">잘린 면 (0.8+0.1)</span><span class="btn">해발선 생성</span></div>
      </div>
      <div class="leg">
        <div class="ph"><span>단면 범례 · 레이어</span><span><span class="q">＋</span><span class="q">⋯</span></span></div>
        <div class="lrows">{SECTION_ROWS}</div>
        <div class="lfoot"><span class="muted">토층표</span><span class="btn">토층표 갱신</span><span class="btn">표 내보내기</span></div>
      </div>
      <div class="props">
        <div class="ph"><span>속성<span class="sub">A–A′ 단면</span></span><span class="muted">⌄</span></div>
        <div class="frow"><span class="k">단면</span><span class="combo" style="flex:1">A–A′ 북벽<span class="chev">▼</span></span></div>
        <div class="frow"><span class="k">표고</span><span class="combo" style="flex:1"><span class="m">152.50</span> m</span></div>
        <div class="ledger">
          <div class="row"><span class="k">단면 너비</span><span class="v">4.60</span><span class="u">m</span></div>
          <div class="row"><span class="k">최대 심도</span><span class="v">0.65</span><span class="u">m</span></div>
        </div>
      </div>
    </aside>
  </div>
  <div class="status">
    <span class="say">기준 표고선 EL. 152.50 m · 10cm 해발선 자동 생성 완료 · 잘린 면 45° 빗금 1.0mm 간격 적용됨</span>
    <span class="c">토층선</span><span class="c">자석 켬</span><span class="c"><i class="cdot"></i>저장 안 됨 1건</span>
    <span class="c"><span class="mono">1:20</span>&nbsp;· 단면 A–A′</span>
  </div>
</div>
<p class="bcap"><b>단면·토층 도면 화면 (J010/J015)</b>: 발굴조사의 핵심인 단면과 토층을 제도하는 전용 화면입니다. 기준 표고선(EL. m)을 축으로 50cm/10cm 자동 해발선 생성, 굴광선 절단면(0.8pt 외곽 + 0.1pt 45° 빗금 해칭), 토층 분할선(0.3pt), 토층 번호 원형 마크, 하단 삼선표(Three-line table) 토층 해설표가 자동 생성됩니다.</p>
</section>

<!-- ========================= 4. ARTIFACT SCREEN ========================= -->
<section class="vpage" id="p-artifact">
<div class="win">
  <div class="cap"><span class="lights"><i></i><i></i><i></i></span><span class="appicon"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M6 15l9-11 3 3-11 9z"/><path d="M4 20h16"/></svg></span>토기 1호 실측 * - Trowel</div>
  <div class="ribbon">{RIBBON_ARTIFACT}<div class="find"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/></svg>명령 찾기</div></div>
  <div class="ctrl">
    <span class="lbl">유물 실측 ›</span>
    <span class="combo" style="width:160px">외형 윤곽선 <span class="m">0.5 pt</span><span class="chev">▼</span></span>
    <span class="lbl">실측 모드</span><span class="seg"><span class="btn">전체 외형</span><span class="btn on">반쪽 대칭 (좌:외형 / 우:단면)</span></span><span class="vr"></span>
    <span class="lbl">3점 원호</span><span class="btn on">구경 24.6 cm</span><span class="btn">저경 9.0 cm</span>
    <span class="right"><span class="btn on">1cm 방안 격자</span><span class="combo"><span class="m">1:1</span> 정밀 실측</span></span>
  </div>
  <div class="body">
    <aside class="tcol">{TOOLS_ARTIFACT}</aside>
    <aside class="set">
      <div class="ph"><span>유물 목록</span><span class="q">＋</span></div>
      <div class="sec">열린 유물 실측</div>
      <div class="drow cur">{IC_FINDS14}토기 1 (무문토기 발)<span class="d"><i class="cdot"></i></span></div>
      <div class="drow">{IC_FINDS14}토기 2 (홍도 파편)</div>
      <div class="drow">{IC_FINDS14}석기 1 (마제석부)</div>
      <div class="sec">1호 주거지 출토품</div>
      <div class="drow">{IC_FINDS14}유물 3호 갈판</div>
      <div class="drow">{IC_FINDS14}유물 4호 방추차</div>
      <div class="setfoot"><div>실측 정밀도 · <span class="mono">0.05</span> mm</div><div>실제 축척 · <span class="mono">1 : 1</span></div><div>대칭 기준선 · <span class="mono">X = 0</span></div></div>
    </aside>
    <section class="center">
      <div class="tabs"><span class="tab">홈</span><span class="tab">1호 주거지 평면</span><span class="tab cur">토기 1호 실측</span><span class="corner">토기 1호&nbsp;&nbsp;<i class="cdot"></i>저장 안 됨</span></div>
      <div class="canvas">
        <div class="rc">+</div>
        <div class="rh"><svg width="1000" height="16"><g stroke="#87867F"><path d="M80 16v-7M170 16v-7M260 16v-7M350 16v-7M440 16v-7M530 16v-7M620 16v-7"/></g><g class="m9"><text x="80" y="9" class="m9">-10</text><text x="170" y="9" class="m9">-6</text><text x="260" y="9" class="m9">-3</text><text x="350" y="9" class="m9">0</text><text x="440" y="9" class="m9">+3</text><text x="530" y="9" class="m9">+6</text><text x="620" y="9" class="m9">+10 cm</text></g></svg></div>
        <div class="rv"></div>
        <div class="paper"><svg width="684" height="484" viewBox="0 0 760 520">{ARTIFACT}</svg></div>
        <div class="ovl guide"><span style="display:flex;color:var(--hand)">{IC_PEN16}</span>윤곽선 실측 · 0.5 pt<span class="vr"></span><span class="h">대칭 중심선 기준 1:1 실측</span></div>
        <div class="zoom ovl"><div>＋</div><div>－</div><div>맞춤</div></div>
        <div class="zpct ovl">화면 <b>100</b> % (1:1)</div>
      </div>
    </section>
    <aside class="rcol">
      <div class="pal">
        <div class="ph"><span>유물 실측 팔레트</span><span class="q">⋯</span></div>
        <div class="prow"><span class="k">굵기</span>{WEIGHTS_ARTIFACT}</div>
        <div class="prow"><span class="k">빗금</span>{HATCH_ARTIFACT}</div>
        <div class="prow"><span class="k">선종</span><span class="dseg"><span class="on">실선</span><span>쇄선</span><span>점선</span></span></div>
        <div class="pbtn"><span class="btn on">태토 단면 빗금</span><span class="btn">구경 측정</span></div>
      </div>
      <div class="leg">
        <div class="ph"><span>유물 범례 · 레이어</span><span><span class="q">＋</span><span class="q">⋯</span></span></div>
        <div class="lrows">{ARTIFACT_ROWS}</div>
        <div class="lfoot"><span class="muted">도판으로</span><span class="btn">도판에 넣기 (1:2)</span></div>
      </div>
      <div class="props">
        <div class="ph"><span>유물 속성<span class="sub">토기 1호</span></span><span class="muted">⌄</span></div>
        <div class="frow"><span class="k">기종</span><span class="combo" style="flex:1">무문토기 발<span class="chev">▼</span></span></div>
        <div class="frow"><span class="k">태토</span><span class="combo" style="flex:1">사립 포함 점토<span class="chev">▼</span></span></div>
        <div class="ledger">
          <div class="row"><span class="k">구경 (입지름)</span><span class="v">24.6</span><span class="u">cm</span></div>
          <div class="row"><span class="k">저경 (바닥지름)</span><span class="v">9.0</span><span class="u">cm</span></div>
          <div class="row"><span class="k">기고 (높이)</span><span class="v">18.2</span><span class="u">cm</span></div>
        </div>
      </div>
    </aside>
  </div>
  <div class="status">
    <span class="say">1:1 실측 모드 (정밀도 0.05 mm) · 중심선 기준 좌:외형선화 우:단면해치 대칭 활성 · 3점 원호 구경 24.6 cm</span>
    <span class="c">유물 실측</span><span class="c">1cm 방안</span><span class="c"><i class="cdot"></i>저장 안 됨 1건</span>
    <span class="c"><span class="mono">1:1</span>&nbsp;· 토기 1호</span>
  </div>
</div>
<p class="bcap"><b>유물 실측 도면 화면 (J011)</b>: 출토 유물(토기, 석기, 청동기)을 1:1 정밀도로 실측 제도하는 전용 화면입니다. 중심선(℄)을 축으로 좌측 외형 선화, 우측 기벽 단면(0.8pt + 0.1pt 태토 빗금)의 반쪽 대칭 제도, 3점 원호 맞춤 구경(입지름) 및 저경(바닥지름) 측정 호 도구, 1cm 실측 모눈 격자가 완벽하게 통합됩니다.</p>
</section>

<!-- ========================= 5. SHEET SCREEN ========================= -->
<section class="vpage" id="p-sheet">
<div class="win">
  <div class="cap"><span class="lights"><i></i><i></i><i></i></span><span class="appicon"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M6 15l9-11 3 3-11 9z"/><path d="M4 20h16"/></svg></span>도판 3호 1호 주거지 종합도판 - Trowel</div>
  <div class="ribbon">{RIBBON_SHEET}<div class="find"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/></svg>명령 찾기</div></div>
  <div class="ctrl">
    <span class="lbl">도판 규격 ›</span>
    <span class="combo" style="width:170px">A3 가로 (420 × 297 mm)<span class="chev">▼</span></span>
    <span class="lbl">도판 번호</span><span class="combo"><span class="m">PLATE 3</span></span><span class="vr"></span>
    <span class="lbl">도판 요소</span><span class="btn on">도북 방위</span><span class="btn on">복합 축척자</span><span class="btn on">정규 표제란</span><span class="vr"></span>
    <span class="right"><span class="btn on" style="background:var(--action);color:#fff;border-color:var(--action);font-weight:700">PDF 출판 내보내기</span></span>
  </div>
  <div class="body">
    <aside class="set" style="width:190px">
      <div class="ph"><span>도판 목록</span><span class="q">＋</span></div>
      <div class="sec">발굴 보고서 도판집</div>
      <div class="drow">{IC_SHEET14}도판 1 유적 위치도</div>
      <div class="drow">{IC_SHEET14}도판 2 유구 배치도</div>
      <div class="drow cur">{IC_SHEET14}도판 3 1호 주거지 실측도<span class="d"><i class="cdot"></i></span></div>
      <div class="drow">{IC_SHEET14}도판 4 2호 주거지 실측도</div>
      <div class="drow">{IC_SHEET14}도판 5 출토 유물 실측도</div>
      <div class="setfoot"><div>도판 판형 · <span class="mono">A3 가로</span></div><div>인쇄 해상도 · <span class="mono">600 DPI</span></div><div>여백 규격 · <span class="mono">20 mm</span></div></div>
    </aside>
    <section class="center">
      <div class="tabs"><span class="tab">홈</span><span class="tab">1호 주거지 평면</span><span class="tab cur">도판 3호 종합도판</span><span class="corner">PLATE 3&nbsp;&nbsp;<i class="cdot"></i>저장 안 됨</span></div>
      <div class="canvas">
        <div class="paper-sheet"><svg width="940" height="520" viewBox="0 0 780 560">{SHEET}</svg></div>
        <div class="ovl guide"><span style="display:flex;color:var(--hand)">{IC_SHEET14}</span>도판 요소 배치 모드<span class="vr"></span><span class="h">항목을 끌어 위치와 축척을 조정합니다</span></div>
        <div class="zoom ovl"><div>＋</div><div>－</div><div>맞춤</div></div>
        <div class="zpct ovl">화면 <b>100</b> % (A3)</div>
      </div>
    </section>
    <aside class="rcol" style="width:260px">
      <div class="pal">
        <div class="ph"><span>도판 구성 요소</span><span class="q">⋯</span></div>
        <div class="prow" style="margin-top:10px"><span class="k" style="width:70px">도판 제목</span><span class="combo" style="flex:1">1호 주거지 실측도</span></div>
        <div class="prow"><span class="k" style="width:70px">도북 방위</span><span class="combo" style="flex:1">북 화살 (28mm)<span class="chev">▼</span></span></div>
        <div class="prow"><span class="k" style="width:70px">축척 막대</span><span class="combo" style="flex:1">복합 쌍칸 막대<span class="chev">▼</span></span></div>
        <div class="prow"><span class="k" style="width:70px">표제란</span><span class="combo" style="flex:1">발굴조사 표준양식<span class="chev">▼</span></span></div>
      </div>
      <div class="leg">
        <div class="ph"><span>배치된 도면 블록</span></div>
        <div class="lrows" style="padding:6px">
          <div class="drow cur">{IC_PLAN14}평면도 (축척 1:40)</div>
          <div class="drow">{IC_SECTION14}A–A′ 단면도 (축척 1:20)</div>
          <div class="drow">{IC_FINDS14}토기 1호 (축척 1:2)</div>
        </div>
      </div>
      <div class="props" style="padding-top:10px">
        <div class="ph"><span>출판 설정</span></div>
        <div class="frow"><span class="k" style="width:60px">해상도</span><span class="combo" style="flex:1"><span class="m">600 DPI</span> (출판용)</span></div>
        <div class="frow"><span class="k" style="width:60px">색공간</span><span class="combo" style="flex:1">먹 1도 (K100)</span></div>
        <div class="frow"><span class="k" style="width:60px">글꼴</span><span class="combo" style="flex:1">곡선(외곽선) 변환</span></div>
        <div style="margin-top:14px;display:flex;flex-direction:column;gap:6px">
          <span class="b32 primary" style="width:100%">PDF 도판 출판 (⌘E)</span>
          <span class="b32" style="width:100%;justify-content:center">DXF CAD 내보내기</span>
        </div>
      </div>
    </aside>
  </div>
  <div class="status">
    <span class="say">국가유산 발굴조사 보고서 규격 검수 통과 (오류 0건) · A3 가로 도판 600 DPI 출판 준비 완료</span>
    <span class="c">도판 출판</span><span class="c"><i class="cdot"></i>저장 안 됨</span>
    <span class="c"><span class="mono">PLATE 3</span>&nbsp;· 1호 주거지</span>
  </div>
</div>
<p class="bcap"><b>보고서 종합 도판 화면 (J012)</b>: 발굴조사 보고서에 수록될 최종 A3/A4 도판을 조판하는 출판 화면입니다. 개별 평면도(1:40), 단면도(1:20), 유물 실측도(1:2)를 한 대지에 배치하고, 공용 도북, 복합 축척자, 자동 추출 범례표, 국가유산 정규 표제란(유적명, 도면명, 축척, 조사기관, 일자)을 포함하여 인쇄용 600 DPI PDF와 DXF를 출력합니다.</p>
</section>

<!-- ========================= 6. DIALOGS SCREEN ========================= -->
<section class="vpage" id="p-dialogs">
<div class="sheet">
  <h2>주요 대화상자 시스템 설계 (핵심 CAD 알고리즘 & 설정)</h2>
  <div class="lead">Strata의 다이얼로그 규격서(DESIGN_SPEC.md §4.5)에 따라, Ground(#FAF9F5) 본체, 표제란, 경도 눈금자, 삼선표, 단 하나의 Clay 주 버튼 규칙을 100% 동일하게 적용한 핵심 인터페이스 4종입니다.</div>

  <div class="dlg-grid">
    <!-- Dialog 1 -->
    <div class="dlg-box">
      <h3>축척 및 국가 TM 좌표계 설정 (J006)</h3>
      <div class="dlg-field"><span class="k">작업 좌표계</span>
        <div class="long-ruler">
          <div class="mark"><span>125°</span><i></i></div>
          <div class="mark"><span>126°</span><i></i></div>
          <div class="mark major"><span style="color:var(--clay-text);font-weight:700">127° (중부)</span><i></i></div>
          <div class="mark"><span>128°</span><i></i></div>
          <div class="mark major"><span style="color:var(--clay-text);font-weight:700">129° (동부)</span><i></i></div>
          <div class="mark"><span>130°</span><i></i></div>
          <div class="mark"><span>131°</span><i></i></div>
        </div>
      </div>
      <div class="dlg-field"><span class="k">원점 선택</span>
        <div class="seg" style="flex:1"><span class="btn on" style="flex:1;justify-content:center">EPSG:5186 중부원점 (추천)</span><span class="btn" style="flex:1;justify-content:center">EPSG:5187 동부원점</span></div>
      </div>
      <div class="dlg-field"><span class="k">도면 실측 축척</span>
        <div class="seg" style="flex:1"><span class="btn">1:10</span><span class="btn on">1:20</span><span class="btn">1:40</span><span class="btn">1:50</span><span class="btn">1:100</span></div>
      </div>
      <div class="dlg-field"><span class="k">종이 굵기 환산</span>
        <div class="inp"><span class="mono">0.8 pt 굴광선 = 종이 실측 0.28 mm</span></div>
      </div>
      <div class="dlg-foot">
        <span class="b32 primary">좌표계 및 축척 적용</span><span class="b32">취소</span>
      </div>
    </div>

    <!-- Dialog 2 -->
    <div class="dlg-box">
      <h3>밑그림 맞추기 2점·4점 정합 (J008)</h3>
      <div class="dlg-field"><span class="k">스캔 밑그림</span>
        <div class="inp"><span>광령리1_현장실측방안_0928.jpg (300 DPI)</span></div>
      </div>
      <div class="dlg-field"><span class="k">정합 방식</span>
        <div class="seg" style="flex:1"><span class="btn">2점 정합 (회전·축척)</span><span class="btn on">4점 아핀 정합 (왜곡 보정)</span></div>
      </div>
      <table class="t3">
        <tr><th>기준점 (GCP)</th><th>스캔 도면 좌표 (px)</th><th>국가 TM 좌표 (m)</th><th class="r">잔차 오차</th></tr>
        <tr><td>GCP 1</td><td class="mono">X 420, Y 380</td><td class="mono">147,962.418, 98,131.207</td><td class="mono r">0.64 mm</td></tr>
        <tr><td>GCP 2</td><td class="mono">X 2840, Y 392</td><td class="mono">148,021.774, 98,128.560</td><td class="mono r">0.82 mm</td></tr>
        <tr><td>GCP 3</td><td class="mono">X 2855, Y 2680</td><td class="mono">148,026.093, 98,181.945</td><td class="mono r">0.95 mm</td></tr>
        <tr><td>GCP 4</td><td class="mono">X 415, Y 2672</td><td class="mono">147,958.650, 98,186.312</td><td class="mono r">0.71 mm</td></tr>
      </table>
      <div class="dlg-field" style="margin-top:6px">
        <span class="k">종합 오차 분석</span>
        <span class="mono" style="font-size:12px;color:var(--done)">평균 제곱근 오차(RMSE): 0.82 mm (허용치 2.0 mm 이내 만족)</span>
      </div>
      <div class="dlg-foot">
        <span class="b32 primary">밑그림 정합 실행</span><span class="b32">취소</span>
      </div>
    </div>

    <!-- Dialog 3 -->
    <div class="dlg-box">
      <h3>DXF CAD 가져오기 및 레이어 매핑 (J007)</h3>
      <div class="dlg-field"><span class="k">CAD 파일</span>
        <div class="inp"><span>광령리1_현장측량_완성도면.dxf</span></div>
      </div>
      <div class="dlg-field"><span class="k">좌표계 유지</span>
        <label style="display:flex;align-items:center;gap:6px;font-size:12px"><input type="checkbox" checked> <b>중심 이동 안 함 (국가 TM 좌표계 절대 좌표 유지)</b></label>
      </div>
      <div class="dlg-field"><span class="k">문자 인코딩</span>
        <div class="seg" style="flex:1"><span class="btn on">CP949 (한국어 CAD 표준)</span><span class="btn">UTF-8</span></div>
      </div>
      <table class="t3">
        <tr><th>DXF 레이어</th><th>객체 수</th><th>Trowel 고고학 범례 짝맞춤</th></tr>
        <tr><td class="mono">SECTION_PROFILE</td><td class="mono">12</td><td>굴광선 (유구 면) 0.8pt</td></tr>
        <tr><td class="mono">LEVEL_CONTOUR</td><td class="mono">48</td><td>해발선 (50cm/10cm) 0.2pt</td></tr>
        <tr><td class="mono">POSTHOLES</td><td class="mono">16</td><td>주혈 및 내부시설 0.5pt</td></tr>
        <tr><td class="mono">GCP_POINTS</td><td class="mono">4</td><td>기준점 (GCP) 삼각 마크</td></tr>
      </table>
      <div class="dlg-foot">
        <span class="b32 primary">가져오기 및 레이어 생성</span><span class="b32">취소</span>
      </div>
    </div>

    <!-- Dialog 4 -->
    <div class="dlg-box">
      <h3>와콤/XP-Pen 펜 타블렛 감도 설정 (J016/J017)</h3>
      <div class="dlg-field"><span class="k">인식된 타블렛</span>
        <div class="inp"><span class="mono">Wacom Intuos Pro PTH-660 (Windows Ink API / macOS Cocoa)</span></div>
      </div>
      <div class="dlg-field"><span class="k">필압 감도 곡선</span>
        <div class="seg" style="flex:1"><span class="btn">부드러움</span><span class="btn on">보통 (표준 고고학 제도)</span><span class="btn">단단함 (균일 선)</span></div>
      </div>
      <div class="dlg-field"><span class="k">정밀 탭 허용치</span>
        <div class="inp"><span class="mono">3 pt (도면 정밀 점 찍기)</span></div>
      </div>
      <div class="dlg-field"><span class="k">닫기 허용 반경</span>
        <div class="inp"><span class="mono">8 px (시작점 근처에서 다각형 자동 닫기)</span></div>
      </div>
      <div class="dlg-field"><span class="k">손바닥 거르기</span>
        <label style="display:flex;align-items:center;gap:6px;font-size:12px"><input type="checkbox" checked> <b>Palm Rejection 켬 (스타일러스 감지 시 손터치 무시)</b></label>
      </div>
      <div class="dlg-foot">
        <span class="b32 primary">타블렛 설정 저장</span><span class="b32">취소</span>
      </div>
    </div>
  </div>
</div>
<p class="bcap"><b>주요 대화상자 시스템</b>: 발굴 CAD의 핵심 작업에 필요한 4대 인터페이스입니다. 경도 눈금자 기반 좌표계 설정(J006), 4점 아핀 밑그림 정합(J008), 절대 좌표 보존 DXF 임포트(J007), 펜 타블렛 필압 및 제스처 보정(J016/J017)을 포함합니다.</p>
</section>

<!-- ========================= 7. DESIGN SYSTEM ========================= -->
<section class="vpage" id="p-system">
<div class="sheet">
  <h2>디자인 기준서: Strata(hgis)와 완전 동일한 「조용한 도판」 규격</h2>
  <div class="lead">Trowel의 테마(theme.rs Brightness::Plate)는 Strata의 DESIGN_SPEC.md 토큰을 1:1로 계승합니다. 도판의 시각적 위계와 절제된 미학을 시스템 차원에서 강제합니다.</div>
  <div class="cols">
    <div>
      <div class="sec" style="padding:0 0 8px">색상 토큰 대조표</div>
      <table class="t3"><tr><th></th><th>토큰</th><th class="r">Hex</th><th>역할 및 규격</th><th>Trowel Tokens</th></tr>{TOKENS}</table>
    </div>
    <div>
      <div class="sec" style="padding:0 0 8px">켜짐의 세 가지 모습 (기준서 §1.4)</div>
      <div class="looks">{LOOKS}</div>
      <div class="sec" style="padding:0 0 8px">타이포그래피 위계 (기준서 §2.3)</div>
      <div class="types">
        <div><span class="k">Display · serif 40</span><span style="font:500 36px/44px var(--serif)">제주 광령리 유적</span></div>
        <div><span class="k">Title · serif 28</span><span style="font:500 28px/36px var(--serif)">Trowel</span></div>
        <div><span class="k">Heading · serif 20</span><span style="font:500 20px/28px var(--serif)">1호 주거지 평면 및 단면도</span></div>
        <div><span class="k">Lead · sans 15</span><span>한국 매장유산 실측도면 · 유물 실측 전문 CAD</span></div>
        <div><span class="k">Body · sans 13</span><span>굴광선 (유구 면) · 0.8 pt · 잘린 면 45° 빗금</span></div>
        <div><span class="k">Figure · mono 20</span><span style="font:400 20px var(--mono)">32.91</span><span class="faint">㎡</span><span style="margin-left:16px;font:400 20px var(--mono)">24.60</span><span class="faint">m</span></div>
        <div><span class="k">Mono · 12</span><span class="mono" style="font-size:12px">X 147,999.305 · Y 98,157.620 · EL. 152.50 m · 1:20</span></div>
      </div>
      <div class="sec" style="padding:24px 0 8px">절대 준수 원칙 (기준서 §1 · §7 Do-not list)</div>
      <ul class="rules">
        <li><b>단 하나의 대지 (One Sheet)</b>: 모든 고정 패널과 윈도우는 상아색 ground(#FAF9F5) 한 장입니다. 카드 안 카드(중첩 박스), 테두리와 그림자의 동시 사용을 엄격히 금지합니다.</li>
        <li><b>선으로 구분 (Rules, not boxes)</b>: 영역은 1px hairline 또는 24px 여백으로만 구분합니다. 모든 표는 머리와 바닥이 먹선인 삼선표(Three-line table)입니다.</li>
        <li><b>클레이는 손길(Hand)이다</b>: 흙색(#B5573A) 채움 단추는 한 화면에 오직 1개만 허용됩니다. 탭, 선택된 줄, 활성 도구에는 절대 흙색을 칠하지 않습니다.</li>
        <li><b>종이는 각지게, 조작은 둥글게</b>: 도면 종이와 표제란은 r0 각진 사각, 누르는 버튼은 r8, 작은 체크박스는 r4를 적용합니다.</li>
        <li><b>세리프는 고유명칭, 모노는 측정값</b>: Serif(명조)는 20px 이상의 조사명/유구명에만 쓰고, Mono는 좌표/치수/축척 숫자에만 씁니다.</li>
        <li><b>금지 목록</b>: 그라데이션, 블러, 유리 효과(Glassmorphism), 이모지, 어두운 다크 테마를 일절 배제합니다.</li>
      </ul>
    </div>
  </div>
</div>
</section>

<!-- ========================= 8. NOW APP SCREENSHOT ========================= -->
<section class="vpage" id="p-now">
  <div class="pair-cap"><h3>지금 실행 중인 Trowel 앱 실제 캡처</h3><span class="muted" style="font-size:12px">macOS 환경에서 제어 채널(ui.screenshot)을 통해 획득한 실시간 화면</span></div>
  <img class="shot" alt="지금 실행되는 Trowel 평면 화면" src="data:image/jpeg;base64,{SHOT}">
  <p class="bcap"><b>현재 개발 진척 상태</b>: J001(브랜딩/Quiet Plate 테마), J002(스마트 줌), J013(CI), J003(메인 창 1600×988 골격), J005(범례=레이어 시스템)까지 빌드 및 검증이 완료된 상태입니다. 이후 J014부터 시작하여 본 설계도의 기능들을 순차적으로 완성해 나갑니다.</p>
</section>

<!-- ========================= 9. BACKLOG ORDER ========================= -->
<section class="vpage" id="p-order">
<div class="sheet">
  <h2>Trowel 공장 개발 순서 및 백로그 (factory/backlog.md)</h2>
  <div class="lead">완성 목표 설계도에 정의된 모든 기능은 아래의 17개 작업 단위로 엄격히 관리되며, TDD 시험 작성 → 구현 → cargo xtask ci 관문 통과 원칙을 준수합니다.</div>
  <table class="t3"><tr><th class="r" style="width:56px">순서</th><th style="width:90px">작업</th><th>기능 구현 내용 및 명세</th><th style="width:120px">진행 상태</th></tr>{BACKLOG}</table>
</div>
</section>

<script>
document.querySelectorAll('.vtab').forEach(function(t){
  t.onclick = function(){
    document.querySelectorAll('.vtab,.vpage').forEach(function(x){x.classList.remove('cur')});
    t.classList.add('cur');
    document.getElementById('p-' + t.dataset.p).classList.add('cur');
  };
});
var c = document.getElementById('wc');
if (document.getElementById('ow')) {
  document.getElementById('ow').onclick = function(){c.classList.toggle('open')};
}
if (document.getElementById('cw')) {
  document.getElementById('cw').onclick = document.getElementById('cw2').onclick = function(){c.classList.remove('open')};
}
</script>
</body>
</html>
"""

if __name__ == "__main__":
    shot = ""
    if len(sys.argv) > 1:
        shot = base64.b64encode(Path(sys.argv[1]).read_bytes()).decode("ascii")
    elif OUT.exists():
        old = OUT.read_text(encoding="utf-8")
        start = old.find("data:image/jpeg;base64,")
        if start >= 0:
            start += len("data:image/jpeg;base64,")
            shot = old[start:old.find('"', start)]
    OUT.write_text(page(shot), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)} ({OUT.stat().st_size:,} bytes)")
