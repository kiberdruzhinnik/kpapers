# Operator guide

## Sign in and create a feed

Open `/manage/` in the web application and sign in with the configured operator username and password. From Overview or Feeds, select **Create feed** to open `/manage/feeds/new`, enter the author's display name and optional description, and create it. Creation opens `/manage/feeds/{feed-id}` so you can configure sources; Cancel returns to `/manage/feeds`. The feed detail view shows the permanent public RSS path `/feeds/{feed-id}`. Editing the name or description keeps this UUID-based URL. Use **Sign out** when finished.

## Choose the PDF extraction engine

The create and edit feed forms include **PDF extraction engine**:

- **Inherit global default** follows the deployment setting. This is the default for existing feeds.
- **Pdfium** always uses the built-in extractor and needs no extra service.
- **Marker** uses the optional private worker and can recover text from scanned PDFs.

The feed detail view shows the saved selection. If Marker is selected but the worker is unavailable, extraction fails visibly and the previous successful cached content remains; the app does not switch to Pdfium automatically. Start the worker and retry, or select Pdfium and refresh. Changing the engine causes the next refresh to reprocess affected PDFs even when the source file is unchanged.

## Archive, restore, or permanently delete

Open a feed’s **More actions** menu or a year’s actions menu to archive that scope. Month actions are under **Manage source**. Use **Archive** to temporarily hide a feed, year, or month from active management, scheduled refresh, and RSS publication. Archived content and identifiers are retained. Restore an archived feed from the **Archived** tab’s contextual menu; restore an archived year or month from its feed details. Restoring a child beneath an archived parent does not publish it until the parent is also restored.

Use **Delete permanently** to remove a feed, year, or month and its descendants, including cached PDFs and refresh history. The UI asks for confirmation. Deleting a feed ends its RSS channel, and a new feed created later receives a different ID. Permanent deletion cannot be undone.

## Add years and month folders

Open `/manage/feeds` and select **Open feed** on an author row. The list separates **Active** and **Archived** feeds; source controls live on the dedicated detail screen. In **Source folders**, select **Add year** to open its focused form. Expand the year and select **Connect month** to open the month-linking form. Years start collapsed and remember their individual expanded or collapsed state in this browser for each feed. A year does not create twelve month records automatically. For each month that has a source folder, choose its calendar month and paste the public folder URL, then select **Link month**. Supported links are HTTPS public folders on `disk.yandex.ru` and `cloud.mail.ru`. Months without links stay absent and are not fetched.

## Review discovered PDFs

Open **Manage source**, then select **Fetch and parse** for a linked month to list its root PDFs and extract/cache native text. The app ignores subfolders. Select **Review PDFs** to open `/manage/pdf-review` with the selected feed/year/month in its URL and inspect discovered files. Each file shows its name, automatic promotional decision, and extraction status. Use the choice menu to publish a file, skip it as promotional, or return it to the automatic decision. The override is stored with the source document and can be changed later. The automatic classification stays visible independently of the override. Including a PDF does not bypass successful extraction; failed PDFs offer **Retry extraction**.

## Public access

RSS readers do not need an operator session. Share the stable `/feeds/{feed-id}` URL with readers. Feed management routes require the operator session cookie.

Month **Last attempt** and **Next refresh** use your browser’s local timezone, including daylight-saving offsets, in `YYYY-MM-DD HH:mm:ss` format. Missing values appear as **Never** and **Pending**.

All dashboard pages use `/manage/`; previous dashboard bookmarks should be updated. Public RSS URLs at `/feeds/{feed-id}` remain unchanged.
