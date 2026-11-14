#!/usr/bin/env python3
"""End-to-end Marker and Pdfium extraction through the management API and RSS."""

import http.cookiejar
import json
import os
import re
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from html.parser import HTMLParser


APP = os.environ["APP_BASE_URL"].rstrip("/")
RSS = os.environ["RSS_BASE_URL"].rstrip("/")
ENGINE = os.environ.get("EXTRACTION_ENGINE", "marker")
MARKER_WORKER = os.environ.get("MARKER_WORKER_URL", "").rstrip("/")


class SafeMarkup(HTMLParser):
    allowed = {"p", "br", "h1", "h2", "h3", "strong", "em", "ul", "ol", "li", "blockquote"}

    def handle_starttag(self, tag, attrs):
        assert tag in self.allowed, f"unexpected HTML tag: {tag}"
        assert not attrs, f"unexpected HTML attributes: {attrs}"

    def handle_startendtag(self, tag, attrs):
        assert tag in self.allowed, f"unexpected HTML tag: {tag}"
        assert not attrs, f"unexpected HTML attributes: {attrs}"


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def request(url, *, method="GET", payload=None, headers=None, opener=None):
    body = None if payload is None else json.dumps(payload).encode()
    actual_headers = {"Content-Type": "application/json", **(headers or {})}
    req = urllib.request.Request(url, data=body, method=method, headers=actual_headers)
    client = opener or urllib.request.build_opener()
    with client.open(req, timeout=180) as response:
        data = response.read()
        return response.status, (
            json.loads(data)
            if data and "json" in response.headers.get("Content-Type", "")
            else data
        )


def rss_items(rss_url):
    _, rss_bytes = request(rss_url)
    channel = ET.fromstring(rss_bytes)
    return channel.findall("./channel/item")


def verify_pinned_mixed_page_runtime():
    """Exercise the pinned worker on a page whose text layer is incomplete."""
    assert MARKER_WORKER, "Marker E2E must provide the private worker URL"
    with open("/marker-fixtures/mixed-scan.pdf", "rb") as source:
        pdf = source.read()
    req = urllib.request.Request(
        f"{MARKER_WORKER}/extract",
        data=pdf,
        method="POST",
        headers={"Content-Type": "application/pdf"},
    )
    with urllib.request.urlopen(req, timeout=180) as response:
        document = json.loads(response.read())
    output = " ".join(
        run["text"]
        for page in document["pages"]
        for line in page["lines"]
        for run in line["runs"]
    )
    for expected in (
        "Native header for mixed page",
        "This native paragraph is in the text layer.",
        "Scanned text in a small region",
        "must still trigger OCR.",
        "Following native text stays outside the image.",
    ):
        assert expected in output, f"pinned Marker omitted {expected!r}: {output}"
    # Tesseract-style OCR can mistake the outlined V for U and space the X;
    # these fragments verify it read the path lettering without requiring a
    # perfect OCR transcript.
    assert "ECTOR" in output, f"pinned Marker omitted vector lettering: {output}"
    assert "TE X T" in output, f"pinned Marker omitted vector lettering: {output}"


def main():
    assert ENGINE == "marker", f"this check expects the Marker engine, got {ENGINE}"
    verify_pinned_mixed_page_runtime()
    beta_filename = (
        "[Философское] «Старые деньги» — FAQ_ как правильно считать дивиденды, "
        "«бессмертные» портфели и стоицизм.pdf"
    )
    beta_title = beta_filename.removesuffix(".pdf")
    cookies = http.cookiejar.CookieJar()
    app_client = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cookies))
    request(
        APP + "/api/management/login",
        method="POST",
        payload={
            "username": os.environ["OPERATOR_USERNAME"],
            "password": os.environ["OPERATOR_PASSWORD"],
        },
        opener=app_client,
    )

    _, feed = request(
        APP + "/api/management/feeds",
        method="POST",
        payload={
            "display_name": "Marker E2E",
            "description": "Disposable Marker test feed",
            "extraction_engine": "marker",
        },
        opener=app_client,
    )
    assert feed["extraction_engine"] == "marker", feed
    feed_id = feed["id"]
    request(
        f"{APP}/api/management/feeds/{feed_id}/years",
        method="POST",
        payload={"year": 2099},
        opener=app_client,
    )
    _, month = request(
        f"{APP}/api/management/feeds/{feed_id}/years/2099/months/11",
        method="PUT",
        payload={"source_url": os.environ["FIXTURE_SOURCE_URL"]},
        opener=app_client,
    )
    month_id = month["id"]

    _, discovery = request(
        f"{APP}/api/management/months/{month_id}/discover",
        method="POST",
        opener=app_client,
    )
    assert discovery["discovered_pdfs"] == 3, discovery
    _, documents = request(
        f"{APP}/api/management/months/{month_id}/documents", opener=app_client
    )
    by_name = {document["filename"]: document for document in documents}
    assert set(by_name) == {"article-alpha.pdf", beta_filename, "club-promotion.pdf"}

    extracted = {}
    for document in documents:
        _, result = request(
            f"{APP}/api/management/documents/{document['id']}/extract?force=true",
            method="POST",
            opener=app_client,
        )
        assert result["status"] == "success", result
        assert result["page_count"] > 0, result
        assert result["characters"] > 0, result
        extracted[document["filename"]] = result

    alpha_sample = extracted["article-alpha.pdf"]["first_page_sample"]
    assert "куплено на" not in alpha_sample.lower(), alpha_sample
    assert extracted[beta_filename]["first_page_sample"].strip()

    for document in documents:
        _, cached = request(
            f"{APP}/api/management/documents/{document['id']}/extract",
            method="POST",
            opener=app_client,
        )
        assert cached["status"] == "cached", cached

    rss_url = f"{RSS}/feeds/{feed_id}"
    items = rss_items(rss_url)
    assert len(items) == 2, f"expected two eligible articles, got {len(items)}"
    by_title = {item.findtext("title"): item.findtext("description") or "" for item in items}
    assert set(by_title) == {"article-alpha", beta_title}, by_title

    alpha_html = by_title["article-alpha"]
    assert "Fixture article alpha wraps across two lines." in alpha_html, alpha_html
    assert (
        "<strong>Bold </strong><em>italic<strong> both</strong></em> styles."
        in alpha_html
    ), alpha_html
    assert "06.11.2026 12:12 Дмитриев" in alpha_html, alpha_html
    for chrome in ("КУПЛЕНО", "skladchik", "onclick", "<script", "<img"):
        assert chrome not in alpha_html, f"unsafe or promotional content leaked: {chrome}: {alpha_html}"
    assert not re.search(r"<p>\s*\d+\s*</p>", alpha_html), "page label leaked"

    beta_html = by_title[beta_title]
    normalized_beta_title = beta_title.replace("FAQ_", "FAQ:", 1)
    assert beta_html.startswith(f"<h1><strong>{normalized_beta_title}</strong></h1>"), beta_html
    assert "Сразу к делу, коллеги." in beta_html, beta_html
    expected_notice = (
        "<blockquote><p>Традиционные технические <strong>предуведомления:</strong></p>\n"
        "<p>Для новых читателей: <em>начинайте с этого текста.</em></p>\n"
        "<p>Продолжение <em><strong>технического предуведомления.</strong></em></p></blockquote>"
    )
    assert expected_notice in beta_html, beta_html
    quote_end = beta_html.find("</blockquote>")
    assert quote_end != -1, beta_html
    following_body = (
        "<p>На данном проекте периодически упоминается термин «старые деньги».</p>"
    )
    following_index = beta_html.find(following_body)
    assert following_index != -1, beta_html
    assert following_index > quote_end, beta_html
    for chrome in ("sponsr", "CrimsonAlter", "27 февр 11:44", "58 минут"):
        assert chrome not in beta_html, f"Sponsr chrome leaked: {chrome}: {beta_html}"
    for html in by_title.values():
        SafeMarkup().feed(html)

    # Explicitly inherit Pdfium, then select Pdfium again. The second selection
    # must reuse the cache because the effective engine is unchanged.
    _, inherited = request(
        f"{APP}/api/management/feeds/{feed_id}",
        method="PUT",
        payload={
            "display_name": "Marker E2E",
            "description": "Disposable Marker test feed",
            "extraction_engine": None,
        },
        opener=app_client,
    )
    assert inherited["extraction_engine"] is None, inherited
    _, pdfium = request(
        f"{APP}/api/management/documents/{by_name['article-alpha.pdf']['id']}/extract?force=true",
        method="POST",
        opener=app_client,
    )
    assert pdfium["status"] == "success", pdfium
    pdfium_items = {item.findtext("title"): item.findtext("description") or "" for item in rss_items(rss_url)}
    pdfium_alpha = pdfium_items["article-alpha"]
    assert "<h1><strong>06.11.2026 12:12</strong></h1>" in pdfium_alpha, pdfium_alpha
    assert pdfium_alpha != alpha_html, "switching engines must replace cached HTML"

    _, explicit = request(
        f"{APP}/api/management/feeds/{feed_id}",
        method="PUT",
        payload={
            "display_name": "Marker E2E",
            "description": "Disposable Marker test feed",
            "extraction_engine": "pdfium",
        },
        opener=app_client,
    )
    assert explicit["extraction_engine"] == "pdfium", explicit
    _, still_cached = request(
        f"{APP}/api/management/documents/{by_name['article-alpha.pdf']['id']}/extract",
        method="POST",
        opener=app_client,
    )
    assert still_cached["status"] == "cached", still_cached

    # Return to Marker and verify a fresh extraction replaces the Pdfium cache.
    _, marker = request(
        f"{APP}/api/management/feeds/{feed_id}",
        method="PUT",
        payload={
            "display_name": "Marker E2E",
            "description": "Disposable Marker test feed",
            "extraction_engine": "marker",
        },
        opener=app_client,
    )
    assert marker["extraction_engine"] == "marker", marker
    _, switched = request(
        f"{APP}/api/management/documents/{by_name['article-alpha.pdf']['id']}/extract",
        method="POST",
        opener=app_client,
    )
    assert switched["status"] == "success", switched
    marker_items = {item.findtext("title"): item.findtext("description") or "" for item in rss_items(rss_url)}
    assert marker_items["article-alpha"] == alpha_html, "Marker cache identity changed unexpectedly"

    # Document links remain stable redirects to the source PDF.
    no_redirect = urllib.request.build_opener(NoRedirect())
    for title, filename in (("article-alpha", "article-alpha.pdf"), (beta_title, beta_filename)):
        item = next(item for item in items if item.findtext("title") == title)
        try:
            request(item.findtext("link"), opener=no_redirect)
            raise AssertionError("document route did not return a redirect")
        except urllib.error.HTTPError as response:
            assert response.code in (301, 302, 303, 307, 308), response.code
            assert urllib.parse.unquote(response.headers["Location"]) == (
                f"https://fixture.invalid/current-month/{filename}"
            ), response.headers["Location"]

    print(
        "Marker E2E passed: extraction, cleanup, safe RSS, cache reuse, "
        "inherit/override switching, and engine-aware cache replacement."
    )


if __name__ == "__main__":
    main()
