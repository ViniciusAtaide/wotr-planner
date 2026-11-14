#!/usr/bin/env python3
"""Renders the saved guide pages (guide-cache/*.html) to plain text. Called by scripts/guide-text.sh; standard library only.

Reads the content of each page (`div#faqwrap`, minus its hidden table of contents) in reading order. The saved pages
carry navigation, scripts, styles, ads and a cookie banner around it; none of those reach the text.

Text format (guide-cache/txt/<page>.txt):
  - A heading is one line, `<#...> <heading>  [<page>#<anchor>]`. The number of `#` is the HTML level minus one
    (`h2` the page title -> `#`, `h3` an archetype or category -> `##`, `h4` a feature -> `###`, `h5` an option -> `####`).
    The anchor is the one the game data links to (`guide: "barbarian#invulnerable-rager"`).
  - A table is one line per row, cells joined by ` | `, header row first. A cell that spans columns is followed by empty
    cells, a cell that spans rows leaves empty cells below, so a column is always the same position. The source's own
    `-` placeholders are kept as they are; a really empty cell stays empty.
  - A list item is one line starting `- ` (two spaces of indent per nesting level). A quote line starts `> `.
  - A paragraph is one line, with its `<br>` line breaks written ` // ` (so `Prerequisites: // Strength: 13`).
  - A table inside a table cell is written inside that cell as `[row ;; row]`. A cell holding paragraphs or a list
    writes them ` // ` apart (list items ` // - `).
  - A footnote mark (`<sup>`) is written `^<mark>`. Links keep their text only; images keep their alt text in `[img: ...]`.
  - Blank lines separate blocks and carry no meaning.
Index (guide-cache/txt/INDEX.txt): one line per section of every rendered page, `<page>#<anchor>\\t<heading>\\t<file>:<line>` (the page title, which has no anchor, is just `<page>`),
the file relative to the repository root.
"""

import re
import sys
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CACHE = ROOT / "guide-cache"
OUT = CACHE / "txt"
HEADINGS = {"h1": 1, "h2": 1, "h3": 2, "h4": 3, "h5": 4, "h6": 5}
BLOCKS = {"p", "ul", "ol", "li", "blockquote", "table", "h1", "h2", "h3", "h4", "h5", "h6", "div", "hr"}
SKIP = {"script", "style", "noscript", "iframe", "svg", "template", "head"}
VOID = {"br", "img", "hr", "meta", "link", "input", "area", "base", "col", "source", "wbr"}
WS = re.compile(r"\s+")


def clean(text):
    return WS.sub(" ", text.replace("\xa0", " ")).strip()


class Page(HTMLParser):
    def __init__(self, name):
        super().__init__(convert_charrefs=True)
        self.name = name
        self.lines = []  # output lines, in order
        self.sections = []  # (line index, anchor, heading text)
        self.tables = 0
        self.stack = []  # open non-void tags
        self.wrap = None  # stack depth just below div#faqwrap once inside it
        self.skip = None  # stack depth at which a skipped element opened
        self.buf = []
        self.kind = None  # what the pending buffer is: text, li, heading
        self.lists = []  # nesting of ul/ol
        self.quote = 0
        self.heading = None  # (level, anchor)
        self.table = None  # {"rows", "row", "cell", "span"}
        self.outer = []  # tables that wait while a table inside one of their cells is read

    # --- output -----------------------------------------------------------------------------------------------
    def blank(self):
        if self.lines and self.lines[-1] != "":
            self.lines.append("")

    def flush(self):
        text = clean("".join(self.buf)).replace(" // //", " //").strip(" /")
        text = re.sub(r"^(// ?)+|( ?//)+$", "", text).strip()
        kind, self.buf, self.kind = self.kind, [], None
        if kind == "heading":
            level, anchor = self.heading
            self.heading = None
            self.blank()
            tag = f"{self.name}#{anchor}" if anchor else self.name
            self.sections.append((len(self.lines), tag, text))
            self.lines.append(f"{'#' * level} {text}  [{tag}]")
            return
        if not text:
            return
        if kind == "li":
            text = "  " * (len(self.lists) - 1) + "- " + text
        if self.quote:
            text = "> " + text
        self.lines.append(text)

    # --- parser events ----------------------------------------------------------------------------------------
    def inside(self):
        return self.wrap is not None and self.skip is None

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag not in VOID:
            self.stack.append(tag)
        depth = len(self.stack)
        if self.wrap is None:
            if tag == "div" and a.get("id") == "faqwrap":
                self.wrap = depth - 1
            return
        if self.skip is not None:
            return
        classes = (a.get("class") or "").split()
        if tag in SKIP or (tag == "div" and "ftoc" in classes) or "hidead" in classes:
            self.skip = depth - 1
            return
        t = self.table
        if tag == "table":
            if t is not None:  # a table inside a cell: the outer one waits on the stack
                self.outer.append(t)
            else:
                self.flush()
                self.blank()
            self.table = {"rows": [], "row": None, "cell": None, "span": {}}
            self.tables += 1
        elif t is not None:
            if tag == "tr":
                t["row"] = []
                self.fill_spans(t)
            elif tag in ("td", "th"):
                t["cell"] = []
                t["cols"] = int(a.get("colspan") or 1)
                t["rows_"] = int(a.get("rowspan") or 1)
            elif tag in ("br",) + tuple(BLOCKS - {"table"}) and t["cell"] is not None:
                # a cell holding paragraphs or a list: its blocks become ` // ` separated pieces, list items `- ` ones
                t["cell"].append(" // - " if tag == "li" else " // ")
            elif tag == "sup" and t["cell"] is not None:
                t["cell"].append("^")
            elif tag == "img" and t["cell"] is not None and a.get("alt"):
                t["cell"].append(f"[img: {clean(a['alt'])}]")
        elif tag in HEADINGS:
            self.flush()
            self.kind, self.heading = "heading", (HEADINGS[tag], None)
        elif tag in ("ul", "ol"):
            self.flush()
            self.lists.append(tag)
        elif tag == "li":
            self.flush()
            self.kind = "li"
        elif tag == "blockquote":
            self.flush()
            self.quote += 1
        elif tag in BLOCKS:
            self.flush()
        elif tag == "br":
            self.buf.append(" // ")
        elif tag == "sup":
            self.buf.append("^")
        elif tag == "img" and a.get("alt"):
            self.buf.append(f"[img: {clean(a['alt'])}]")
        if tag == "a" and self.heading is not None and not self.heading[1]:
            anchor = a.get("id") or a.get("name")
            if anchor:
                self.heading = (self.heading[0], anchor)
        if self.kind is None and not self.table and tag in ("p", "div", "blockquote"):
            self.kind = "text"

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)

    def handle_endtag(self, tag):
        if tag in VOID or tag not in self.stack:
            return
        while self.stack:
            top = self.stack.pop()
            depth = len(self.stack)
            self.on_close(top, depth)
            if top == tag:
                break

    def on_close(self, tag, depth):
        if self.wrap is None:
            return
        if self.skip is not None:
            if depth == self.skip:
                self.skip = None
            return
        if tag == "div" and depth == self.wrap:
            self.flush()
            self.wrap = None
            return
        t = self.table
        if t is not None:
            if tag in ("td", "th") and t["cell"] is not None:
                self.end_cell(t)
            elif tag == "tr" and t["row"] is not None:
                if t["cell"] is not None:
                    self.end_cell(t)
                self.fill_spans(t)
                t["rows"].append(t["row"])
                t["row"] = None
            elif tag == "table":
                self.end_table(t)
            return
        if tag in HEADINGS or tag in ("p", "li", "div"):
            self.flush()
        elif tag in ("ul", "ol"):
            self.flush()
            if self.lists:
                self.lists.pop()
            if not self.lists:
                self.blank()
        elif tag == "blockquote":
            self.flush()
            self.quote = max(0, self.quote - 1)
            if not self.quote:
                self.blank()

    def handle_data(self, data):
        if not self.inside():
            return
        t = self.table
        if t is not None:
            if t["cell"] is not None:
                t["cell"].append(data)
            return
        if self.kind is None and data.strip():
            self.kind = "text"
        if self.kind is not None:
            self.buf.append(data)

    # --- tables -----------------------------------------------------------------------------------------------
    def fill_spans(self, t):
        """Empty cells where an earlier row's rowspan reaches into this row's next free column."""
        row = t["row"]
        while row is not None and t["span"].get(len(row), 0) > 0:
            col = len(row)
            t["span"][col] -= 1
            row.append("")

    def end_cell(self, t):
        text = clean("".join(t["cell"])).replace("|", "\\|").replace("\x01", "|")
        text = re.sub(r"( ?//)+ ?$|^( ?//)+ ?", "", text).strip()
        text = re.sub(r"( //)+ //", " //", text)
        t["cell"] = None
        row = t["row"]
        if row is None:
            return
        col = len(row)
        row.append(text)
        for k in range(1, t["cols"]):
            row.append("")
            if t["rows_"] > 1:
                t["span"][col + k] = t["rows_"] - 1
        if t["rows_"] > 1:
            t["span"][col] = t["rows_"] - 1
        self.fill_spans(t)

    def end_table(self, t):
        rows = [" | ".join(row) for row in t["rows"]]
        if self.outer:
            # Nested table: one piece of its cell, rows between `;;`, e.g. `[HP | AC ;; 5 | 12]`.
            self.table = self.outer.pop()
            if self.table["cell"] is not None:
                self.table["cell"].append(" // [" + " ;; ".join(rows).replace(" | ", " \x01 ") + "] // ")
            return
        self.table = None
        self.lines.extend(rows)
        self.blank()

    def render(self):
        self.flush()
        while self.lines and self.lines[-1] == "":
            self.lines.pop()
        return self.lines


def render_page(path):
    page = Page(path.stem)
    page.feed(path.read_text(encoding="utf-8", errors="replace"))
    page.close()
    return page, page.render()


def main(argv):
    pages = sorted(CACHE.glob("*.html"))
    if not pages:
        print(f"guide-cache has no pages: save them with scripts/save-guide-pages.sh ({CACHE}/*.html)", file=sys.stderr)
        return 1
    if argv:
        by_name = {p.stem: p for p in pages}
        unknown = [n for n in argv if n.removesuffix(".html") not in by_name]
        if unknown:
            print(f"no saved page named {', '.join(unknown)}; saved: {' '.join(by_name)}", file=sys.stderr)
            return 2
        pages = [by_name[n.removesuffix(".html")] for n in argv]
    OUT.mkdir(parents=True, exist_ok=True)
    if not argv:
        keep = {p.stem for p in pages}
        for old in OUT.glob("*.txt"):
            if old.stem not in keep and old.name != "INDEX.txt":
                old.unlink()
    total_sections = total_tables = 0
    for path in pages:
        page, lines = render_page(path)
        if not page.sections:
            print(f"{path.name}: no content found in div#faqwrap; is it a saved guide page?", file=sys.stderr)
            return 1
        (OUT / f"{path.stem}.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
        total_sections += len(page.sections)
        total_tables += page.tables
        print(f"{path.stem}: {len(page.sections)} sections, {page.tables} tables")
    # The index covers every rendered page in the folder, found by each file's heading lines, so a run of some pages
    # keeps the others' entries.
    entries = []
    for txt in sorted(OUT.glob("*.txt")):
        if txt.name == "INDEX.txt":
            continue
        head = re.compile(r"^#+ (.*)  \[(" + re.escape(txt.stem) + r"(?:#[^\]\s]*)?)\]$")
        for n, line in enumerate(txt.read_text(encoding="utf-8").split("\n"), 1):
            m = head.match(line)
            if m:
                entries.append(f"{m.group(2)}\t{m.group(1)}\t{txt.relative_to(ROOT)}:{n}")
    (OUT / "INDEX.txt").write_text("\n".join(entries) + "\n", encoding="utf-8")
    print(f"total: {len(pages)} pages, {total_sections} sections, {total_tables} tables -> {OUT.relative_to(ROOT)}/")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
