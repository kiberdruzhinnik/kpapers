#!/usr/bin/env python3
"""End-to-end feed publication and Miniflux reader contract."""

import http.cookiejar
import json
import os
import time
import urllib.request
import urllib.error
import urllib.parse
import xml.etree.ElementTree as ET
from html.parser import HTMLParser


APP = os.environ["APP_BASE_URL"].rstrip("/")
RSS = os.environ["RSS_BASE_URL"].rstrip("/")
MINIFLUX = os.environ["MINIFLUX_BASE_URL"].rstrip("/")
CONTENT_NS = "{http://purl.org/rss/1.0/modules/content/}encoded"
ARTICLE_ALPHA_TITLE = "Древние индейцы делятся зелёными технологиями"
ARTICLE_ALPHA_LEAD = "Сегодня для нас нет ничего актуальней добра."


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
    with client.open(req, timeout=15) as response:
        data = response.read()
        return response.status, (json.loads(data) if data and "json" in response.headers.get("Content-Type", "") else data)


def wait_for(check, message, timeout=90):
    deadline = time.monotonic() + timeout
    last_error = None
    while time.monotonic() < deadline:
        try:
            result = check()
            if result:
                return result
        except Exception as error:  # services may still be starting or polling
            last_error = error
        time.sleep(1)
    raise AssertionError(f"Timed out waiting for {message}; last error: {last_error}")


def main():
    reference_pdf_test = os.environ.get("REFERENCE_PDF_TEST") == "true"
    generated_beta_filename = "[Философское] «Старые деньги» — FAQ_ как правильно считать дивиденды, «бессмертные» портфели и стоицизм.pdf"
    beta_filename = os.environ.get("REFERENCE_DOCUMENT_FILENAME", generated_beta_filename) if reference_pdf_test else generated_beta_filename
    beta_title = beta_filename.removesuffix(".pdf").removesuffix(".PDF")
    scanned_only_filename = "scanned-only.pdf"

    def expect_scanned_only_failure(document, force=True):
        query = "?force=true" if force else ""
        try:
            request(f"{APP}/api/management/documents/{document['id']}/extract{query}", method="POST", opener=app_client)
            raise AssertionError("scanned-only PDF without OCR must fail extraction")
        except urllib.error.HTTPError as response:
            assert response.code == 502, response.code

    cookies = http.cookiejar.CookieJar()
    app_client = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(cookies))
    request(APP + "/api/management/login", method="POST", payload={
        "username": os.environ["OPERATOR_USERNAME"],
        "password": os.environ["OPERATOR_PASSWORD"],
    }, opener=app_client)

    _, feed = request(APP + "/api/management/feeds", method="POST", payload={
        "display_name": "Miniflux E2E", "description": "Disposable test feed",
    }, opener=app_client)
    feed_id = feed["id"]
    request(f"{APP}/api/management/feeds/{feed_id}/years", method="POST", payload={"year": 2099}, opener=app_client)
    _, month = request(f"{APP}/api/management/feeds/{feed_id}/years/2099/months/11", method="PUT", payload={
        "source_url": os.environ["FIXTURE_SOURCE_URL"],
    }, opener=app_client)
    month_id = month["id"]

    _, discovery = request(f"{APP}/api/management/months/{month_id}/discover", method="POST", opener=app_client)
    assert discovery["discovered_pdfs"] == 4, discovery
    _, documents = request(f"{APP}/api/management/months/{month_id}/documents", opener=app_client)
    by_name = {document["filename"]: document for document in documents}
    assert set(by_name) == {"article-alpha.pdf", beta_filename, "club-promotion.pdf", scanned_only_filename}
    for document in documents:
        if document["filename"] == scanned_only_filename:
            expect_scanned_only_failure(document)
            continue
        _, extraction = request(f"{APP}/api/management/documents/{document['id']}/extract?force=true", method="POST", opener=app_client)
        if reference_pdf_test and document["filename"] == beta_filename:
            sample = extraction["first_page_sample"]
            assert sample.strip(), "reference PDF extracted no first-page text"
            assert "куплено на" not in sample.lower(), sample
            assert "skladchik.org" not in sample.lower(), sample
            reference_title = beta_title[beta_title.find("["):] if "[" in beta_title else beta_title
            for suffix in (" [skladchik.org]", " [sklad"):
                reference_title = reference_title.removesuffix(suffix)
            assert sample.startswith(reference_title), f"reference title missing at start of extraction: {sample[:500]!r}"
    _, documents = request(f"{APP}/api/management/months/{month_id}/documents", opener=app_client)
    scanned_only = next(document for document in documents if document["filename"] == scanned_only_filename)
    assert scanned_only["extraction_status"] == "failure", scanned_only
    assert "no usable native text" in (scanned_only["last_error"] or "").lower(), scanned_only
    for document in documents:
        if document["filename"] == scanned_only_filename:
            expect_scanned_only_failure(document, force=False)
            continue
        _, extraction = request(f"{APP}/api/management/documents/{document['id']}/extract", method="POST", opener=app_client)
        assert extraction["status"] == "cached", extraction

    rss_url = f"{RSS}/feeds/{feed_id}"
    _, rss_bytes = request(rss_url)
    try:
        channel = ET.fromstring(rss_bytes)
    except ET.ParseError as error:
        line, column = error.position
        context = rss_bytes.decode("utf-8", "replace").splitlines()[line - 1]
        raise AssertionError(f"RSS XML parse failed at {line}:{column}: {context[max(0, column - 120):column + 120]}") from error
    items = channel.findall("./channel/item")
    assert len(items) == 2, f"expected two eligible articles, got {len(items)}"
    guid_by_title = {item.findtext("title"): item.findtext("guid") for item in items}
    assert set(guid_by_title) == {"article-alpha", beta_title}
    document_id_by_name = {document["filename"]: document["id"] for document in documents}
    item_url_by_title = {item.findtext("title"): item.findtext("link") for item in items}
    expected_links = {
        "article-alpha": f"{RSS}/documents/{document_id_by_name['article-alpha.pdf']}/download",
        beta_title: f"{RSS}/documents/{document_id_by_name[beta_filename]}/download",
    }
    assert item_url_by_title == expected_links, f"RSS links must be stable per-document redirects: {item_url_by_title}"
    alpha_item = next(item for item in items if item.findtext("title") == "article-alpha")
    alpha_html = alpha_item.findtext("description") or ""
    alpha_encoded = alpha_item.findtext(CONTENT_NS) or ""
    assert alpha_encoded == alpha_html, "description and encoded content must publish the same article HTML"
    assert alpha_html.startswith(f"<h1>{ARTICLE_ALPHA_TITLE}</h1>"), alpha_html[:800]
    assert f"<p>{ARTICLE_ALPHA_LEAD}</p>" in alpha_html, alpha_html[:1200]
    assert "<p>Fixture article alpha wraps across two lines." in alpha_html, alpha_html[:1200]
    assert "<p>это не инвестиционные рекомендации</p>" in alpha_html, alpha_html[:1200]
    beta_item = next(item for item in items if item.findtext("title") == beta_title)
    beta_html = beta_item.findtext("description") or ""
    if reference_pdf_test:
        assert beta_html.strip(), "reference PDF produced empty RSS content"
        expected_reference_title = beta_title[beta_title.find("["):] if "[" in beta_title else beta_title
        for suffix in (" [skladchik.org]", " [sklad"):
            expected_reference_title = expected_reference_title.removesuffix(suffix)
        expected_title_markup = f"<h1><strong>{expected_reference_title}</strong></h1>"
        assert beta_html.startswith(expected_title_markup), f"reference article does not start with its filename headline: {beta_html[:800]!r}"
        assert "куплено на" not in beta_html.lower(), beta_html[:1200]
        assert "skladchik.org" not in beta_html.lower(), beta_html[:1200]
        assert not __import__("re").search(r"<p>\s*\d+\s*</p>", beta_html), "standalone page number leaked into reference content"
        print(f"Live reference PDF first 1,200 HTML chars: {beta_html[:1200]}")
    else:
        assert beta_html.startswith("<h1><strong>[Философское] «Старые деньги» — FAQ: как правильно считать дивиденды, «бессмертные» портфели и стоицизм</strong></h1>"), beta_html
        assert "<p>Сразу к делу, коллеги.</p>" in beta_html, beta_html
        assert beta_html.endswith("<p>На данном проекте периодически упоминается термин «старые деньги».</p>"), beta_html
        assert "<h2>" not in beta_html and "<h3>" not in beta_html, beta_html
        for chrome in ("sponsr", "CrimsonAlter", "27 февр 11:44", "58 минут"):
            assert chrome not in beta_html, f"Sponsr publication chrome leaked: {beta_html}"
    assert "Дмитриев у Трампов" in alpha_html, alpha_html
    assert "У Европы кризис жанра" in alpha_html, alpha_html
    assert "Fixture article alpha wraps" in alpha_html, alpha_html
    assert "это не инвестиционные рекомендации" in alpha_html, alpha_html
    assert "Традиционные технические предуведомления" in alpha_html, alpha_html
    assert "skladchik.ORG" not in alpha_html, "the final promotional page leaked into RSS content"
    no_redirect = urllib.request.build_opener(NoRedirect())
    for title, filename in (("article-alpha", "article-alpha.pdf"), (beta_title, beta_filename)):
        try:
            request(item_url_by_title[title], opener=no_redirect)
            raise AssertionError("document route did not return a redirect")
        except urllib.error.HTTPError as response:
            assert response.code in (301, 302, 303, 307, 308), response.code
            assert urllib.parse.unquote(response.headers["Location"]) == f"https://fixture.invalid/current-month/{filename}", response.headers["Location"]
    for item in items:
        SafeMarkup().feed(item.findtext("description") or "")
        SafeMarkup().feed(item.findtext(CONTENT_NS) or "")

    basic = "Basic " + __import__("base64").b64encode(
        f"{os.environ['MINIFLUX_USERNAME']}:{os.environ['MINIFLUX_PASSWORD']}".encode()
    ).decode()
    miniflux_headers = {"Authorization": basic}
    categories = wait_for(lambda: request(MINIFLUX + "/v1/categories", headers=miniflux_headers)[1], "Miniflux readiness")
    category_id = categories[0]["id"]
    _, created = request(MINIFLUX + "/v1/feeds", method="POST", payload={
        "feed_url": rss_url, "category_id": category_id,
    }, headers=miniflux_headers)
    miniflux_feed_id = created["feed_id"]

    def read_entries():
        _, result = request(f"{MINIFLUX}/v1/feeds/{miniflux_feed_id}/entries?limit=20&order=id&direction=asc", headers=miniflux_headers)
        entries = result["entries"] if isinstance(result, dict) else result
        return entries if len(entries) == 2 else None

    request(f"{MINIFLUX}/v1/feeds/{miniflux_feed_id}/refresh?force=true", method="PUT", headers=miniflux_headers)
    entries = wait_for(read_entries, "Miniflux to ingest two PDF articles")
    ids_before = {entry["title"]: entry["id"] for entry in entries}
    assert set(ids_before) == {"article-alpha", beta_title}
    miniflux_urls = {entry["title"]: entry["url"] for entry in entries}
    assert miniflux_urls == item_url_by_title, f"Miniflux links must retain exact per-PDF targets: {miniflux_urls}"
    for title, filename in (("article-alpha", "article-alpha.pdf"), (beta_title, beta_filename)):
        try:
            request(miniflux_urls[title], opener=no_redirect)
            raise AssertionError("Miniflux document link did not return a redirect")
        except urllib.error.HTTPError as response:
            assert response.code in (301, 302, 303, 307, 308), response.code
            assert urllib.parse.unquote(response.headers["Location"]) == f"https://fixture.invalid/current-month/{filename}", response.headers["Location"]
    alpha = next(entry for entry in entries if entry["title"] == "article-alpha")
    assert alpha["content"].startswith(f"<h1>{ARTICLE_ALPHA_TITLE}</h1>"), alpha["content"][:800]
    assert f"<p>{ARTICLE_ALPHA_LEAD}</p>" in alpha["content"], alpha["content"][:1200]
    assert "<p>Fixture article alpha wraps across two lines." in alpha["content"], alpha["content"][:1200]
    assert "<p>это не инвестиционные рекомендации</p>" in alpha["content"], alpha["content"][:1200]
    assert "Fixture article alpha" in alpha["content"]
    assert alpha["url"] == expected_links["article-alpha"]

    # Change bytes under the same fixture filename/provider identity, then re-ingest.
    import subprocess
    subprocess.run(["python", "/tests/e2e/generate_fixtures.py", "article-alpha.pdf"], check=True, env={**os.environ, "FIXTURE_ROOT": "/fixtures"})
    if reference_pdf_test:
        os.unlink(f"/fixtures/current-month/{beta_filename}")
    _, discovery = request(f"{APP}/api/management/months/{month_id}/discover", method="POST", opener=app_client)
    assert discovery["discovered_pdfs"] == 4
    _, documents = request(f"{APP}/api/management/months/{month_id}/documents", opener=app_client)
    alpha_doc = next(document for document in documents if document["filename"] == "article-alpha.pdf")
    assert alpha_doc["id"] == by_name["article-alpha.pdf"]["id"], "fixture provider identity changed"
    request(f"{APP}/api/management/documents/{alpha_doc['id']}/extract?force=true", method="POST", opener=app_client)
    _, refreshed_rss = request(rss_url)
    refreshed_items = ET.fromstring(refreshed_rss).findall("./channel/item")
    assert len(refreshed_items) == 2
    refreshed_alpha_item = next(item for item in refreshed_items if item.findtext("title") == "article-alpha")
    refreshed_alpha_html = refreshed_alpha_item.findtext("description") or ""
    refreshed_alpha_encoded = refreshed_alpha_item.findtext(CONTENT_NS) or ""
    assert refreshed_alpha_encoded == refreshed_alpha_html, "refreshed description and encoded content diverged"
    assert refreshed_alpha_html.startswith(f"<h1>{ARTICLE_ALPHA_TITLE}</h1>"), refreshed_alpha_html[:800]
    assert f"<p>{ARTICLE_ALPHA_LEAD}</p>" in refreshed_alpha_html, refreshed_alpha_html[:1200]
    assert "<h2><strong>Обновлённый Дмитриев у Трампов, трудности у танкеров, мегаполисы США у социалистов</strong></h2>" in refreshed_alpha_html, refreshed_alpha_html
    assert "Обновлённый Дмитриев у Трампов" in refreshed_alpha_html
    assert "Fixture article alpha wraps" in refreshed_alpha_html
    assert "это не инвестиционные рекомендации" in refreshed_alpha_html
    assert "skladchik.ORG" not in refreshed_alpha_html
    refreshed_guids = {item.findtext("title"): item.findtext("guid") for item in refreshed_items}
    assert refreshed_guids == guid_by_title, "RSS GUIDs changed during refresh"

    request(f"{MINIFLUX}/v1/feeds/{miniflux_feed_id}/refresh?force=true", method="PUT", headers=miniflux_headers)
    updated_entries = wait_for(read_entries, "Miniflux to ingest updated article content")
    assert len(updated_entries) == 2, "refresh created duplicate Miniflux entries"
    ids_after = {entry["title"]: entry["id"] for entry in updated_entries}
    assert ids_after == ids_before, "Miniflux changed entry identity after content refresh"
    updated_alpha = next(entry for entry in updated_entries if entry["title"] == "article-alpha")
    assert updated_alpha["content"].startswith(f"<h1>{ARTICLE_ALPHA_TITLE}</h1>"), updated_alpha["content"][:800]
    assert f"<p>{ARTICLE_ALPHA_LEAD}</p>" in updated_alpha["content"], updated_alpha["content"][:1200]
    assert "Обновлённый Дмитриев у Трампов, трудности у танкеров, мегаполисы США у социалистов" in updated_alpha["content"]
    print("Miniflux E2E passed: subscription, safe RSS items, promotional exclusion, stable GUIDs, no duplicates, refreshed content.")


if __name__ == "__main__":
    main()
