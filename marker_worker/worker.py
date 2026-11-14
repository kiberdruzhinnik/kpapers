"""Private HTTP worker that converts PDFs with Marker.

The Rust API sends a bounded PDF body to ``POST /extract`` and receives a
page-aware JSON document containing plain text and emphasis runs.  This worker
never returns Marker's HTML: the API applies the same cleanup, classification
and HTML escaping pipeline that it uses for Pdfium output.

The module deliberately imports only the Python standard library at import
time.  Marker and its heavy model dependencies are imported lazily by
``MarkerConverter.load`` so the response contract and HTTP behavior can be
tested without a model installation.
"""

from __future__ import annotations

import io
import json
import math
import os
import re
import signal
import tempfile
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from concurrent.futures import TimeoutError as FutureTimeoutError
from dataclasses import dataclass, field
from html.parser import HTMLParser
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Iterable, Sequence

DEFAULT_MAX_INPUT_BYTES = 25 * 1024 * 1024
DEFAULT_MAX_RESPONSE_BYTES = 8 * 1024 * 1024
DEFAULT_MAX_CONCURRENCY = 1
DEFAULT_EXECUTION_TIMEOUT_SECONDS = 110.0
DEFAULT_MODE = "fast"
# Pdfium stops descending into Form XObjects at this level. If a form remains
# at the boundary, an image may be hidden below it, so completeness is unknown.
PDF_OBJECT_MAX_DEPTH = 32

# Marker's output is block based.  A block becomes one line in the Rust
# pipeline.  Heading sizes are relative to the body size so the existing
# heading detector can recognise Marker section headers without layout
# coordinates.
BODY_SIZE = 12.0
# Marker returns paragraphs, not visual lines.  A block boundary must become a
# paragraph boundary in the shared pipeline even when the block's vertical gap
# is ordinary line spacing.
MIN_BLOCK_GAP = BODY_SIZE * 2.5
HEADING_SIZES = {1: 24.0, 2: 20.0, 3: 17.0, 4: 15.0, 5: 13.5, 6: 13.0}
BLOCK_SIZES = {
    "Caption": 10.0,
    "Footnote": 10.0,
    "PageFooter": 9.0,
    "PageHeader": 9.0,
    "Reference": 10.0,
    "Code": 11.0,
    "Equation": 12.0,
}
TEXT_BLOCK_TYPES = {
    "Caption",
    "Code",
    "Equation",
    "Footnote",
    "Handwriting",
    "ListItem",
    "Reference",
    "SectionHeader",
    "Table",
    "TableCell",
    "Text",
    "TextInlineMath",
}
# Blocks that Marker emits as containers.  Their children are walked instead of
# the container itself.
CONTAINER_BLOCK_TYPES = {
    "Document",
    "FigureGroup",
    "ListGroup",
    "Page",
    "PictureGroup",
    "TableGroup",
}
IGNORED_BLOCK_TYPES = {
    "Diagram",
    "Figure",
    "PageFooter",
    "PageHeader",
    "Picture",
    "TableOfContents",
}
_CONTENT_REF_RE = re.compile(r"<content-ref\b[^>]*>.*?</content-ref>", re.IGNORECASE | re.DOTALL)
_CONTENT_REF_SELF_RE = re.compile(r"<content-ref\b[^>]*/?>", re.IGNORECASE)
_WHITESPACE_RE = re.compile(r"\s+")
_BOLD_FONT_TOKENS = ("bold", "black", "heavy", "semibold", "demibold", "ultrabold")
_ITALIC_FONT_TOKENS = ("italic", "oblique")


class WorkerError(Exception):
    """An expected request or conversion failure."""

    def __init__(self, message: str, status: HTTPStatus = HTTPStatus.UNPROCESSABLE_ENTITY):
        super().__init__(message)
        self.status = status


@dataclass
class StyledRun:
    text: str
    bold: bool = False
    italic: bool = False


@dataclass
class StyledLine:
    runs: list[StyledRun]
    size: float = BODY_SIZE
    gap_before: float = 0.0
    left: float = 0.0


@dataclass
class Block:
    """The subset of Marker's JSON block output this worker needs."""

    block_type: str
    html: str = ""
    bbox: Sequence[float] | None = None
    children: list["Block"] | None = None
    section_hierarchy: dict[int, str] | None = None


@dataclass
class Page:
    lines: list[StyledLine] = field(default_factory=list)


@dataclass
class MarkerDocument:
    page_count: int
    pages: list[Page]

    def to_dict(self) -> dict:
        return {
            "page_count": self.page_count,
            "pages": [
                {
                    "lines": [
                        {
                            "runs": [
                                {"text": run.text, "bold": run.bold, "italic": run.italic}
                                for run in line.runs
                            ],
                            "size": line.size,
                            "gap_before": line.gap_before,
                            "left": line.left,
                        }
                        for line in page.lines
                    ]
                }
                for page in self.pages
            ],
        }


class _BlockHtmlParser(HTMLParser):
    """Turn one Marker block's HTML into styled runs.

    Only text, emphasis, and heading level are retained.  Links, images,
    tables, attributes and every other tag are discarded before the result
    leaves the worker, and the Rust side escapes the text again.
    """

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.runs: list[StyledRun] = []
        self.paragraphs: list[list[StyledRun]] = []
        self.heading_level: int | None = None
        self._bold = 0
        self._italic = 0
        self._heading_depth = 0
        self._ignored_depth = 0

    def handle_starttag(self, tag: str, attrs) -> None:
        tag = tag.lower()
        if tag in ("script", "style"):
            self._ignored_depth += 1
            return
        if self._ignored_depth:
            return
        if tag in ("b", "strong"):
            self._bold += 1
        elif tag in ("i", "em"):
            self._italic += 1
        elif tag in ("h1", "h2", "h3", "h4", "h5", "h6"):
            self._heading_depth += 1
            self.heading_level = int(tag[1])
        elif tag in ("br", "p", "div", "li", "tr", "blockquote", "pre"):
            self._flush_paragraph()
        elif tag in ("td", "th"):
            self._append(" ")

    def handle_startendtag(self, tag: str, attrs) -> None:
        if tag.lower() == "br":
            self._flush_paragraph()

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if tag in ("script", "style"):
            self._ignored_depth = max(0, self._ignored_depth - 1)
            return
        if self._ignored_depth:
            return
        if tag in ("b", "strong"):
            self._bold = max(0, self._bold - 1)
        elif tag in ("i", "em"):
            self._italic = max(0, self._italic - 1)
        elif tag in ("h1", "h2", "h3", "h4", "h5", "h6"):
            self._heading_depth = max(0, self._heading_depth - 1)
            self._flush_paragraph()
        elif tag in ("p", "div", "li", "tr", "blockquote", "pre"):
            self._flush_paragraph()

    def handle_data(self, data: str) -> None:
        if self._ignored_depth:
            return
        self._append(data)

    def _append(self, text: str) -> None:
        text = _WHITESPACE_RE.sub(" ", text)
        if not text:
            return
        bold = self._bold > 0 or self._heading_depth > 0
        italic = self._italic > 0
        if (
            self.runs
            and self.runs[-1].bold == bold
            and self.runs[-1].italic == italic
        ):
            self.runs[-1].text += text
        else:
            self.runs.append(StyledRun(text=text, bold=bold, italic=italic))

    def _flush_paragraph(self) -> None:
        if self.runs:
            self.paragraphs.append(self.runs)
            self.runs = []

    def parsed_paragraphs(self) -> list[list[StyledRun]]:
        return self.paragraphs + ([self.runs] if self.runs else [])


def _clean_html(html: str) -> str:
    html = _CONTENT_REF_RE.sub(" ", html or "")
    html = _CONTENT_REF_SELF_RE.sub(" ", html)
    return html


def _normalise_runs(runs: Iterable[StyledRun]) -> list[StyledRun]:
    cleaned: list[StyledRun] = []
    for run in runs:
        text = _WHITESPACE_RE.sub(" ", run.text)
        if not text:
            continue
        if cleaned and cleaned[-1].bold == run.bold and cleaned[-1].italic == run.italic:
            cleaned[-1].text += text
        else:
            cleaned.append(StyledRun(text=text, bold=run.bold, italic=run.italic))
    if not cleaned:
        return []
    cleaned[0].text = cleaned[0].text.lstrip()
    cleaned[-1].text = cleaned[-1].text.rstrip()
    return [run for run in cleaned if run.text]


def _normalise_text(text: str) -> str:
    return _WHITESPACE_RE.sub(" ", text).strip()


def _has_complete_text_layer(pages: Sequence[dict] | None) -> bool:
    """Whether every page has native text that an OCR-free pass can read.

    If any page has no text layer, it may be an image-only scan.  Such a
    document must go through OCR; an OCR-disabled pass could silently omit the
    scanned content.
    """

    if not pages:
        return False
    for page in pages:
        if not any(
            _normalise_text(
                "".join(str(span.get("text") or "") for span in line.get("spans") or [])
            )
            for block in page.get("blocks") or []
            for line in block.get("lines") or []
        ):
            return False
    return True


def _pdf_image_scan(data: bytes) -> tuple[bool, bool]:
    """Return whether unverified graphics exist and traversal was truncated.

    Without running OCR the worker cannot prove an embedded image contains no
    text, and vector paths can contain outlined lettering absent from the
    text layer. Images and paths therefore make native text incomplete,
    including small regions on a page that also has native text. A form at the
    traversal boundary means the scan cannot establish whether more objects
    remain below it.
    """

    import pypdfium2 as pdfium
    from pypdfium2 import raw

    document = pdfium.PdfDocument(data)
    try:
        image_type = int(raw.FPDF_PAGEOBJ_IMAGE)
        path_type = int(raw.FPDF_PAGEOBJ_PATH)
        form_type = int(raw.FPDF_PAGEOBJ_FORM)
        has_unverified_graphics = False
        traversal_truncated = False
        for page in document:
            for item in page.get_objects(max_depth=PDF_OBJECT_MAX_DEPTH):
                item_type = int(getattr(item, "type", -1))
                if item_type in (image_type, path_type):
                    has_unverified_graphics = True
                if (
                    item_type == form_type
                    and int(getattr(item, "level", PDF_OBJECT_MAX_DEPTH))
                    >= PDF_OBJECT_MAX_DEPTH - 1
                ):
                    traversal_truncated = True
        return has_unverified_graphics, traversal_truncated
    finally:
        document.close()


def _has_scanned_images(data: bytes) -> bool:
    has_images, traversal_truncated = _pdf_image_scan(data)
    return has_images or traversal_truncated


def _native_text_layer_status(
    data: bytes, pages: Sequence[dict] | None
) -> tuple[bool, bool]:
    """Whether an OCR-free pass can read the complete document.

    Every page needs native text and no page may contain an image or vector
    path whose text content cannot be verified. If completeness cannot be
    established, the caller must use OCR rather than risk omitting scanned
    content. The second return value reports a traversal limit that makes the
    PDF structure itself unverifiable and requires a safe failure.
    """

    try:
        has_images, traversal_truncated = _pdf_image_scan(data)
    except Exception:
        return False, False
    return (
        _has_complete_text_layer(pages) and not has_images and not traversal_truncated,
        traversal_truncated,
    )


def _native_text_layer_complete(data: bytes, pages: Sequence[dict] | None) -> bool:
    return _native_text_layer_status(data, pages)[0]


def _pdftext_bbox(item: dict) -> tuple[float, float, float, float] | None:
    bbox = item.get("bbox")
    if not bbox or len(bbox) < 4:
        return None
    try:
        values = tuple(float(value) for value in bbox[:4])
    except (TypeError, ValueError):
        return None
    if not all(math.isfinite(value) for value in values):
        return None
    return values


def _font_emphasis(font: dict | None) -> tuple[bool, bool]:
    """Infer bold/italic from a PDF text-layer font description."""

    if not isinstance(font, dict):
        return False, False
    name = str(font.get("name") or "").lower()
    bold = any(token in name for token in _BOLD_FONT_TOKENS)
    italic = any(token in name for token in _ITALIC_FONT_TOKENS)
    try:
        flags = int(font.get("flags") or 0)
    except (TypeError, ValueError):
        flags = 0
    # PDF font descriptor flags: bit 19 is ForceBold, bit 7 is Italic.
    bold = bold or bool(flags & (1 << 18))
    italic = italic or bool(flags & (1 << 6))
    try:
        weight = int(font.get("weight") or 0)
    except (TypeError, ValueError):
        weight = 0
    bold = bold or weight >= 600
    return bold, italic


def _pdftext_lines_within(
    page: dict | None, box: tuple[float, float, float, float] | None
) -> list[tuple[list[StyledRun], tuple[float, float, float, float]]]:
    """Return the PDF text-layer lines that fall inside a Marker block.

    Fast-mode Marker emits an indented region as one ``ListGroup`` paragraph
    and joins its source lines with spaces.  The raw text layer still has the
    line geometry and font names, so paragraph boundaries and emphasis can be
    restored before the response leaves the worker.
    """

    if not page or box is None:
        return []
    x1, y1, x2, y2 = box
    lines = []
    for block in page.get("blocks") or []:
        for line in block.get("lines") or []:
            line_box = _pdftext_bbox(line)
            if line_box is None:
                continue
            center_x = (line_box[0] + line_box[2]) / 2
            center_y = (line_box[1] + line_box[3]) / 2
            if not (x1 <= center_x <= x2 and y1 <= center_y <= y2):
                continue
            runs = []
            for span in line.get("spans") or []:
                text = _WHITESPACE_RE.sub(" ", str(span.get("text") or ""))
                if not text:
                    continue
                bold, italic = _font_emphasis(span.get("font"))
                if runs and runs[-1].bold == bold and runs[-1].italic == italic:
                    runs[-1].text += text
                else:
                    runs.append(StyledRun(text=text, bold=bold, italic=italic))
            runs = _normalise_runs(runs)
            text = "".join(run.text for run in runs)
            if text:
                lines.append((runs, line_box))
    return lines


def _run_styles(runs: Sequence[StyledRun]) -> list[tuple[bool, bool]]:
    styles: list[tuple[bool, bool]] = []
    for run in runs:
        styles.extend([(run.bold, run.italic)] * len(run.text))
    return styles


def _split_paragraph_by_pdftext(
    runs: list[StyledRun],
    pdftext_lines: Sequence[Sequence[StyledRun]],
) -> list[list[StyledRun]] | None:
    """Split or re-style one paragraph using the PDF text-layer lines.

    Marker's fast mode can merge an indented region into a single plain-text
    ``ListGroup`` paragraph, losing both the source line boundaries and its
    emphasis.  The PDF text layer provides both.  Marker's own emphasis is
    retained as well, so a paragraph that Marker formatted is not downgraded
    when the text layer has no useful font names.
    """

    if not pdftext_lines:
        return None
    full_text = "".join(run.text for run in runs)
    line_texts = ["".join(run.text for run in line) for line in pdftext_lines]
    if " ".join(line_texts) != full_text:
        return None
    marker_styles = _run_styles(runs)
    pdf_styles: list[tuple[bool, bool]] = []
    for index, line_runs in enumerate(pdftext_lines):
        if index:
            # Account for the joining space that Marker inserted.
            pdf_styles.append((False, False))
        pdf_styles.extend(_run_styles(line_runs))
    if len(pdf_styles) != len(full_text):
        return None

    split: list[list[StyledRun]] = []
    offset = 0
    for text in line_texts:
        combined = []
        for position, char in enumerate(text):
            marker_bold, marker_italic = marker_styles[offset + position]
            pdf_bold, pdf_italic = pdf_styles[offset + position]
            combined.append(StyledRun(
                text=char,
                bold=marker_bold or pdf_bold,
                italic=marker_italic or pdf_italic,
            ))
        normalized = _normalise_runs(combined)
        if not normalized:
            return None
        split.append(normalized)
        offset += len(text) + 1
    return split


def _heading_level(block: Block, parsed: _BlockHtmlParser) -> int | None:
    if parsed.heading_level:
        return parsed.heading_level
    if block.block_type != "SectionHeader":
        return None
    hierarchy = block.section_hierarchy or {}
    levels = [int(level) for level in hierarchy if str(level).isdigit()]
    return min(levels) if levels else 1


def _block_size(block: Block, heading_level: int | None) -> float:
    if heading_level is not None:
        return HEADING_SIZES.get(heading_level, BODY_SIZE)
    return BLOCK_SIZES.get(block.block_type, BODY_SIZE)


def _bbox(block: Block) -> tuple[float, float, float, float] | None:
    bbox = block.bbox
    if not bbox or len(bbox) < 4:
        return None
    try:
        x1, y1, x2, y2 = (float(value) for value in bbox[:4])
    except (TypeError, ValueError):
        return None
    if not all(math.isfinite(value) for value in (x1, y1, x2, y2)):
        return None
    return x1, y1, x2, y2


def _leaf_blocks(block: Block) -> Iterable[Block]:
    if block.block_type in IGNORED_BLOCK_TYPES:
        return
    if block.block_type in CONTAINER_BLOCK_TYPES or block.children:
        children = block.children or []
        if children:
            for child in children:
                yield from _leaf_blocks(child)
            return
        # Marker sometimes emits a container such as ListGroup with the text
        # inline and no child blocks.  Keep that text instead of dropping the
        # whole block; empty Page/Document containers still produce no runs
        # because their content references are removed before parsing.
        if block.html.strip():
            yield block
        return
    if block.block_type in TEXT_BLOCK_TYPES:
        yield block


def blocks_to_document(
    pages: Sequence[Block], pdftext_pages: Sequence[dict] | None = None
) -> MarkerDocument:
    """Convert Marker page blocks into the worker's page/line/run contract."""

    output_pages: list[Page] = []
    for page_index, page in enumerate(pages):
        pdftext_page = (
            pdftext_pages[page_index]
            if pdftext_pages is not None and page_index < len(pdftext_pages)
            else None
        )
        lines: list[StyledLine] = []
        previous_bottom: float | None = None
        for block in _leaf_blocks(page):
            parser = _BlockHtmlParser()
            try:
                parser.feed(_clean_html(block.html))
                parser.close()
            except Exception:
                continue
            paragraphs = [
                runs for raw in parser.parsed_paragraphs()
                if (runs := _normalise_runs(raw))
            ]
            if not paragraphs:
                continue
            heading_level = _heading_level(block, parser)
            if block.block_type == "SectionHeader":
                # Marker section headers are semantically headings even when
                # its HTML does not wrap them in <b> or <strong>.
                paragraphs = [[StyledRun(text=run.text, bold=True, italic=run.italic) for run in runs] for runs in paragraphs]
            size = _block_size(block, heading_level)
            box = _bbox(block)
            left = box[0] if box is not None else 0.0
            gap_before = 0.0
            if box is not None and previous_bottom is not None:
                gap_before = max(0.0, box[1] - previous_bottom)
            if box is not None:
                previous_bottom = max(box[1], box[3])
            if lines:
                gap_before = max(gap_before, MIN_BLOCK_GAP)
            split_lines = None
            if len(paragraphs) == 1 and box is not None:
                pdftext_lines = _pdftext_lines_within(pdftext_page, box)
                split = _split_paragraph_by_pdftext(
                    paragraphs[0], [line_runs for line_runs, _ in pdftext_lines]
                )
                if split is not None:
                    split_lines = list(zip(split, [line_box for _, line_box in pdftext_lines]))
            if split_lines:
                for index, (runs, line_box) in enumerate(split_lines):
                    if index == 0:
                        line_gap = gap_before
                    else:
                        previous_box = split_lines[index - 1][1]
                        line_height = max(
                            previous_box[3] - previous_box[1],
                            line_box[3] - line_box[1],
                        )
                        # A blank line between source lines marks a paragraph
                        # boundary; keep wrapped lines joined.
                        paragraph_break = (
                            line_height > 0
                            and line_box[1] - previous_box[1] > line_height * 1.4
                        )
                        line_gap = (
                            MIN_BLOCK_GAP
                            if paragraph_break
                            else max(0.0, line_box[1] - previous_box[3])
                        )
                    lines.append(StyledLine(
                        runs=runs,
                        size=size,
                        gap_before=line_gap,
                        left=max(0.0, line_box[0]),
                    ))
            else:
                for index, runs in enumerate(paragraphs):
                    lines.append(StyledLine(
                        runs=runs,
                        size=size,
                        gap_before=gap_before if index == 0 else MIN_BLOCK_GAP,
                        left=max(0.0, left),
                    ))
        output_pages.append(Page(lines=lines))
    return MarkerDocument(page_count=len(output_pages), pages=output_pages)


class _NormalizeEmptyPageChildren:
    """Repair Marker's ``None`` child list for an empty page.

    Marker 2.2.0's document builder leaves ``page.children`` as ``None`` when a
    page has no output blocks (for example a page containing only a filtered
    page number).  Its section-header processor then fails while iterating the
    page.  Normalising the value to an empty list before the processors run
    keeps empty pages in the document without skipping the failing document.
    """

    def __call__(self, document) -> None:
        for page in getattr(document, "pages", None) or []:
            if getattr(page, "children", None) is None:
                page.children = []


class MarkerConverter:
    """Loads Marker once and converts PDF bytes on demand."""

    def __init__(self, mode: str | None = None) -> None:
        self.mode = (mode or os.environ.get("MARKER_MODE") or DEFAULT_MODE).strip()
        self._converter = None
        self._native_converter = None
        self._ocr_converter = None
        self._artifacts = None
        self._lock = threading.Lock()
        self._ready = False

    @property
    def ready(self) -> bool:
        return self._ready

    def load(self) -> None:
        """Import Marker and load its models.  Called once before serving."""

        if self._ready:
            return
        with self._lock:
            if self._ready:
                return
            try:
                from marker.converters.pdf import PdfConverter
                from marker.models import create_model_dict
            except Exception as error:  # pragma: no cover - depends on install
                raise RuntimeError(
                    "Marker is not installed; install the pinned marker_worker requirements"
                ) from error
            try:
                artifacts = create_model_dict()
                self._artifacts = artifacts

                def build(mode: str, *, force_ocr: bool = False):
                    converter = PdfConverter(
                        artifact_dict=artifacts,
                        renderer="marker.renderers.json.JSONRenderer",
                        config={
                            "mode": mode,
                            "disable_ocr": mode == "no_ocr",
                            "force_ocr": force_ocr,
                            "paginate_output": False,
                        },
                    )
                    converter.processor_list.insert(0, _NormalizeEmptyPageChildren())
                    return converter

                if self.mode == "fast":
                    # A fully native PDF should use its text layer rather than
                    # OCR. Uncertain PDFs use a fast converter with OCR forced,
                    # so Marker's automatic decision cannot omit scanned text.
                    # Both converters share the loaded model weights.
                    self._ocr_converter = build("fast", force_ocr=True)
                    self._converter = self._ocr_converter
                    self._native_converter = build("no_ocr")
                else:
                    self._converter = build(self.mode)
                    if self.mode != "no_ocr":
                        self._ocr_converter = build("fast", force_ocr=True)
            except Exception as error:  # pragma: no cover - depends on model files
                raise RuntimeError("Marker model initialization failed") from error
            self._ready = True

    def _convert_with(self, converter, data: bytes):
        return converter(io.BytesIO(data))

    def _pdftext_pages(self, data: bytes) -> Sequence[dict] | None:
        """Read the source PDF text layer to restore merged line boundaries.

        This is best effort: scanned pages and PDFs without a usable text layer
        keep Marker's block-level output.
        """

        try:
            from pdftext.extraction import dictionary_output
        except Exception:  # pragma: no cover - depends on install
            return None
        temp_file = None
        try:
            temp_file = tempfile.NamedTemporaryFile(suffix=".pdf", delete=False)
            temp_file.write(data)
            temp_file.close()
            return dictionary_output(temp_file.name, sort=False, disable_links=True)
        except Exception:
            return None
        finally:
            if temp_file is not None:
                try:
                    os.unlink(temp_file.name)
                except OSError:
                    pass

    def _native_text_layer_complete(
        self, data: bytes, pages: Sequence[dict] | None
    ) -> bool:
        return self._native_text_layer_status(data, pages)[0]

    def _native_text_layer_status(
        self, data: bytes, pages: Sequence[dict] | None
    ) -> tuple[bool, bool]:
        return _native_text_layer_status(data, pages)

    def convert(self, data: bytes) -> MarkerDocument:
        if not self._ready or self._converter is None:
            raise WorkerError("worker is not ready", HTTPStatus.SERVICE_UNAVAILABLE)
        pdftext_pages = self._pdftext_pages(data)
        converter = self._converter
        native_complete, traversal_truncated = self._native_text_layer_status(
            data, pdftext_pages
        )
        if traversal_truncated:
            raise WorkerError(
                "PDF form traversal limit prevents safe extraction"
            )
        if self.mode == "no_ocr" and not native_complete:
            # Explicitly disabling OCR is only safe when the native text
            # layer is complete.  Fail closed rather than return a document
            # that may omit scanned content.
            raise WorkerError(
                "OCR-disabled mode cannot establish a complete extraction for this PDF"
            )
        if self.mode == "fast" and self._native_converter is not None and native_complete:
            # Use OCR unless the text layer is established as complete.  A
            # page can have a native header and scanned content, so native
            # text on every page is not sufficient on its own.
            converter = self._native_converter
        elif not native_complete and self.mode != "no_ocr":
            if self._ocr_converter is None:
                raise WorkerError("OCR is required but no OCR converter is available")
            converter = self._ocr_converter
        try:
            rendered = self._convert_with(converter, data)
        except Exception as error:
            # Never retry a document without OCR: an image-only page could be
            # silently omitted.  Preserve the previous good cache by failing
            # closed.
            raise WorkerError("Marker could not convert this PDF") from error
        pages = list(getattr(rendered, "children", None) or [])
        document = blocks_to_document(
            [
                Block(
                    block_type=getattr(page, "block_type", "Page"),
                    html=getattr(page, "html", ""),
                    bbox=getattr(page, "bbox", None),
                    children=_convert_children(getattr(page, "children", None)),
                    section_hierarchy=getattr(page, "section_hierarchy", None),
                )
                for page in pages
            ],
            pdftext_pages,
        )
        if document.page_count == 0:
            raise WorkerError("Marker returned no pages")
        return document


def _convert_children(children) -> list[Block] | None:
    if not children:
        return None
    converted = []
    for child in children:
        converted.append(
            Block(
                block_type=str(getattr(child, "block_type", "")),
                html=getattr(child, "html", "") or "",
                bbox=getattr(child, "bbox", None),
                children=_convert_children(getattr(child, "children", None)),
                section_hierarchy=getattr(child, "section_hierarchy", None),
            )
        )
    return converted


class WorkerConfig:
    def __init__(
        self,
        max_input_bytes: int = DEFAULT_MAX_INPUT_BYTES,
        max_response_bytes: int = DEFAULT_MAX_RESPONSE_BYTES,
        max_concurrency: int = DEFAULT_MAX_CONCURRENCY,
        execution_timeout: float = DEFAULT_EXECUTION_TIMEOUT_SECONDS,
    ) -> None:
        self.max_input_bytes = max_input_bytes
        self.max_response_bytes = max_response_bytes
        self.max_concurrency = max_concurrency
        self.execution_timeout = execution_timeout

    @classmethod
    def from_env(cls) -> "WorkerConfig":
        return cls(
            max_input_bytes=_positive_int(
                os.environ.get("MARKER_WORKER_MAX_INPUT_BYTES"), DEFAULT_MAX_INPUT_BYTES
            ),
            max_response_bytes=_positive_int(
                os.environ.get("MARKER_WORKER_MAX_RESPONSE_BYTES"),
                DEFAULT_MAX_RESPONSE_BYTES,
            ),
            max_concurrency=_positive_int(
                os.environ.get("MARKER_WORKER_MAX_CONCURRENCY"), DEFAULT_MAX_CONCURRENCY
            ),
            execution_timeout=_positive_float(
                os.environ.get("MARKER_WORKER_EXECUTION_TIMEOUT_SECONDS"),
                DEFAULT_EXECUTION_TIMEOUT_SECONDS,
            ),
        )


def _positive_int(value: str | None, default: int) -> int:
    try:
        parsed = int(value) if value is not None else default
    except (TypeError, ValueError):
        return default
    return parsed if parsed > 0 else default


def _positive_float(value: str | None, default: float) -> float:
    try:
        parsed = float(value) if value is not None else default
    except (TypeError, ValueError):
        return default
    return parsed if parsed > 0 and math.isfinite(parsed) else default


class WorkerServer(ThreadingHTTPServer):
    daemon_threads = True
    allow_reuse_address = True

    def __init__(self, address, converter, config: WorkerConfig, restart_on_timeout=None):
        super().__init__(address, MarkerRequestHandler)
        self.converter = converter
        self.config = config
        self.restart_on_timeout = restart_on_timeout or self._restart_process
        self.executor = ThreadPoolExecutor(
            max_workers=config.max_concurrency, thread_name_prefix="marker"
        )
        self.slots = threading.BoundedSemaphore(config.max_concurrency)

    def _restart_process(self) -> None:
        # Marker/PyTorch conversion runs in a thread and cannot be safely
        # interrupted. Exit non-zero after the 504 reaches the caller; the
        # container's restart policy reloads the model and releases all slots.
        timer = threading.Timer(0.5, os._exit, args=(1,))
        timer.daemon = True
        timer.start()

    def server_close(self) -> None:
        self.executor.shutdown(wait=False, cancel_futures=True)
        super().server_close()


class MarkerRequestHandler(BaseHTTPRequestHandler):
    server_version = "kpapers-marker-worker"
    protocol_version = "HTTP/1.1"

    @property
    def worker(self) -> WorkerServer:
        return self.server  # type: ignore[return-value]

    def log_message(self, format: str, *args) -> None:
        # Do not log request bodies or extracted document contents.
        super().log_message(format, *args)

    def _json(self, status: HTTPStatus, payload: dict) -> None:
        body = json.dumps(payload, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _error(self, status: HTTPStatus, message: str) -> None:
        self._json(status, {"error": message})

    def _handle_timeout(self) -> None:
        # A timed-out conversion cannot be cancelled safely, so the worker must
        # restart to release its slot.  Schedule the restart before writing the
        # 504: a client that disconnects or never reads the response must not
        # keep a stuck conversion holding extraction capacity.
        self.worker.restart_on_timeout()
        try:
            self._error(
                HTTPStatus.GATEWAY_TIMEOUT,
                "conversion exceeded the execution time limit",
            )
            self.wfile.flush()
        except OSError:
            # The client may already be gone; the restart is already scheduled.
            pass

    def do_GET(self) -> None:
        if self.path != "/healthz":
            self._error(HTTPStatus.NOT_FOUND, "not found")
            return
        ready = bool(getattr(self.worker.converter, "ready", False))
        self._json(
            HTTPStatus.OK if ready else HTTPStatus.SERVICE_UNAVAILABLE,
            {"status": "ok" if ready else "starting", "ready": ready},
        )

    def do_POST(self) -> None:
        if self.path != "/extract":
            self._error(HTTPStatus.NOT_FOUND, "not found")
            return
        content_type = (self.headers.get("Content-Type") or "").split(";", 1)[0].strip().lower()
        if content_type != "application/pdf":
            self._error(HTTPStatus.UNSUPPORTED_MEDIA_TYPE, "expected application/pdf")
            return
        length_header = self.headers.get("Content-Length")
        try:
            length = int(length_header) if length_header is not None else -1
        except ValueError:
            length = -1
        if length < 0:
            self._error(HTTPStatus.LENGTH_REQUIRED, "Content-Length is required")
            return
        if length > self.worker.config.max_input_bytes:
            self._error(HTTPStatus.REQUEST_ENTITY_TOO_LARGE, "PDF exceeds the input size limit")
            return
        data = self.rfile.read(length)
        if len(data) != length:
            self._error(HTTPStatus.BAD_REQUEST, "request body is incomplete")
            return

        if not self.worker.slots.acquire(blocking=False):
            self._error(HTTPStatus.SERVICE_UNAVAILABLE, "worker is busy")
            return

        future = self.worker.executor.submit(self.worker.converter.convert, data)
        slots = self.worker.slots
        future.add_done_callback(lambda _future: slots.release())
        try:
            document = future.result(timeout=self.worker.config.execution_timeout)
        except FutureTimeoutError:
            self._handle_timeout()
            return
        except WorkerError as error:
            self._error(error.status, str(error))
            return
        except Exception:
            self._error(HTTPStatus.INTERNAL_SERVER_ERROR, "conversion failed")
            return

        payload = document.to_dict()
        body = json.dumps(payload, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        if len(body) > self.worker.config.max_response_bytes:
            self._error(
                HTTPStatus.REQUEST_ENTITY_TOO_LARGE,
                "response exceeds the output size limit",
            )
            return
        self.send_response(HTTPStatus.OK)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)


def serve(converter, config: WorkerConfig, host: str, port: int) -> None:
    server = WorkerServer((host, port), converter, config)
    try:
        server.serve_forever()
    finally:
        server.server_close()


def main() -> int:
    config = WorkerConfig.from_env()
    host = os.environ.get("MARKER_WORKER_HOST", "0.0.0.0")
    port = _positive_int(os.environ.get("MARKER_WORKER_PORT"), 8000)
    converter = MarkerConverter()
    started = time.monotonic()
    try:
        converter.load()
    except Exception as error:
        print(f"marker worker startup failed: {error}", flush=True)
        return 1
    print(
        f"marker worker ready in {time.monotonic() - started:.1f}s "
        f"(mode={converter.mode}, concurrency={config.max_concurrency})",
        flush=True,
    )

    def stop(_signum, _frame):
        raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, stop)
    try:
        serve(converter, config, host, port)
    except KeyboardInterrupt:
        return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
