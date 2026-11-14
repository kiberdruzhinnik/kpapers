"""Contract and HTTP tests for the Marker worker.

Run from the repository root with:

    python3 -m unittest discover -s marker_worker/tests -v

These tests do not import Marker or download models.  A fake converter keeps
the page/line/run contract and the worker's resource boundaries testable on a
plain Python installation.
"""

from __future__ import annotations

import json
import importlib.util
import os
import socket
import subprocess
import sys
import threading
import time
from types import ModuleType
import unittest
import urllib.error
import urllib.request
from unittest import mock

from marker_worker import worker

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Run the production server boundary in a separate process.  The default
# restart hook calls os._exit, so this cannot run inside the test process.
WORKER_PROCESS = r"""
import sys
import time

from marker_worker import worker


class Converter:
    ready = True

    def __init__(self, delay, text):
        self.delay = delay
        self.text = text

    def convert(self, data):
        time.sleep(self.delay)
        return worker.blocks_to_document([
            worker.Block(
                "Page",
                children=[
                    worker.Block(
                        "Text",
                        html="<p>%s</p>" % self.text,
                        bbox=[10, 10, 100, 20],
                    )
                ],
            )
        ])


mode = sys.argv[1]
port = int(sys.argv[2])
if mode == "slow":
    converter = Converter(2.0, "never")
    timeout = 0.05
else:
    converter = Converter(0.0, "recovered")
    timeout = 5.0
worker.serve(
    converter,
    worker.WorkerConfig(max_concurrency=1, execution_timeout=timeout),
    "127.0.0.1",
    port,
)
"""


def block(block_type, html="", bbox=None, children=None, section_hierarchy=None):
    return worker.Block(
        block_type=block_type,
        html=html,
        bbox=bbox,
        children=children,
        section_hierarchy=section_hierarchy,
    )


class BlockConversionTests(unittest.TestCase):
    def test_html_paragraphs_remain_separate_marker_lines(self):
        document = worker.blocks_to_document([block(
            "Text",
            html="<p>Традиционные технические предуведомления:</p>"
            "<p>Для новых читателей: начинайте с этого текста.</p>"
            "<p>Продолжение технического предуведомления.</p>",
            bbox=[70, 10, 400, 80],
        )])
        self.assertEqual(
            ["".join(run.text for run in line.runs) for line in document.pages[0].lines],
            [
                "Традиционные технические предуведомления:",
                "Для новых читателей: начинайте с этого текста.",
                "Продолжение технического предуведомления.",
            ],
        )
        self.assertTrue(all(line.left == 70 for line in document.pages[0].lines))

    def test_pdftext_lines_restore_merged_notice_paragraphs(self):
        pdftext_pages = [{"blocks": [{"lines": [
            {"bbox": [70, 10, 400, 30], "spans": [{"text": "First notice.\n"}]},
            {"bbox": [70, 62, 400, 82], "spans": [{"text": "Second notice.\n"}]},
            {"bbox": [70, 114, 400, 134], "spans": [{"text": "Third notice.\n"}]},
        ]}]}]
        document = worker.blocks_to_document(
            [block(
                "Page",
                children=[block(
                    "ListGroup",
                    html="<p>First notice. Second notice. Third notice.</p>",
                    bbox=[70, 10, 400, 134],
                )],
            )],
            pdftext_pages,
        )
        lines = document.pages[0].lines
        self.assertEqual(
            ["".join(run.text for run in line.runs) for line in lines],
            ["First notice.", "Second notice.", "Third notice."],
        )
        self.assertTrue(all(line.left == 70 for line in lines))
        self.assertEqual([line.gap_before for line in lines], [0.0, 30.0, 30.0])

    def test_pdftext_wrapped_lines_keep_normal_line_spacing(self):
        pdftext_pages = [{"blocks": [{"lines": [
            {"bbox": [50, 10, 400, 30], "spans": [{"text": "Wrapped line one.\n"}]},
            {"bbox": [50, 32, 400, 52], "spans": [{"text": "Wrapped line two.\n"}]},
        ]}]}]
        document = worker.blocks_to_document(
            [block(
                "Page",
                children=[block(
                    "Text",
                    html="<p>Wrapped line one. Wrapped line two.</p>",
                    bbox=[50, 10, 400, 52],
                )],
            )],
            pdftext_pages,
        )
        lines = document.pages[0].lines
        self.assertEqual(len(lines), 2)
        self.assertEqual([line.gap_before for line in lines], [0.0, 2.0])

    def test_pdftext_split_preserves_mixed_emphasis(self):
        pdftext_pages = [{"blocks": [{"lines": [
            {
                "bbox": [70, 10, 400, 30],
                "spans": [
                    {"text": "First ", "font": {"name": "Helvetica"}},
                    {"text": "bold", "font": {"name": "Helvetica-Bold"}},
                    {"text": " line.\n", "font": {"name": "Helvetica"}},
                ],
            },
            {
                "bbox": [70, 62, 400, 82],
                "spans": [
                    {"text": "Second ", "font": {"name": "Helvetica"}},
                    {"text": "italic", "font": {"name": "Helvetica-Oblique"}},
                    {"text": " line.\n", "font": {"name": "Helvetica"}},
                ],
            },
            {
                "bbox": [70, 114, 400, 134],
                "spans": [
                    {"text": "Third", "font": {"name": "Helvetica-BoldOblique"}},
                    {"text": " line.\n", "font": {"name": "Helvetica"}},
                ],
            },
        ]}]}]
        document = worker.blocks_to_document(
            [block(
                "Page",
                children=[block(
                    "ListGroup",
                    html="<p>First bold line. Second italic line. Third line.</p>",
                    bbox=[70, 10, 400, 134],
                )],
            )],
            pdftext_pages,
        )
        lines = document.pages[0].lines
        self.assertEqual(
            ["".join(run.text for run in line.runs) for line in lines],
            ["First bold line.", "Second italic line.", "Third line."],
        )
        self.assertEqual(
            [(run.text, run.bold, run.italic) for run in lines[0].runs],
            [("First ", False, False), ("bold", True, False), (" line.", False, False)],
        )
        self.assertEqual(
            [(run.text, run.bold, run.italic) for run in lines[1].runs],
            [("Second ", False, False), ("italic", False, True), (" line.", False, False)],
        )
        self.assertEqual(
            [(run.text, run.bold, run.italic) for run in lines[2].runs],
            [("Third", True, True), (" line.", False, False)],
        )

    def test_pdftext_split_keeps_marker_emphasis_when_fonts_are_plain(self):
        pdftext_pages = [{"blocks": [{"lines": [
            {"bbox": [70, 10, 400, 30], "spans": [{"text": "Plain bold ", "font": {"name": "Helvetica"}}]},
            {"bbox": [70, 62, 400, 82], "spans": [{"text": "second line.\n", "font": {"name": "Helvetica"}}]},
        ]}]}]
        document = worker.blocks_to_document(
            [block(
                "Page",
                children=[block(
                    "ListGroup",
                    html="<p>Plain <strong>bold</strong> second line.</p>",
                    bbox=[70, 10, 400, 82],
                )],
            )],
            pdftext_pages,
        )
        lines = document.pages[0].lines
        self.assertEqual(
            ["".join(run.text for run in line.runs) for line in lines],
            ["Plain bold", "second line."],
        )
        self.assertTrue(any(run.bold and run.text == "bold" for run in lines[0].runs))

    def test_merged_paragraph_stays_intact_when_pdftext_does_not_match(self):
        pdftext_pages = [{"blocks": [{"lines": [
            {"bbox": [70, 10, 400, 30], "spans": [{"text": "Unrelated line one.\n"}]},
            {"bbox": [70, 62, 400, 82], "spans": [{"text": "Unrelated line two.\n"}]},
        ]}]}]
        document = worker.blocks_to_document(
            [block(
                "Page",
                children=[block(
                    "ListGroup",
                    html="<p>Merged notice text.</p>",
                    bbox=[70, 10, 400, 82],
                )],
            )],
            pdftext_pages,
        )
        lines = document.pages[0].lines
        self.assertEqual(len(lines), 1)
        self.assertEqual(
            "".join(run.text for run in lines[0].runs), "Merged notice text."
        )

    def test_pages_become_page_aware_lines_with_emphasis_and_heading_sizes(self):
        document = worker.blocks_to_document(
            [
                block(
                    "Page",
                    children=[
                        block(
                            "SectionHeader",
                            html="<h1>Title &amp; More</h1>",
                            bbox=[10, 20, 200, 50],
                            section_hierarchy={1: "header"},
                        ),
                        block(
                            "Text",
                            html="<p>Body <strong>bold</strong> and <em>italic</em>.</p>",
                            bbox=[10, 60, 300, 80],
                        ),
                    ],
                ),
                block("Page", children=[block("Text", html="<p>Second page</p>")]),
            ]
        )

        self.assertEqual(document.page_count, 2)
        first = document.pages[0].lines
        self.assertEqual(len(first), 2)
        self.assertEqual("".join(run.text for run in first[0].runs), "Title & More")
        self.assertTrue(all(run.bold for run in first[0].runs))
        self.assertEqual(first[0].size, 24.0)
        self.assertEqual(first[0].gap_before, 0.0)
        self.assertEqual(first[0].left, 10.0)

        body = first[1]
        self.assertEqual("".join(run.text for run in body.runs), "Body bold and italic.")
        self.assertEqual(body.gap_before, 30.0)
        self.assertEqual(body.size, 12.0)
        self.assertTrue(any(run.bold and run.text == "bold" for run in body.runs))
        self.assertTrue(any(run.italic and run.text == "italic" for run in body.runs))
        self.assertEqual(
            "".join(run.text for run in document.pages[1].lines[0].runs),
            "Second page",
        )

    def test_nested_containers_are_flattened_and_ignored_chrome_is_dropped(self):
        document = worker.blocks_to_document(
            [
                block(
                    "Page",
                    children=[
                        block("PageHeader", html="<p>running header</p>"),
                        block(
                            "TableGroup",
                            children=[
                                block(
                                    "Table",
                                    children=[
                                        block("TableCell", html="<p>cell one</p>", bbox=[1, 2, 3, 4]),
                                        block("TableCell", html="<p>cell two</p>", bbox=[1, 5, 3, 7]),
                                    ],
                                )
                            ],
                        ),
                        block("PageFooter", html="<p>page 1</p>"),
                    ],
                )
            ]
        )
        texts = ["".join(run.text for run in line.runs) for line in document.pages[0].lines]
        self.assertEqual(texts, ["cell one", "cell two"])

    def test_container_without_children_keeps_its_inline_text(self):
        document = worker.blocks_to_document(
            [
                block(
                    "Page",
                    children=[
                        block(
                            "ListGroup",
                            html="<p>First notice. Second notice.</p>",
                            bbox=[68, 198, 418, 278],
                        )
                    ],
                )
            ]
        )
        lines = document.pages[0].lines
        self.assertEqual(len(lines), 1)
        self.assertEqual(
            "".join(run.text for run in lines[0].runs),
            "First notice. Second notice.",
        )

    def test_markup_is_reduced_to_text_without_script_or_attributes(self):
        document = worker.blocks_to_document(
            [
                block(
                    "Page",
                    children=[
                        block(
                            "Text",
                            html=(
                                '<p onclick="steal()">Safe <a href="https://x">link</a>'
                                "<script>alert(1)</script><style>p{}</style>"
                                "<img src='x'> text</p>"
                            ),
                        )
                    ],
                )
            ]
        )
        text = "".join(run.text for run in document.pages[0].lines[0].runs)
        self.assertIn("Safe link", text)
        self.assertIn("text", text)
        for unsafe in ("onclick", "alert", "https://x", "<", ">"):
            self.assertNotIn(unsafe, text)

    def test_content_references_do_not_leak_placeholders(self):
        document = worker.blocks_to_document(
            [
                block(
                    "Page",
                    children=[
                        block(
                            "Text",
                            html="<p>before<content-ref src='/page/1/Text/2'></content-ref>after</p>",
                        )
                    ],
                )
            ]
        )
        text = "".join(run.text for run in document.pages[0].lines[0].runs)
        self.assertEqual(text, "before after")

    def test_section_header_without_markup_is_still_a_bold_heading(self):
        document = worker.blocks_to_document(
            [
                block(
                    "Page",
                    children=[
                        block(
                            "SectionHeader",
                            html="<p>Section</p>",
                            section_hierarchy={2: "section"},
                        )
                    ],
                )
            ]
        )
        line = document.pages[0].lines[0]
        self.assertEqual(line.size, 20.0)
        self.assertTrue(all(run.bold for run in line.runs))

    def test_invalid_coordinates_are_replaced_with_safe_defaults(self):
        document = worker.blocks_to_document(
            [
                block(
                    "Page",
                    children=[
                        block("Text", html="<p>first</p>", bbox=[float("inf"), 1, 2, 3]),
                        block("Text", html="<p>second</p>", bbox=[5, 10, 20, 20]),
                    ],
                )
            ]
        )
        lines = document.pages[0].lines
        self.assertEqual(lines[0].left, 0.0)
        self.assertEqual(lines[0].gap_before, 0.0)
        self.assertEqual(lines[1].gap_before, 30.0)

    def test_empty_pages_are_preserved_to_keep_page_count_consistent(self):
        document = worker.blocks_to_document(
            [
                block("Page", children=[]),
                block("Page", children=[block("Text", html="<p>content</p>")]),
            ]
        )
        self.assertEqual(document.page_count, 2)
        self.assertEqual(document.pages[0].lines, [])
        self.assertEqual(len(document.pages[1].lines), 1)
        self.assertEqual(document.to_dict()["page_count"], 2)


class FakeConverter:
    def __init__(self, document=None, delay=0.0, error=None, ready=True):
        self.ready = ready
        self.document = document
        self.delay = delay
        self.error = error

    def convert(self, data):
        if self.delay:
            time.sleep(self.delay)
        if self.error:
            raise self.error
        return self.document


class NativeTextSelectionTests(unittest.TestCase):
    def text_page(self, text):
        return {"blocks": [{"lines": [{"spans": [{"text": text}]}]}]}

    def test_complete_text_layer_requires_every_page(self):
        self.assertTrue(worker._has_complete_text_layer([self.text_page("one"), self.text_page("two")]))
        self.assertFalse(worker._has_complete_text_layer([self.text_page("one"), {"blocks": []}]))
        self.assertFalse(worker._has_complete_text_layer(None))

    def test_native_text_layer_requires_no_scanned_images(self):
        pages = [self.text_page("native")]
        with mock.patch.object(worker, "_pdf_image_scan", return_value=(False, False)):
            self.assertTrue(worker._native_text_layer_complete(b"%PDF-1.4", pages))
        with mock.patch.object(worker, "_pdf_image_scan", return_value=(True, False)):
            self.assertFalse(worker._native_text_layer_complete(b"%PDF-1.4", pages))
        with mock.patch.object(worker, "_pdf_image_scan", side_effect=RuntimeError):
            self.assertFalse(worker._native_text_layer_complete(b"%PDF-1.4", pages))

    def test_any_embedded_image_makes_the_text_layer_incomplete(self):
        class Item:
            type = 3

        class Page:
            def get_objects(self, max_depth):
                return [Item()]

        class Document:
            def __iter__(self):
                return iter([Page()])

            def close(self):
                pass

        pdfium = mock.Mock()
        pdfium.PdfDocument.return_value = Document()
        raw = mock.Mock()
        raw.FPDF_PAGEOBJ_IMAGE = 3
        raw.FPDF_PAGEOBJ_PATH = 2
        raw.FPDF_PAGEOBJ_FORM = 4
        pdfium.raw = raw
        with mock.patch.dict(
            sys.modules, {"pypdfium2": pdfium, "pypdfium2.raw": raw}
        ):
            self.assertTrue(worker._has_scanned_images(b"%PDF-1.4"))
        with mock.patch.object(worker, "_pdf_image_scan", return_value=(True, False)):
            self.assertFalse(
                worker._native_text_layer_complete(
                    b"%PDF-1.4", [self.text_page("native header")]
                )
            )

    def test_form_at_object_traversal_limit_makes_layer_incomplete(self):
        class BoundaryForm:
            type = 4
            level = worker.PDF_OBJECT_MAX_DEPTH - 1

        observed_depths = []

        class Page:
            def get_objects(self, max_depth):
                observed_depths.append(max_depth)
                return [BoundaryForm()]

        class Document:
            def __iter__(self):
                return iter([Page()])

            def close(self):
                pass

        pdfium = mock.Mock()
        pdfium.PdfDocument.return_value = Document()
        raw = mock.Mock()
        raw.FPDF_PAGEOBJ_IMAGE = 3
        raw.FPDF_PAGEOBJ_PATH = 2
        raw.FPDF_PAGEOBJ_FORM = 4
        pdfium.raw = raw
        with mock.patch.dict(
            sys.modules, {"pypdfium2": pdfium, "pypdfium2.raw": raw}
        ):
            self.assertTrue(worker._has_scanned_images(b"%PDF-1.4"))
        self.assertEqual(observed_depths, [worker.PDF_OBJECT_MAX_DEPTH])

    @unittest.skipUnless(
        importlib.util.find_spec("pypdfium2"),
        "real PDF image traversal requires pypdfium2",
    )
    def test_real_deeply_nested_image_pdf_fails_closed(self):
        fixture = os.path.join(ROOT, "tests", "fixtures", "marker", "mixed-scan-nested.pdf")
        with open(fixture, "rb") as source:
            data = source.read()
        self.assertEqual(worker._pdf_image_scan(data), (True, True))

        class Converter(worker.MarkerConverter):
            def __init__(self):
                super().__init__("fast")
                self._ready = True
                self._converter = lambda _data: (_ for _ in ()).throw(
                    AssertionError("must reject before converting")
                )
                self._ocr_converter = self._converter

            def _pdftext_pages(self, pdf_data):
                return [{"blocks": [{"lines": [{"spans": [{"text": "native"}]}]}]}]

        with self.assertRaisesRegex(worker.WorkerError, "traversal limit"):
            Converter().convert(data)

    def test_small_embedded_image_still_requires_ocr(self):
        class Page:
            def get_objects(self, max_depth):
                class SmallScannedImage:
                    type = 3

                    def get_bounds(self):
                        # Less than 1% of the 612x792pt page.
                        return (10, 10, 70, 70)

                return [SmallScannedImage()]

        class Document:
            def __iter__(self):
                return iter([Page()])

            def close(self):
                pass

        pdfium = mock.Mock()
        pdfium.PdfDocument.return_value = Document()
        raw = mock.Mock()
        raw.FPDF_PAGEOBJ_IMAGE = 3
        raw.FPDF_PAGEOBJ_PATH = 2
        raw.FPDF_PAGEOBJ_FORM = 4
        pdfium.raw = raw
        with mock.patch.dict(
            sys.modules, {"pypdfium2": pdfium, "pypdfium2.raw": raw}
        ):
            self.assertTrue(worker._has_scanned_images(b"%PDF-1.4"))
        with mock.patch.object(worker, "_pdf_image_scan", return_value=(True, False)):
            self.assertFalse(
                worker._native_text_layer_complete(
                    b"%PDF-1.4", [self.text_page("native header")]
                )
            )

    def test_vector_paths_make_native_text_incomplete(self):
        class VectorPath:
            type = 2

        class Page:
            def get_objects(self, max_depth):
                return [VectorPath()]

        class Document:
            def __iter__(self):
                return iter([Page()])

            def close(self):
                pass

        pdfium = mock.Mock()
        pdfium.PdfDocument.return_value = Document()
        raw = mock.Mock()
        raw.FPDF_PAGEOBJ_IMAGE = 3
        raw.FPDF_PAGEOBJ_PATH = 2
        raw.FPDF_PAGEOBJ_FORM = 4
        pdfium.raw = raw
        with mock.patch.dict(
            sys.modules, {"pypdfium2": pdfium, "pypdfium2.raw": raw}
        ):
            self.assertFalse(
                worker._native_text_layer_complete(
                    b"%PDF-1.4", [self.text_page("native header")]
                )
            )

    @unittest.skipUnless(
        importlib.util.find_spec("pypdfium2"),
        "real PDF vector traversal requires pypdfium2",
    )
    def test_real_vector_lettering_pdf_requires_ocr(self):
        fixture = os.path.join(ROOT, "tests", "fixtures", "marker", "mixed-scan.pdf")
        with open(fixture, "rb") as source:
            data = source.read()
        has_graphics, traversal_truncated = worker._pdf_image_scan(data)
        self.assertTrue(has_graphics)
        self.assertFalse(traversal_truncated)
        self.assertFalse(
            worker._native_text_layer_complete(
                data, [self.text_page("Native header for mixed page")]
            )
        )

    def test_page_with_native_text_and_scanned_content_uses_ocr(self):
        calls = []
        text_page = self.text_page("native header")

        class Page:
            block_type = "Page"
            html = "<p>ocr</p>"
            bbox = [0, 0, 10, 10]
            children = None
            section_hierarchy = None

        class Rendered:
            children = [Page()]

        class Converter(worker.MarkerConverter):
            def __init__(self):
                super().__init__("fast")
                self._ready = True
                self._artifacts = {}
                self._converter = lambda _data: (calls.append("automatic"), Rendered())[1]
                self._native_converter = lambda _data: (calls.append("no_ocr"), Rendered())[1]
                self._ocr_converter = lambda _data: (calls.append("forced_ocr"), Rendered())[1]

            def _pdftext_pages(self, data):
                return [text_page]

        converter = Converter()
        with mock.patch.object(worker, "_pdf_image_scan", return_value=(True, False)):
            document = converter.convert(b"%PDF-1.4")
        self.assertEqual(calls, ["forced_ocr"])
        self.assertEqual(document.page_count, 1)

    def test_explicit_ocr_disabled_mode_rejects_incomplete_text_layer(self):
        text_page = self.text_page("native header")

        def fail(_data):
            raise AssertionError("conversion must not run when OCR is required")

        class Converter(worker.MarkerConverter):
            def __init__(self):
                super().__init__("no_ocr")
                self._ready = True
                self._artifacts = {}
                self._converter = fail

            def _pdftext_pages(self, data):
                return [text_page]

            def _native_text_layer_status(self, data, pages):
                return False, False

        with self.assertRaisesRegex(worker.WorkerError, "OCR-disabled"):
            Converter().convert(b"%PDF-1.4")

    def test_uninspectable_deep_form_fails_without_conversion(self):
        class Converter(worker.MarkerConverter):
            def __init__(self):
                super().__init__("fast")
                self._ready = True
                self._converter = lambda _data: (_ for _ in ()).throw(
                    AssertionError("must not convert a page with uninspectable forms")
                )
                self._ocr_converter = lambda _data: (_ for _ in ()).throw(
                    AssertionError("OCR cannot establish form traversal completeness")
                )

            def _pdftext_pages(self, data):
                return [self.text_page("native header")]

            def text_page(self, text):
                return {"blocks": [{"lines": [{"spans": [{"text": text}]}]}]}

            def _native_text_layer_status(self, data, pages):
                return False, True

        with self.assertRaisesRegex(worker.WorkerError, "traversal limit"):
            Converter().convert(b"%PDF-1.4")

    def test_explicit_ocr_disabled_mode_converts_complete_native_text(self):
        calls = []
        text_page = self.text_page("native")

        class Page:
            block_type = "Page"
            html = "<p>native</p>"
            bbox = [0, 0, 10, 10]
            children = None
            section_hierarchy = None

        class Rendered:
            children = [Page()]

        class Converter(worker.MarkerConverter):
            def __init__(self):
                super().__init__("no_ocr")
                self._ready = True
                self._artifacts = {}
                self._converter = lambda _data: (calls.append("no_ocr"), Rendered())[1]

            def _pdftext_pages(self, data):
                return [text_page]

            def _native_text_layer_status(self, data, pages):
                return True, False

        document = Converter().convert(b"%PDF-1.4")
        self.assertEqual(calls, ["no_ocr"])
        self.assertEqual(document.page_count, 1)

    def test_native_document_uses_text_layer_mode(self):
        calls = []
        text_page = self.text_page("native")

        class Page:
            block_type = "Page"
            html = "<p>native</p>"
            bbox = [0, 0, 10, 10]
            children = None
            section_hierarchy = None

        class Rendered:
            children = [Page()]

        class Converter(worker.MarkerConverter):
            def __init__(self):
                super().__init__("fast")
                self._ready = True
                self._artifacts = {}
                self._converter = lambda _data: (calls.append("fast"), Rendered())[1]
                self._native_converter = lambda _data: (calls.append("no_ocr"), Rendered())[1]
                self._ocr_converter = lambda _data: (calls.append("forced_ocr"), Rendered())[1]

            def _pdftext_pages(self, data):
                return [text_page, text_page]

        converter = Converter()
        with mock.patch.object(worker, "_pdf_image_scan", return_value=(False, False)):
            document = converter.convert(b"%PDF-1.4")
        self.assertEqual(calls, ["no_ocr"])
        self.assertEqual(document.page_count, 1)

    def test_mixed_document_uses_ocr_mode(self):
        calls = []
        text_page = self.text_page("native")

        class Page:
            block_type = "Page"
            html = "<p>ocr</p>"
            bbox = [0, 0, 10, 10]
            children = None
            section_hierarchy = None

        class Rendered:
            children = [Page()]

        class Converter(worker.MarkerConverter):
            def __init__(self):
                super().__init__("fast")
                self._ready = True
                self._artifacts = {}
                self._converter = lambda _data: (calls.append("fast"), Rendered())[1]
                self._native_converter = lambda _data: (calls.append("no_ocr"), Rendered())[1]
                self._ocr_converter = lambda _data: (calls.append("forced_ocr"), Rendered())[1]

            def _pdftext_pages(self, data):
                return [text_page, {"blocks": []}]

        converter = Converter()
        document = converter.convert(b"%PDF-1.4")
        self.assertEqual(calls, ["forced_ocr"])
        self.assertEqual(document.page_count, 1)


class MarkerConverterSetupTests(unittest.TestCase):
    def test_fast_mode_builds_a_converter_that_forces_ocr(self):
        configs = []

        class FakePdfConverter:
            def __init__(self, *, artifact_dict, renderer, config):
                configs.append(config)
                self.processor_list = []

            def __call__(self, _data):
                raise AssertionError("this setup test does not convert a document")

        marker = ModuleType("marker")
        marker.__path__ = []
        converters = ModuleType("marker.converters")
        converters.__path__ = []
        pdf = ModuleType("marker.converters.pdf")
        pdf.PdfConverter = FakePdfConverter
        models = ModuleType("marker.models")
        models.create_model_dict = lambda: {}
        with mock.patch.dict(
            sys.modules,
            {
                "marker": marker,
                "marker.converters": converters,
                "marker.converters.pdf": pdf,
                "marker.models": models,
            },
        ):
            converter = worker.MarkerConverter("fast")
            converter.load()

        self.assertTrue(any(config.get("force_ocr") is True for config in configs))
        self.assertIsNotNone(converter._ocr_converter)
        self.assertIs(converter._converter, converter._ocr_converter)


class EmptyPageNormalizationTests(unittest.TestCase):
    def test_none_children_become_empty_lists_before_marker_processors(self):
        class Page:
            children = None

        class Document:
            pages = [Page(), Page()]

        worker._NormalizeEmptyPageChildren()(Document())
        self.assertEqual([page.children for page in Document.pages], [[], []])


class MarkerConverterFailureTests(unittest.TestCase):
    class Failing:
        def __call__(self, _data):
            raise RuntimeError("fast mode failed")

    class Converter(worker.MarkerConverter):
        def __init__(self):
            super().__init__("fast")
            self._ready = True
            self._converter = MarkerConverterFailureTests.Failing()
            self._ocr_converter = MarkerConverterFailureTests.Failing()

    def test_fast_mode_failure_is_reported_without_ocr_disabled_retry(self):
        converter = self.Converter()
        with self.assertRaisesRegex(worker.WorkerError, "could not convert"):
            converter.convert(b"%PDF-1.4")


class WorkerHttpTests(unittest.TestCase):
    def setUp(self):
        self.server = None
        self.thread = None

    def tearDown(self):
        if self.server is not None:
            self.server.shutdown()
            self.server.server_close()
        if self.thread is not None:
            self.thread.join(timeout=2)

    def start(self, converter, restart_on_timeout=None, **config):
        self.server = worker.WorkerServer(
            ("127.0.0.1", 0), converter, worker.WorkerConfig(**config),
            restart_on_timeout=restart_on_timeout,
        )
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        return f"http://127.0.0.1:{self.server.server_address[1]}"

    def request(self, base, path, data=None, content_type="application/pdf", headers=None):
        request = urllib.request.Request(
            f"{base}{path}",
            data=data,
            headers={"Content-Type": content_type, **(headers or {})},
            method="POST" if data is not None else "GET",
        )
        try:
            with urllib.request.urlopen(request, timeout=5) as response:
                return response.status, json.loads(response.read())
        except urllib.error.HTTPError as error:
            try:
                return error.code, json.loads(error.read())
            finally:
                error.close()

    def test_health_reports_starting_and_ready(self):
        converter = FakeConverter(ready=False)
        base = self.start(converter)
        self.assertEqual(self.request(base, "/healthz"), (503, {"status": "starting", "ready": False}))
        converter.ready = True
        self.assertEqual(self.request(base, "/healthz"), (200, {"status": "ok", "ready": True}))

    def test_extract_returns_the_page_aware_contract(self):
        document = worker.blocks_to_document(
            [block("Page", children=[block("Text", html="<p>hello</p>")])]
        )
        base = self.start(FakeConverter(document))
        status, payload = self.request(base, "/extract", b"%PDF-1.4")
        self.assertEqual(status, 200)
        self.assertEqual(payload["page_count"], 1)
        self.assertEqual(payload["pages"][0]["lines"][0]["runs"][0]["text"], "hello")

    def test_invalid_requests_are_rejected_without_conversion(self):
        base = self.start(FakeConverter(worker.blocks_to_document([block("Page")])))
        self.assertEqual(
            self.request(base, "/extract", b"%PDF-1.4", content_type="text/plain")[0], 415
        )
        self.assertEqual(self.request(base, "/missing", b"%PDF-1.4")[0], 404)
        self.assertEqual(self.request(base, "/healthz")[0], 200)
        self.assertEqual(
            self.request(
                base,
                "/extract",
                b"%PDF-1.4",
                headers={"Content-Length": "999999999"},
            )[0],
            413,
        )

    def test_conversion_errors_are_reported_without_document_text(self):
        base = self.start(
            FakeConverter(error=worker.WorkerError("Marker could not convert this PDF"))
        )
        status, payload = self.request(base, "/extract", b"%PDF-1.4")
        self.assertEqual(status, 422)
        self.assertNotIn("secret", payload["error"])

    def test_unexpected_conversion_errors_are_generic(self):
        base = self.start(FakeConverter(error=RuntimeError("secret document text")))
        status, payload = self.request(base, "/extract", b"%PDF-1.4")
        self.assertEqual(status, 500)
        self.assertEqual(payload["error"], "conversion failed")

    def test_response_size_limit_is_enforced(self):
        document = worker.blocks_to_document(
            [block("Page", children=[block("Text", html="<p>" + "x" * 500 + "</p>")])]
        )
        base = self.start(FakeConverter(document), max_response_bytes=128)
        status, payload = self.request(base, "/extract", b"%PDF-1.4")
        self.assertEqual(status, 413)
        self.assertIn("output size", payload["error"])

    def test_execution_timeout_returns_a_gateway_timeout(self):
        document = worker.blocks_to_document([block("Page")])
        base = self.start(
            FakeConverter(document, delay=0.3),
            restart_on_timeout=lambda: None,
            execution_timeout=0.05,
        )
        status, payload = self.request(base, "/extract", b"%PDF-1.4")
        self.assertEqual(status, 504)
        self.assertIn("execution time", payload["error"])

    def test_timeout_restarts_worker_and_next_conversion_succeeds(self):
        document = worker.blocks_to_document([block("Page", children=[block("Text", "<p>recovered</p>")])])
        restart_requests = []

        def restart_worker():
            restart_requests.append(True)
            old_executor = self.server.executor
            self.server.executor = worker.ThreadPoolExecutor(max_workers=1)
            self.server.slots = threading.BoundedSemaphore(1)
            self.server.converter = FakeConverter(document)
            old_executor.shutdown(wait=False, cancel_futures=True)

        base = self.start(
            FakeConverter(document, delay=0.3),
            restart_on_timeout=restart_worker,
            execution_timeout=0.05,
            max_concurrency=1,
        )
        status, payload = self.request(base, "/extract", b"%PDF-1.4")
        self.assertEqual(status, 504, payload)
        self.assertEqual(restart_requests, [True])
        status, payload = self.request(base, "/extract", b"%PDF-1.4")
        self.assertEqual(status, 200, payload)
        self.assertEqual(payload["pages"][0]["lines"][0]["runs"][0]["text"], "recovered")

    def test_timeout_restart_runs_before_a_failing_response(self):
        events = []
        handle_timeout = worker.MarkerRequestHandler._handle_timeout

        class Handler:
            worker = type(
                "Worker",
                (),
                {"restart_on_timeout": staticmethod(lambda: events.append("restart"))},
            )()

            def _error(self, status, message):
                events.append("response")
                raise BrokenPipeError("client disconnected")

            _handle_timeout = handle_timeout

        Handler()._handle_timeout()
        self.assertEqual(events, ["restart", "response"])

    def test_concurrency_limit_rejects_extra_work(self):
        document = worker.blocks_to_document([block("Page")])
        base = self.start(FakeConverter(document, delay=0.4), max_concurrency=1)
        first = {}
        thread = threading.Thread(
            target=lambda: first.update(result=self.request(base, "/extract", b"%PDF-1.4"))
        )
        thread.start()
        time.sleep(0.1)
        self.assertEqual(self.request(base, "/extract", b"%PDF-1.4")[0], 503)
        thread.join(timeout=3)
        self.assertEqual(first["result"][0], 200)


class WorkerProcessRestartTests(unittest.TestCase):
    def free_port(self):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
            probe.bind(("127.0.0.1", 0))
            return probe.getsockname()[1]

    def start(self, mode, port):
        return subprocess.Popen(
            [sys.executable, "-c", WORKER_PROCESS, mode, str(port)],
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

    def wait_ready(self, port, process):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if process.poll() is not None:
                _, stderr = process.communicate()
                self.fail(f"worker exited before becoming ready: {stderr}")
            try:
                with urllib.request.urlopen(
                    f"http://127.0.0.1:{port}/healthz", timeout=0.5
                ) as response:
                    if response.status == 200:
                        return
            except (urllib.error.URLError, ConnectionError):
                time.sleep(0.05)
        self.fail("worker did not become ready")

    def extract(self, port):
        request = urllib.request.Request(
            f"http://127.0.0.1:{port}/extract",
            data=b"%PDF-1.4",
            headers={"Content-Type": "application/pdf"},
            method="POST",
        )
        try:
            with urllib.request.urlopen(request, timeout=5) as response:
                return response.status, json.loads(response.read())
        except urllib.error.HTTPError as error:
            try:
                return error.code, json.loads(error.read())
            finally:
                error.close()

    def test_timeout_restarts_the_production_process_and_the_restarted_worker_converts(self):
        port = self.free_port()
        slow = self.start("slow", port)
        try:
            self.wait_ready(port, slow)
            status, payload = self.extract(port)
            self.assertEqual(status, 504, payload)
            self.assertEqual(slow.wait(timeout=10), 1)
        finally:
            if slow.poll() is None:
                slow.kill()
                slow.wait()
            slow.communicate()

        restarted = self.start("fast", port)
        try:
            self.wait_ready(port, restarted)
            status, payload = self.extract(port)
            self.assertEqual(status, 200, payload)
            self.assertEqual(
                payload["pages"][0]["lines"][0]["runs"][0]["text"], "recovered"
            )
        finally:
            if restarted.poll() is None:
                restarted.terminate()
                try:
                    restarted.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    restarted.kill()
                    restarted.wait()
            restarted.communicate()


if __name__ == "__main__":
    unittest.main()
