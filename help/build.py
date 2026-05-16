#!/usr/bin/env python3
"""TCML help HTML builder (ja + en).

source.py の DATA_JA / DATA_EN を読み、各 sample ブロックの TCML を
tchart-cli でレンダリングして SVG を inline 埋め込みし、
スタンドアロン HTML をスタイル含めて生成する。

Usage:
    python3 help/build.py            # ja + en 両方を出力
    python3 help/build.py --lang ja  # 日本語のみ
    python3 help/build.py --lang en  # 英語のみ

出力先:
    help/output/tcml-format.html      # 日本語
    help/output/tcml-format.en.html   # 英語
"""
from __future__ import annotations

import argparse
import html
import re
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
sys.path.insert(0, str(HERE))
from source import DATA_EN, DATA_JA  # noqa: E402

OUT_DIR = HERE / "output"
TCHART_BIN = ROOT / "target" / "debug" / "tchart"


# ---------- tchart-cli runner ----------


def ensure_tchart_built() -> None:
    subprocess.run(
        [
            "cargo",
            "build",
            "--quiet",
            "--manifest-path",
            str(ROOT / "Cargo.toml"),
            "--bin",
            "tchart",
        ],
        check=True,
    )


_SVG_CACHE: dict[str, str] = {}


def collect_sample_tcmls(data: dict) -> list[str]:
    """DATA をスキャンして sample ブロックの TCML を順番通りに収集する。"""
    samples: list[str] = []
    for section in data["sections"]:
        for block in section.get("blocks", []):
            if block["type"] in ("sample", "wavedrom_sample"):
                samples.append(block["code"])
    return samples


def batch_render_samples(samples: list[str]) -> None:
    """全 sample TCML を `tchart batch svg` で 1 回呼び出してまとめて SVG 化し、
    `_SVG_CACHE` (TCML テキスト → SVG テキスト) を埋める。
    """
    unique_tcmls = [t for t in dict.fromkeys(samples) if t not in _SVG_CACHE]
    if not unique_tcmls:
        return
    with tempfile.TemporaryDirectory() as d:
        tmpdir = Path(d)
        out_dir = tmpdir / "out"
        out_dir.mkdir()
        in_paths: list[Path] = []
        for i, tcml in enumerate(unique_tcmls):
            tc_path = tmpdir / f"s{i:04d}.tc"
            tc_path.write_text(tcml)
            in_paths.append(tc_path)
        subprocess.run(
            [
                str(TCHART_BIN),
                "batch",
                "svg",
                *(str(p) for p in in_paths),
                "-o",
                f"{out_dir}/",
            ],
            check=True,
            capture_output=True,
        )
        for i, tcml in enumerate(unique_tcmls):
            svg = (out_dir / f"s{i:04d}.svg").read_text()
            svg = re.sub(r"<metadata>.*?</metadata>", "", svg, flags=re.DOTALL)
            _SVG_CACHE[tcml] = svg


def render_svg(tcml: str) -> str:
    return _SVG_CACHE[tcml]


# ---------- code highlighting (very light) ----------


def highlight(code: str) -> str:
    out_lines = []
    for line in code.split("\n"):
        m = re.search(r"(^|\s)#", line)
        if m:
            cut = m.start() if m.start() == 0 else m.start() + 1
            head = line[:cut]
            tail = line[cut:]
            out_lines.append(_hl_no_comment(head) + f'<span class="c">{html.escape(tail)}</span>')
        else:
            out_lines.append(_hl_no_comment(line))
    return "\n".join(out_lines)


def _hl_no_comment(s: str) -> str:
    s = html.escape(s)
    s = re.sub(
        r"(^|[\s(])(@-&gt;|@[A-Za-z_][\w-]*)",
        r'\1<span class="d">\2</span>',
        s,
    )
    s = re.sub(r"(&quot;[^&]*?&quot;)", r'<span class="s">\1</span>', s)
    return s


# ---------- block renderers ----------


def render_block(block: dict, data: dict) -> str:
    t = block["type"]
    if t == "text":
        return block["content"]
    if t == "heading":
        lv = block.get("level", 3)
        return f"<h{lv}>{block['text']}</h{lv}>"
    if t == "code":
        return f"<pre><code>{highlight(block['code'])}</code></pre>"
    if t == "sample":
        tcml = block["code"]
        display = block.get("display", tcml)
        svg = render_svg(tcml)
        return (
            f"<pre><code>{highlight(display)}</code></pre>\n"
            f'<div class="preview">{svg}</div>'
        )
    if t == "wavedrom_sample":
        tcml = block["code"]
        json_text = block["json"]
        wavedrom_svg_path = HERE / block["wavedrom_svg_file"]
        wavedrom_svg = wavedrom_svg_path.read_text(encoding="utf-8")
        tcml_svg_path = HERE / block["tcml_svg_file"]
        tcml_svg = tcml_svg_path.read_text(encoding="utf-8")
        labels = data["labels"]
        return (
            '<div class="wavedrom-sample">\n'
            f"<h4>{labels['wavedrom_tcml_input']}</h4>\n"
            f"<pre><code>{highlight(tcml)}</code></pre>\n"
            f"<h4>{labels['wavedrom_tcml_render']} (<code>tchart svg</code>)</h4>\n"
            f'<div class="preview">{tcml_svg}</div>\n'
            f"<h4>{labels['wavedrom_json_output']} (<code>tchart wavedrom</code>)</h4>\n"
            f"<pre><code>{html.escape(json_text)}</code></pre>\n"
            f"<h4>{labels['wavedrom_render']} (<code>wavedrom-cli</code>)</h4>\n"
            f'<div class="preview">{wavedrom_svg}</div>\n'
            "</div>"
        )
    if t == "table":
        h = "".join(f"<th>{c}</th>" for c in block["headers"])
        rows = "\n".join(
            "<tr>" + "".join(f"<td>{c}</td>" for c in r) + "</tr>"
            for r in block["rows"]
        )
        return (
            f"<table><thead><tr>{h}</tr></thead><tbody>\n{rows}\n</tbody></table>"
        )
    if t == "error_table":
        h = "".join(f"<th>{c}</th>" for c in block["headers"])
        rows = "\n".join(
            f'<tr class="error-row">'
            + "".join(f"<td>{c}</td>" for c in r)
            + "</tr>"
            for r in block["rows"]
        )
        return (
            f"<table><thead><tr>{h}</tr></thead><tbody>\n{rows}\n</tbody></table>"
        )
    if t == "references":
        refs = "".join(
            f'<li><a href="{r["url"]}" target="_blank" rel="noopener">{r["name"]}</a></li>'
            for r in data.get("references", [])
        )
        return f"<ul>{refs}</ul>"
    raise ValueError(f"unknown block type: {t}")


def render_section(s: dict, data: dict) -> str:
    blocks = "\n".join(render_block(b, data) for b in s.get("blocks", []))
    return (
        f'<section id="{s["id"]}">\n'
        f'<h2><span class="num">{s["num"]}.</span>{s["title"]}</h2>\n'
        f"{blocks}\n</section>\n"
    )


# ---------- main HTML ----------

STYLE = """
:root {
  --primary: #2563eb;
  --primary-dark: #1e40af;
  --primary-soft: #eff6ff;
  --bg: #ffffff;
  --bg-alt: #f8fafc;
  --text: #0f172a;
  --text-muted: #64748b;
  --border: #e2e8f0;
  --code-bg: #1e293b;
  --code-text: #e2e8f0;
  --code-comment: #94a3b8;
  --code-dir: #fbbf24;
  --code-string: #86efac;
  --warn-bd: #f59e0b;
  --danger: #dc2626;
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Hiragino Sans",
               "Hiragino Kaku Gothic ProN", "Noto Sans CJK JP", "Yu Gothic UI",
               Meiryo, sans-serif;
  line-height: 1.7;
  color: var(--text);
  background: var(--bg);
}
header.top { background: var(--bg-alt); padding: 1.4rem 1.5rem; border-bottom: 1px solid var(--border); }
header.top .inner { max-width: 1100px; margin: 0 auto; display: flex; align-items: baseline; flex-wrap: wrap; gap: 0.7rem; }
header.top h1 { margin: 0; font-size: 1.8rem; color: var(--primary); letter-spacing: -0.01em; }
header.top .subtitle { margin: 0; color: var(--text-muted); font-size: 0.95rem; }
header.top .ext { margin-left: auto; font-size: 0.85rem; color: var(--text-muted); }
header.top .ext code { background: white; }
header.top .lang-switch { font-size: 0.85rem; }
header.top .lang-switch a { color: var(--primary); text-decoration: none; }
header.top .lang-switch a:hover { text-decoration: underline; }
.container { max-width: 1100px; margin: 0 auto; padding: 1.5rem; display: grid; grid-template-columns: 220px 1fr; gap: 2rem; }
nav.toc { position: sticky; top: 0.75rem; align-self: start; max-height: calc(100vh - 1.5rem); overflow-y: auto; font-size: 0.88rem; border-right: 1px solid var(--border); padding-right: 1rem; }
nav.toc h3 { margin: 0 0 0.5rem; font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.1em; color: var(--text-muted); }
nav.toc ul { list-style: none; padding: 0; margin: 0; }
nav.toc a { color: var(--text); text-decoration: none; display: block; padding: 0.25rem 0.6rem; border-left: 2px solid transparent; border-radius: 3px; font-size: 0.86rem; }
nav.toc a:hover { background: var(--primary-soft); border-left-color: var(--primary); color: var(--primary-dark); }
main { min-width: 0; }
section { margin-bottom: 2.5rem; padding-bottom: 0.5rem; border-bottom: 1px solid var(--border); }
section:last-child { border-bottom: none; }
h2 { color: var(--primary); font-size: 1.45rem; margin: 0 0 0.5rem; letter-spacing: -0.01em; }
h2 .num { color: var(--text-muted); font-weight: 400; margin-right: 0.4em; }
h3 { color: var(--text); margin-top: 1.5rem; margin-bottom: 0.4rem; font-size: 1.05rem; }
h4 { color: var(--text-muted); margin: 1rem 0 0.3rem; font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.06em; font-weight: 600; }
p { margin: 0.5rem 0; }
ul, ol { margin: 0.5rem 0; padding-left: 1.5rem; }
li { margin: 0.15rem 0; }
code { font-family: "SF Mono", "JetBrains Mono", Menlo, Consolas, monospace; background: var(--bg-alt); color: var(--primary-dark); padding: 0.1em 0.35em; border-radius: 3px; font-size: 0.88em; overflow-wrap: anywhere; word-break: break-word; }
pre { background: var(--code-bg); color: var(--code-text); padding: 0.9rem 1.1rem; border-radius: 6px; overflow-x: auto; font-size: 0.86em; line-height: 1.55; font-family: "SF Mono", "JetBrains Mono", Menlo, Consolas, monospace; margin: 0.5rem 0; }
pre code { background: transparent; color: inherit; padding: 0; white-space: pre; font-size: inherit; }
pre .c { color: var(--code-comment); font-style: italic; }
pre .d { color: var(--code-dir); }
pre .s { color: var(--code-string); }
table { border-collapse: collapse; width: 100%; margin: 0.7rem 0; font-size: 0.92em; }
th, td { border: 1px solid var(--border); padding: 0.45rem 0.7rem; text-align: left; vertical-align: top; }
th { background: var(--bg-alt); color: var(--primary-dark); font-weight: 600; }
td code, th code { background: white; }
tr:nth-child(even) td { background: var(--bg-alt); }
.preview { margin: 0.6rem 0 1rem; background: white; border: 1px solid var(--border); border-radius: 6px; padding: 0.8rem; text-align: center; }
.preview svg { display: block; max-width: 100%; height: auto; margin: 0 auto; }
.error-row td:first-child { font-family: "SF Mono", monospace; color: var(--danger); font-weight: 600; white-space: nowrap; font-size: 0.85em; }
@media (max-width: 880px) {
  .container { grid-template-columns: 1fr; padding: 1rem; gap: 1rem; }
  nav.toc { position: static; max-height: none; border-right: none; padding-right: 0; }
}
"""


TOC_SCROLL_SCRIPT = """<script data-tcml-toc-handler>
(function () {
  document.addEventListener("click", function (event) {
    var node = event.target;
    while (node && node !== document.body) {
      if (node.tagName === "A") {
        var href = node.getAttribute("href");
        if (href && href.length > 1 && href.charAt(0) === "#") {
          var el = document.getElementById(href.substring(1));
          if (el) {
            event.preventDefault();
            el.scrollIntoView({ behavior: "smooth", block: "start" });
          }
        }
        return;
      }
      node = node.parentNode;
    }
  });
})();
</script>"""


def render_document(data: dict) -> str:
    sections_html = "\n".join(render_section(s, data) for s in data["sections"])
    labels = data["labels"]
    toc = "\n".join(
        f'<li><a href="#{s["id"]}">{s["num"]}. {s["title"]}</a></li>'
        for s in data["sections"]
    )
    lang_switch_target = data["labels"]["lang_switch_href"]
    lang_switch_label = data["labels"]["lang_switch_label"]
    return f"""<!DOCTYPE html>
<!-- SPDX-License-Identifier: 0BSD -->
<html lang="{data['lang']}">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{data['title']} {labels['title_suffix']} — {data['subtitle']}</title>
<style>{STYLE}</style>
</head>
<body>
<header class="top">
<div class="inner">
<h1>{data['title']}</h1>
<p class="subtitle">{data['subtitle']}</p>
<p class="lang-switch"><a href="{lang_switch_target}">{lang_switch_label}</a></p>
<p class="ext">{labels['extension_label']} <code>{data['extension']}</code></p>
</div>
</header>
<div class="container">
<nav class="toc">
<h3>{labels['toc']}</h3>
<ul>
{toc}
</ul>
</nav>
<main>
{sections_html}
</main>
</div>
{TOC_SCROLL_SCRIPT}
</body>
</html>
"""


def write_output(data: dict, filename: str) -> None:
    out_path = OUT_DIR / filename
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    out = render_document(data)
    out_path.write_text(out)
    print(f"wrote help/output/{filename} ({len(out):,} bytes)")


def main() -> None:
    parser = argparse.ArgumentParser(description="Build TCML help HTML")
    parser.add_argument(
        "--lang",
        choices=["ja", "en", "all"],
        default="all",
        help="generate Japanese, English, or both (default: all)",
    )
    args = parser.parse_args()

    ensure_tchart_built()

    targets: list[tuple[dict, str]] = []
    if args.lang in ("ja", "all"):
        targets.append((DATA_JA, "tcml-format.html"))
    if args.lang in ("en", "all"):
        targets.append((DATA_EN, "tcml-format.en.html"))

    all_samples: list[str] = []
    for data, _ in targets:
        all_samples.extend(collect_sample_tcmls(data))
    batch_render_samples(all_samples)

    for data, filename in targets:
        write_output(data, filename)


if __name__ == "__main__":
    main()
