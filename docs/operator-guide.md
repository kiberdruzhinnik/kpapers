# Operator guide

## Sign in and create a feed

Open the web application and sign in with the configured operator username and password. Select **Create a feed**, enter the author's display name and optional description, and create it. The feed detail view shows the permanent public RSS path `/feeds/{feed-id}`. Editing the name or description keeps this UUID-based URL. Use **Sign out** when finished.

## Archive, restore, or permanently delete

Use **Archive** to temporarily hide a feed, year, or month from active management, scheduled refresh, and RSS publication. Archived content and identifiers are retained. Restore an archived feed from the archived-feeds list; restore an archived year or month from its feed details. Restoring a child beneath an archived parent does not publish it until the parent is also restored.

Use **Delete permanently** to remove a feed, year, or month and its descendants, including cached PDFs and refresh history. The UI asks for confirmation. Deleting a feed ends its RSS channel, and a new feed created later receives a different ID. Permanent deletion cannot be undone.

## Add years and month folders

Select a feed and add a year. A year does not create twelve month records automatically. For each month that has a source folder, choose its calendar month and paste the public folder URL, then select **Link month**. Supported links are HTTPS public folders on `disk.yandex.ru` and `cloud.mail.ru`. Months without links stay absent and are not fetched.

## Review discovered PDFs

Select **Fetch and parse** for a linked month to list its root PDFs and extract/cache native text. The app ignores subfolders. Select **Review PDFs** to inspect discovered files. Each file shows its name, automatic promotional decision, and extraction status. Use the choice menu to publish a file, skip it as promotional, or return it to the automatic decision. The override is stored with the source document and can be changed later.

## Public access

RSS readers do not need an operator session. Share the stable `/feeds/{feed-id}` URL with readers. Feed management routes require the operator session cookie.
