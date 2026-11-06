#!/usr/bin/env python3
"""End-to-end feed publication and Miniflux reader contract."""

import http.cookiejar
import json
import os
import time
import urllib.request
import urllib.error
import xml.etree.ElementTree as ET
from html.parser import HTMLParser


APP = os.environ["APP_BASE_URL"].rstrip("/")
RSS = os.environ["RSS_BASE_URL"].rstrip("/")
MINIFLUX = os.environ["MINIFLUX_BASE_URL"].rstrip("/")


class SafeMarkup(HTMLParser):
    allowed = {"p", "br", "h1", "h2", "h3", "h4", "strong", "em", "ul", "ol", "li", "blockquote"}

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
    assert discovery["discovered_pdfs"] == 3, discovery
    _, documents = request(f"{APP}/api/management/months/{month_id}/documents", opener=app_client)
    by_name = {document["filename"]: document for document in documents}
    assert set(by_name) == {"article-alpha.pdf", "article-beta.pdf", "club-promotion.pdf"}
    for document in documents:
        request(f"{APP}/api/management/documents/{document['id']}/extract?force=true", method="POST", opener=app_client)

    rss_url = f"{RSS}/feeds/{feed_id}"
    _, rss_bytes = request(rss_url)
    channel = ET.fromstring(rss_bytes)
    items = channel.findall("./channel/item")
    assert len(items) == 2, f"expected two eligible articles, got {len(items)}"
    guid_by_title = {item.findtext("title"): item.findtext("guid") for item in items}
    assert set(guid_by_title) == {"article-alpha", "article-beta"}
    document_id_by_name = {document["filename"]: document["id"] for document in documents}
    item_url_by_title = {item.findtext("title"): item.findtext("link") for item in items}
    expected_links = {
        "article-alpha": f"{RSS}/documents/{document_id_by_name['article-alpha.pdf']}/download",
        "article-beta": f"{RSS}/documents/{document_id_by_name['article-beta.pdf']}/download",
    }
    assert item_url_by_title == expected_links, f"RSS links must be stable per-document redirects: {item_url_by_title}"
    alpha_item = next(item for item in items if item.findtext("title") == "article-alpha")
    alpha_html = alpha_item.findtext("description") or ""
    assert "<h1><strong>Fixture article alpha</strong></h1>" in alpha_html, alpha_html
    assert "<h2><strong>Typography</strong></h2>" in alpha_html, alpha_html
    assert "<p>Fixture article alpha wraps across two lines. <strong>Bold </strong><em>italic</em><strong><em> both</em></strong> styles.</p>" in alpha_html, alpha_html
    assert "skladchik.ORG" not in alpha_html, "the final promotional page leaked into RSS content"
    assert "<p>1</p>" not in alpha_html, "the printed page label leaked into RSS content"
    no_redirect = urllib.request.build_opener(NoRedirect())
    for title, filename in (("article-alpha", "article-alpha.pdf"), ("article-beta", "article-beta.pdf")):
        try:
            request(item_url_by_title[title], opener=no_redirect)
            raise AssertionError("document route did not return a redirect")
        except urllib.error.HTTPError as response:
            assert response.code in (301, 302, 303, 307, 308), response.code
            assert response.headers["Location"] == f"https://fixture.invalid/current-month/{filename}"
    for item in items:
        SafeMarkup().feed(item.findtext("description") or "")

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
    assert set(ids_before) == {"article-alpha", "article-beta"}
    miniflux_urls = {entry["title"]: entry["url"] for entry in entries}
    assert miniflux_urls == item_url_by_title, f"Miniflux links must retain exact per-PDF targets: {miniflux_urls}"
    for title, filename in (("article-alpha", "article-alpha.pdf"), ("article-beta", "article-beta.pdf")):
        try:
            request(miniflux_urls[title], opener=no_redirect)
            raise AssertionError("Miniflux document link did not return a redirect")
        except urllib.error.HTTPError as response:
            assert response.code in (301, 302, 303, 307, 308), response.code
            assert response.headers["Location"] == f"https://fixture.invalid/current-month/{filename}"
    alpha = next(entry for entry in entries if entry["title"] == "article-alpha")
    assert "Fixture article alpha" in alpha["content"]
    assert alpha["url"] == expected_links["article-alpha"]

    # Change bytes under the same fixture filename/provider identity, then re-ingest.
    import subprocess
    subprocess.run(["python", "/tests/e2e/generate_fixtures.py", "article-alpha.pdf"], check=True, env={**os.environ, "FIXTURE_ROOT": "/fixtures"})
    _, discovery = request(f"{APP}/api/management/months/{month_id}/discover", method="POST", opener=app_client)
    assert discovery["discovered_pdfs"] == 3
    _, documents = request(f"{APP}/api/management/months/{month_id}/documents", opener=app_client)
    alpha_doc = next(document for document in documents if document["filename"] == "article-alpha.pdf")
    assert alpha_doc["id"] == by_name["article-alpha.pdf"]["id"], "fixture provider identity changed"
    request(f"{APP}/api/management/documents/{alpha_doc['id']}/extract?force=true", method="POST", opener=app_client)
    _, refreshed_rss = request(rss_url)
    refreshed_items = ET.fromstring(refreshed_rss).findall("./channel/item")
    assert len(refreshed_items) == 2
    refreshed_alpha_item = next(item for item in refreshed_items if item.findtext("title") == "article-alpha")
    refreshed_alpha_html = refreshed_alpha_item.findtext("description") or ""
    assert "<h1><strong>Updated fixture article alpha</strong></h1>" in refreshed_alpha_html, refreshed_alpha_html
    assert "<h2><strong>Typography</strong></h2>" in refreshed_alpha_html, refreshed_alpha_html
    assert "<p>Updated fixture article alpha wraps across two lines. <strong>Bold </strong><em>italic</em><strong><em> both</em></strong> styles.</p>" in refreshed_alpha_html, refreshed_alpha_html
    assert "skladchik.ORG" not in refreshed_alpha_html
    assert "<p>1</p>" not in refreshed_alpha_html
    refreshed_guids = {item.findtext("title"): item.findtext("guid") for item in refreshed_items}
    assert refreshed_guids == guid_by_title, "RSS GUIDs changed during refresh"

    request(f"{MINIFLUX}/v1/feeds/{miniflux_feed_id}/refresh?force=true", method="PUT", headers=miniflux_headers)
    updated_entries = wait_for(read_entries, "Miniflux to ingest updated article content")
    assert len(updated_entries) == 2, "refresh created duplicate Miniflux entries"
    ids_after = {entry["title"]: entry["id"] for entry in updated_entries}
    assert ids_after == ids_before, "Miniflux changed entry identity after content refresh"
    updated_alpha = next(entry for entry in updated_entries if entry["title"] == "article-alpha")
    assert "Updated fixture article alpha" in updated_alpha["content"]
    print("Miniflux E2E passed: subscription, safe RSS items, promotional exclusion, stable GUIDs, no duplicates, refreshed content.")


if __name__ == "__main__":
    main()
