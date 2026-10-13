#!/usr/bin/env python3
"""Build docs/design/trowel-design-blueprint.html — Trowel's finished-target design board.

Two sources, nothing invented on top of them:

* The look is Strata's binding design 「조용한 도판」 (Quiet plate) from the user's hgis repository
  (docs/design/2026-10-04-quiet-plate/DESIGN_SPEC.md, tokens.json): the same colour tokens, the
  same three looks of "on" (hand fill = tool in hand, oat + ring = shown, ink underline = current
  view), one clay fill per view, serif only for names at 20 px and up, mono only for numbers.
  Trowel's own Quiet Plate theme (crates/ui-egui/src/theme.rs, Brightness::Plate) has the same values.
* The layout is Trowel's window as the app draws it today (crates/ui-egui/src/trowel, jobs J003
  and J005), checked against a screenshot of the running app taken over the control channel,
  which this page also shows. What later jobs bring (J006 scale and coordinates, J009 plan tools,
  J012 sheet elements, J014 hatching) is drawn finished, in the same parts.

Icons are Trowel's original assets/icons/tw-*.svg, inlined. The sample is Strata's survey
(광령리1, EPSG:5186 중부원점, 1호 주거지 32.91 ㎡) so both apps show the same site.

Run from the repository root: python3 docs/design/build_blueprint.py [screenshot.jpg]
"""

import base64
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ICONS = ROOT / "assets" / "icons"
OUT = ROOT / "docs" / "design" / "trowel-design-blueprint.html"


def ic(name: str, size: int = 24) -> str:
    svg = (ICONS / f"{name}.svg").read_text(encoding="utf-8").strip()
    return svg.replace('width="24" height="24"', f'width="{size}" height="{size}"', 1)


T = {
    "ground": "#FAF9F5", "surface": "#FFFFFF", "wash": "#F5F4ED", "desk": "#F0EEE6",
    "press": "#E8E6DC", "oat": "#E3DACC", "hand": "#30302E", "ink": "#141413",
    "ink2": "#3D3D3A", "muted": "#5E5D59", "faint": "#73726C", "line": "#DEDCD1",
    "edge": "#C2C0B6", "ring": "#B0AEA5", "action": "#B5573A", "clay": "#D97757",
    "clayText": "#9C4A2F", "block": "#A33B3B", "caution": "#7A5A00", "done": "#3F6B31",
}
# Screen px for a paper weight in pt (design 2.1 §7).
PX = {1.0: 2.5, 0.8: 2.0, 0.5: 1.25, 0.3: 0.85, 0.2: 0.6, 0.1: 0.45}


def dash(kind, w=1.0):
    w = max(w, 0.8)
    return {
        "dashdot": f' stroke-dasharray="{6*w:.1f} {2.5*w:.1f} {0.9:.1f} {2.5*w:.1f}"',
        "dash": f' stroke-dasharray="{4*w:.1f} {2.5*w:.1f}"',
        "dot": f' stroke-dasharray="0.9 {2.4*w:.1f}"',
    }.get(kind, "")


# ------------------------------------------------------------------ ribbon (ribbon.rs groups())
RIBBON = [
    ("파일", [("tw-open", "열기", ""), ("tw-save", "저장", "save"), ("tw-import", "가져오기", "")]),
    ("새 도면", [("tw-plan", "평면", ""), ("tw-section", "단면", ""), ("tw-finds", "유물", ""), ("tw-sheet", "도판", "")]),
    ("밑그림", [("tw-align", "맞추기", ""), ("tw-underlay", "밑그림", "shown")]),
    ("그리기", [("tw-pen", "펜", "on"), ("tw-pencil", "연필", ""), ("tw-section-line", "단면선", ""), ("tw-spot-height", "표고", ""), ("tw-bucket", "칠통", "")]),
    ("도면 요소", [("tw-north", "방위", ""), ("tw-scale-bar", "축척", ""), ("tw-legend", "범례표", ""), ("tw-title-block", "표제란", "")]),
    ("보기", [("tw-grid", "격자", ""), ("tw-magnet", "자석", "shown")]),
    ("내보내기", [("tw-pdf", "PDF", ""), ("tw-dxf", "DXF", ""), ("tw-strata", "Strata로", "")]),
]


def tile(icon, label, look):
    return f'<div class="tile {look}"><div class="well">{ic(icon, 22)}</div><span>{label}</span></div>'


def ribbon():
    return "".join(
        f'<div class="rgroup"><div class="rcap">{cap}</div><div class="tiles">{"".join(tile(*t) for t in tiles)}</div></div>'
        for cap, tiles in RIBBON
    )


# ------------------------------------------------------------------ tool column (tools.rs)
TOOLS = [
    [("tw-select", "V"), ("tw-direct", "A")],
    [("tw-pen", "P"), ("tw-pencil", "N"), ("tw-line", "\\"), ("tw-rect", "M"), ("tw-type", "T")],
    [("tw-rotate", "R"), ("tw-scale", "S"), ("tw-scissors", "C")],
    [("tw-bucket", "K"), ("tw-dropper", "I")],
    [("tw-hand", "H"), ("tw-zoom", "Z")],
]
ARCH = ["tw-section-line", "tw-spot-height", "tw-find-number", "tw-posthole", "tw-hachure"]


def tools():
    out = []
    for i, group in enumerate(TOOLS):
        for icon, key in group:
            out.append(f'<div class="tool{" on" if icon == "tw-pen" else ""}">{ic(icon, 19)}<b>{key}</b></div>')
        out.append('<div class="tsep"></div>' if i < len(TOOLS) - 1 else "")
    out.append('<div class="tcap">고고학</div>')
    out += [f'<div class="tool">{ic(icon, 19)}</div>' for icon in ARCH]
    out.append('<div class="proxy"><i class="fb"></i><i class="sb"></i></div>')
    return "".join(out)


# ------------------------------------------------------------------ legend (J005 평면 묶음, as the app sorts it)
EYE = '<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/></svg>'
OPEN = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" opacity=".35"><rect x="5" y="11" width="14" height="10" rx="1"/><path d="M8 11V7a4 4 0 0 1 7.5-2"/></svg>'
SHUT = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="5" y="11" width="14" height="10" rx="1"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg>'

STRATA_ROWS = [  # name, weight, dash, look, star, count, flag
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


def look(weight, d, kind):
    ink = T["ink"]
    if kind == "mark":
        return f'<span class="mk">{d}</span>'
    if kind == "under":
        return f'<svg width="22" height="12"><rect x="1.5" y="1.5" width="19" height="9" fill="{T["wash"]}" stroke="{T["edge"]}"/><path d="M4 9l4-4 3 3 3-3 4 4" fill="none" stroke="{T["faint"]}"/></svg>'
    w = PX[weight] if weight else 1
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


# ------------------------------------------------------------------ palette (palette.rs)
def weight_chips():
    out = [f'<div class="chip{" chosen" if w == 0.8 else ""}"><i style="height:{max(PX[w],1):.2f}px"></i><span>{w:.1f}</span></div>'
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


def hatch_chips():
    spec = [("45", "45°", True), ("135", "135°", False), ("alt", "번갈아", False), ("none", "없음", False)]
    return "".join(f'<div class="hchip{" chosen" if c else ""}">{hatch_icon(k) if k != "none" else ""}<span>{l}</span></div>' for k, l, c in spec)


# ------------------------------------------------------------------ the plan on paper
def plan():
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
    # 도판 요소 (J012): 축척 막대, 도북
    g.append(f'<g transform="translate(60 540)"><rect width="66" height="5" fill="{ink}"/><rect x="66" width="66" height="5" fill="#fff" stroke="{ink}"/><rect x="132" width="66" height="5" fill="{ink}"/>'
             f'<text y="-4" class="m9">0</text><text x="62" y="-4" class="m9">2</text><text x="128" y="-4" class="m9">4</text><text x="190" y="-4" class="m9">6 m</text><text x="214" y="6" class="s10">축척 1 : 20</text></g>')
    g.append(f'<g transform="translate(742 500)"><path d="M0 -26 l7 22 -7 -5 -7 5 z" fill="{ink}"/><text y="12" class="s10" text-anchor="middle">도북</text></g>')
    return "".join(g)


# ------------------------------------------------------------------ design-basis tables
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
    ("J013", "깃허브 액션 CI", "끝남"), ("J003", "메인 창 짜임 (이 판)", "끝남"), ("J005", "범례 = 레이어", "끝남"),
    ("J014", "그리기 팔레트와 빗금, 잘린 면", "다음"), ("J002b", "누르고 있는 동안만 Z 돋보기", ""),
    ("J004", "홈 화면과 새 도면 견본", ""), ("J006", "축척과 실제 좌표 (5186 · 5187)", ""), ("J015", "해발선 만들기", ""),
    ("J007", "좌표를 지키는 DXF 가져오기, desca 단면", ""), ("J016", "펜 1단계", ""), ("J008", "밑그림 맞추기 (2점 · 4점)", ""),
    ("J009", "평면 도구: 단면선 · 표고 · 유물 번호 · 주혈 · 경사", ""), ("J010", "단면 · 토층 화면", ""), ("J011", "유물 화면", ""),
    ("J012", "도판과 PDF: 방위 · 축척 막대 · 범례표 · 표제란", ""), ("J017", "펜 2단계", ""), ("J018", "양식 파일과 범례표", ""),
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
        "{RIBBON}": ribbon(), "{TOOLS}": tools(), "{STRATA}": rows(STRATA_ROWS), "{TROWEL}": rows(TROWEL_ROWS),
        "{WEIGHTS}": weight_chips(), "{HATCH}": hatch_chips(), "{PLAN}": plan(),
        "{IC_PEN16}": ic("tw-pen", 16), "{IC_PLAN14}": ic("tw-plan", 14), "{IC_SECTION14}": ic("tw-section", 14),
        "{IC_FINDS14}": ic("tw-finds", 14), "{IC_SHEET14}": ic("tw-sheet", 14),
        "{TOKENS}": tokens_table(), "{LOOKS}": looks(), "{BACKLOG}": backlog(), "{SHOT}": shot,
    }
    html = HTML
    for k, v in rep.items():
        html = html.replace(k, v)
    return html


HTML = r"""<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="UTF-8">
<title>Trowel 완성 목표 설계도 · 조용한 도판</title>
<style>
:root{--ground:#FAF9F5;--surface:#FFFFFF;--wash:#F5F4ED;--desk:#F0EEE6;--press:#E8E6DC;--oat:#E3DACC;--hand:#30302E;--ink:#141413;--ink2:#3D3D3A;--muted:#5E5D59;--faint:#73726C;--line:#DEDCD1;--edge:#C2C0B6;--ring:#B0AEA5;--action:#B5573A;--action-h:#A44D32;--clay:#D97757;--clay-text:#9C4A2F;--block:#A33B3B;--caution:#7A5A00;--caution-edge:#D9C27A;--caution-wash:#F7EFD6;
--serif:"Noto Serif KR","AppleMyungjo",Batang,serif;--sans:"Noto Sans KR","Apple SD Gothic Neo","Malgun Gothic",sans-serif;--mono:"JetBrains Mono",Consolas,"SF Mono",Menlo,monospace;--overlay:0 2px 12px rgba(20,20,19,.12)}
*{box-sizing:border-box;margin:0;padding:0}
body{background:var(--desk);color:var(--ink);font:400 13px/20px var(--sans);word-break:keep-all}
.mono{font-family:var(--mono)} .r{text-align:right} .muted{color:var(--muted)} .faint{color:var(--faint)}
.cdot{display:inline-block;width:6px;height:6px;border-radius:50%;background:var(--clay);margin-right:6px;vertical-align:1px}
/* viewer: the same plate */
.vhead{background:var(--ground);border-bottom:1px solid var(--line);padding:24px 32px 0}
.vhead .top{display:flex;align-items:center;gap:16px}
.vhead h1{font:500 28px/36px var(--serif)}
.vhead .rule{width:1px;height:28px;background:var(--edge)}
.vhead .lead{font-size:15px;line-height:24px;color:var(--muted)}
.vhead p{color:var(--muted);font-size:12px;line-height:18px;margin-top:8px;max-width:1000px}
.vtabs{display:flex;margin-top:16px;height:36px}
.vtab{padding:0 16px;display:flex;align-items:center;color:var(--muted);cursor:pointer;border-bottom:2px solid transparent;margin-bottom:-1px}
.vtab:hover{color:var(--ink)} .vtab.cur{color:var(--ink);border-bottom-color:var(--ink)}
.vpage{display:none;padding:32px} .vpage.cur{display:block}
.bcap{font-size:11px;line-height:16px;color:var(--muted);margin-top:10px;max-width:1600px}

/* ===== window 1600 x 988 ===== */
.win{width:1600px;height:988px;background:var(--ground);border:1px solid var(--edge);display:flex;flex-direction:column;overflow:hidden;position:relative}
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
.btn{height:24px;display:inline-flex;align-items:center;padding:0 8px;border:1px solid var(--edge);border-radius:8px;background:var(--surface);white-space:nowrap}
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
.ph{height:30px;display:flex;align-items:center;justify-content:space-between;padding:0 10px 0 10px}
.q{height:22px;min-width:22px;padding:0 4px;display:inline-flex;align-items:center;justify-content:center;border-radius:7px;font-size:12px}
.sec{display:flex;align-items:center;gap:8px;font-size:11px;line-height:16px;color:var(--muted);padding:8px 10px 6px}
.sec::after{content:"";flex:1;height:1px;background:var(--line)}
.drow{height:26px;margin:0 6px;border-radius:6px;display:flex;align-items:center;gap:7px;padding:0 6px;font-size:12px}
.drow svg{color:var(--hand);flex:none}
.drow.cur{background:var(--oat)} .drow .d{margin-left:auto}
.setfoot{margin-top:auto;border-top:1px solid var(--line);padding:8px 10px;font-size:11px;line-height:17px;color:var(--muted)}
.setfoot .mono{color:var(--ink)}
.center{flex:1;display:flex;flex-direction:column;min-width:0}
.tabs{height:28px;flex:none;display:flex;border-bottom:1px solid var(--line);padding:0 8px;font-size:12px}
.tab{display:flex;align-items:center;padding:0 14px;color:var(--muted);border-bottom:2px solid transparent;margin-bottom:-1px}
.tab.cur{color:var(--ink);border-bottom-color:var(--ink)}
.tabs .corner{margin-left:auto;display:flex;align-items:center;padding-right:6px}
.canvas{flex:1;position:relative;background:var(--desk);overflow:hidden}
.rh{position:absolute;left:16px;right:0;top:0;height:16px;background:var(--wash);border-bottom:1px solid var(--line)}
.rv{position:absolute;left:0;top:16px;bottom:0;width:16px;background:var(--wash);border-right:1px solid var(--line)}
.rc{position:absolute;left:0;top:0;width:16px;height:16px;background:var(--wash);border:solid var(--line);border-width:0 1px 1px 0;font-size:10px;text-align:center;line-height:14px;color:var(--muted)}
.paper{position:absolute;left:92px;top:38px;width:684px;height:484px;background:#fff;border:1px solid var(--ink2)}
.m9{font:400 9px var(--mono);fill:var(--muted)} .m8{font:400 8px var(--mono);fill:var(--ink)} .s10{font:400 10px var(--sans);fill:var(--ink)} .s12{font:400 12px var(--sans);fill:var(--ink)}
.ovl{position:absolute;background:var(--surface);border:1px solid var(--edge);border-radius:8px;box-shadow:var(--overlay)}
.guide{left:24px;top:24px;height:28px;display:flex;align-items:center;gap:8px;padding:0 10px;font-size:12px}
.guide .h{color:var(--ink2)}
.zoom{right:10px;bottom:46px;width:28px;display:flex;flex-direction:column}
.zoom div{height:28px;display:flex;align-items:center;justify-content:center;border-bottom:1px solid var(--line);font-size:14px}
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
.b32{height:32px;padding:0 16px;border-radius:8px;border:1px solid var(--edge);background:#fff;font-size:13px;display:inline-flex;align-items:center}
.b32.primary{background:var(--action);border-color:var(--action);color:#fff;font-weight:700;min-width:88px;justify-content:center}
/* other boards */
.sheet{background:var(--ground);border:1px solid var(--edge);width:1600px;padding:32px}
.sheet h2{font:500 20px/28px var(--serif);margin-bottom:4px}
.sheet .lead{font-size:12px;line-height:18px;color:var(--muted);margin-bottom:16px;max-width:960px}
.sheet table.t3 td{height:32px;font-size:13px;padding:0 8px} .sheet table.t3 th{padding:0 8px}
.sw{display:inline-block;width:64px;height:20px;border-radius:4px;border:1px solid var(--line);vertical-align:middle}
.cols{display:grid;grid-template-columns:1fr 1fr;gap:48px}
.looks{display:flex;gap:40px;margin:8px 0 24px}
.lookcell{display:flex;flex-direction:column;align-items:center;gap:8px}
.lookcell .cap2{font-size:11px;color:var(--muted)}
.lookcell .cap{height:auto;border:0;padding:0;font-size:11px;color:var(--muted);text-align:center;width:130px;display:block}
.types div{display:flex;align-items:baseline;gap:16px;border-top:1px solid var(--press);padding:8px 0}.types div:first-child{border-top:0}
.types .k{width:130px;font-size:11px;color:var(--muted);flex:none}
.rules li{list-style:none;padding:6px 0;border-top:1px solid var(--press)}.rules li:first-child{border-top:0}
.shot{width:1600px;border:1px solid var(--edge);display:block}
.pair-cap{display:flex;gap:8px;align-items:baseline;margin:0 0 8px}.pair-cap h3{font:400 15px/24px var(--sans)}
</style>
</head>
<body>
<header class="vhead">
  <div class="top"><h1>Trowel</h1><span class="rule"></span><span class="lead">고고학 실측 도면 · 완성 목표 설계도</span></div>
  <p>디자인 기준은 Strata(hgis)의 「조용한 도판」 기준서입니다. 같은 색 토큰, 같은 켜짐의 세 모습, 한 화면에 흙색 채움 하나, 이름은 serif · 숫자는 mono. 창 짜임은 지금 실행되는 Trowel 앱 그대로이며(「지금 앱」 판의 실제 화면), 뒤 작업이 더할 것(축척 · 좌표, 평면 도구, 도판 요소, 빗금)은 같은 자리에 완성된 모습으로 그렸습니다. 견본 조사는 Strata와 같은 「광령리1」입니다.</p>
  <nav class="vtabs"><div class="vtab cur" data-p="main">완성 목표</div><div class="vtab" data-p="now">지금 앱</div><div class="vtab" data-p="system">디자인 기준</div><div class="vtab" data-p="order">개발 순서</div></nav>
</header>

<section class="vpage cur" id="p-main">
<div class="win">
  <div class="cap"><span class="lights"><i></i><i></i><i></i></span><span class="appicon"><svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M6 15l9-11 3 3-11 9z"/><path d="M4 20h16"/></svg></span>1호 주거지 평면 * - Trowel</div>
  <div class="ribbon">{RIBBON}<div class="find"><svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/></svg>명령 찾기</div></div>
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
    <aside class="tcol">{TOOLS}</aside>
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
        <div class="ovl zoom"><div>＋</div><div>－</div><div>맞춤</div></div>
        <div class="ovl zpct">화면 <b>68</b> %</div>
      </div>
    </section>
    <aside class="rcol">
      <div class="pal">
        <div class="ph"><span>그리기 팔레트<span class="sub">다음에 그릴 선 · pt</span></span><span class="q">⋯</span></div>
        <div class="prow"><span class="k">굵기</span>{WEIGHTS}</div>
        <div class="prow"><span class="k">빗금</span>{HATCH}</div>
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
<p class="bcap">완성 목표 · 평면 화면. 짜임과 부품은 지금 앱과 같고, 뒤 작업이 채울 자리(축척 1:20 · 작업 5186, 시대 · 종류 · 면적, 빗금 45°, 도판의 축척 막대와 도북)를 채웠습니다. 흙색 채움은 저장 칸 하나, 고른 펜은 먹 채움, 켜진 밑그림 · 자석은 귀리색, 지금 탭은 먹 밑줄입니다. 「모든 굵기 ×0.8 …」을 누르면 카드가 열립니다.</p>
</section>

<section class="vpage" id="p-now">
  <div class="pair-cap"><h3>지금 앱 · 실제 화면</h3><span class="muted" style="font-size:12px">2026-10-13, macOS, 제어 채널 ui.screenshot · 새 도면 → 평면용 범례 넣기 → 펜</span></div>
  <img class="shot" alt="지금 실행되는 Trowel 평면 화면" src="data:image/jpeg;base64,{SHOT}">
  <p class="bcap">완성 목표 판은 이 화면의 짜임 · 부품 · 글자를 그대로 따릅니다. 다른 점은 뒤 작업이 채울 자리뿐입니다: 흐린 리본 칸(단면 · 유물 · 도판 · 맞추기 · 단면선 · 표고 · 도면 요소 · Strata로)과 고고학 도구, 「축척 정하지 않음 · 좌표계 정하지 않음」, 속성의 「고르세요」.</p>
</section>

<section class="vpage" id="p-system">
<div class="sheet">
  <h2>디자인 기준: Strata와 같은 판</h2>
  <div class="lead">Trowel의 Quiet Plate 테마(theme.rs Brightness::Plate)는 Strata 기준서 §2 토큰과 같은 값입니다. 오른쪽 열은 그 토큰을 쓰는 Trowel Tokens 이름입니다.</div>
  <div class="cols">
    <div><div class="sec" style="padding:0 0 8px">색 토큰</div>
      <table class="t3"><tr><th></th><th>토큰</th><th class="r">값</th><th>쓰임</th><th>Trowel Tokens</th></tr>{TOKENS}</table></div>
    <div><div class="sec" style="padding:0 0 8px">켜짐의 세 모습 (기준서 §1.4)</div>
      <div class="looks">{LOOKS}</div>
      <div class="sec" style="padding:0 0 8px">글꼴 역할 (기준서 §2.3)</div>
      <div class="types">
        <div><span class="k">Title · serif 28</span><span style="font:500 28px/36px var(--serif)">Trowel</span></div>
        <div><span class="k">Heading · serif 20</span><span style="font:500 20px/28px var(--serif)">1호 주거지</span></div>
        <div><span class="k">Body · sans 13</span><span>굴광선 (유구 면)</span></div>
        <div><span class="k">Label · sans 12</span><span class="muted" style="font-size:12px">굵기 · 빗금 · 선종은 오른쪽 「그리기 팔레트」에서</span></div>
        <div><span class="k">Figure · mono 20</span><span style="font:400 20px var(--mono)">32.91</span><span class="faint">㎡</span></div>
        <div><span class="k">Mono · 12</span><span class="mono" style="font-size:12px">X 147,999.305 · EL. 152.36 · 1:20 · 5186 → 5179</span></div>
      </div>
      <div class="sec" style="padding:24px 0 8px">지키는 것 (기준서 §1 · §7)</div>
      <ul class="rules">
        <li>붙은 판은 모두 ground 한 장입니다. 카드 안 카드, 테두리와 그림자를 함께 쓰지 않습니다.</li>
        <li>구역은 1 px 선 하나 또는 여백으로 나눕니다. 표는 삼선표입니다.</li>
        <li>흙색 채움은 한 화면에 하나입니다. 탭 · 초점 · 고른 줄 · 고른 도구에는 쓰지 않습니다.</li>
        <li>종이를 그린 것은 각지게, 누르는 것은 둥글게(8), 작은 표시는 4.</li>
        <li>serif는 20 px 이상의 이름에만, mono는 숫자와 코드에만 씁니다.</li>
        <li>그라데이션 · 흐림 · 반투명 · 그림 문자 · 어두운 화면을 쓰지 않습니다.</li>
      </ul></div>
  </div>
</div>
</section>

<section class="vpage" id="p-order">
<div class="sheet">
  <h2>개발 순서</h2>
  <div class="lead">factory/backlog.md 순서입니다. 작업마다 명세 → 시험 먼저 → 구현 → cargo xtask ci → 실제 화면을 이 판과 나란히 찍어 확인 → 커밋 하나.</div>
  <table class="t3"><tr><th class="r" style="width:56px">순서</th><th style="width:90px">작업</th><th>내용</th><th style="width:120px">상태</th></tr>{BACKLOG}</table>
</div>
</section>
<script>
document.querySelectorAll('.vtab').forEach(function(t){t.onclick=function(){document.querySelectorAll('.vtab,.vpage').forEach(function(x){x.classList.remove('cur')});t.classList.add('cur');document.getElementById('p-'+t.dataset.p).classList.add('cur')}});
var c=document.getElementById('wc');document.getElementById('ow').onclick=function(){c.classList.toggle('open')};
document.getElementById('cw').onclick=document.getElementById('cw2').onclick=function(){c.classList.remove('open')};
</script>
</body>
</html>
"""

if __name__ == "__main__":
    shot = ""
    if len(sys.argv) > 1:
        shot = base64.b64encode(Path(sys.argv[1]).read_bytes()).decode("ascii")
    elif OUT.exists():
        # Keep the screenshot already in the page when none is given.
        old = OUT.read_text(encoding="utf-8")
        start = old.find("data:image/jpeg;base64,")
        if start >= 0:
            start += len("data:image/jpeg;base64,")
            shot = old[start:old.find('"', start)]
    OUT.write_text(page(shot), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)} ({OUT.stat().st_size:,} bytes)")
