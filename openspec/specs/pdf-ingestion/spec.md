# pdf-ingestion Specification

## Purpose

Discovers eligible PDFs in supported public cloud folders, extracts and safely formats their text, and caches source and processed data for feed publication.

## Requirements

### Requirement: Discover only root-level PDFs
The system SHALL list only PDF files directly inside a configured month folder and SHALL NOT traverse subfolders.

#### Scenario: Folder contains a subfolder
- **WHEN** a refresh lists a month folder containing PDFs and a subfolder
- **THEN** only the direct PDF files are considered and the subfolder is ignored

#### Scenario: Folder contains non-PDF files
- **WHEN** a refresh lists a direct child that is not a PDF
- **THEN** the child is not ingested as a feed entry

### Requirement: Extract and sanitize PDF text
The system SHALL extract available text from each eligible PDF and expose it only as HTML composed of an allowlist of basic structural and emphasis tags without executable content or unsafe attributes.

#### Scenario: Text PDF is processed
- **WHEN** an eligible PDF contains extractable text
- **THEN** the system produces readable HTML using only the configured safe tag allowlist

#### Scenario: PDF has no extractable text
- **WHEN** text extraction yields no usable text or fails
- **THEN** the system records an ingestion error and does not publish an empty or unsafe entry

### Requirement: Cache source and processed PDF data
The system SHALL cache each discovered file's provider metadata, content fingerprint, extracted text, and sanitized HTML for reuse between refreshes.

#### Scenario: Source file is unchanged
- **WHEN** a scheduled refresh sees the same source fingerprint as the cached version
- **THEN** the system reuses the cached extraction without downloading and parsing the PDF again

#### Scenario: Source file changes
- **WHEN** a refresh detects a changed source fingerprint
- **THEN** the system downloads and processes the changed PDF and updates its cached version

### Requirement: Validate provider PDF download destinations
Before fetching a discovered PDF, the system SHALL require an HTTPS download URL whose resolved destination addresses are public, and SHALL reject private or otherwise non-public network destinations without making the request. The same policy SHALL apply to every redirect destination.

#### Scenario: PDF download uses a public HTTPS host outside the provider's usual domains
- **WHEN** a provider returns an HTTPS download URL on a public network host
- **THEN** the system may request that destination subject to the existing timeout, response-size, and redirect controls

#### Scenario: Yandex Disk returns an allowed download destination
- **WHEN** a Yandex Disk PDF is fetched from an HTTPS destination that resolves to public addresses
- **THEN** the system may request that destination subject to the existing timeout, response-size, and redirect controls

#### Scenario: Yandex Disk returns a destination outside its allowlist
- **WHEN** a Yandex Disk PDF download URL uses a public HTTPS host outside the former provider allowlist
- **THEN** the system may request that destination subject to the same public-address, timeout, response-size, and redirect controls

#### Scenario: Mail Cloud returns an allowed download destination
- **WHEN** a Mail Cloud PDF is fetched from an HTTPS destination that resolves to public addresses
- **THEN** the system may request that destination subject to the existing timeout, response-size, and redirect controls

#### Scenario: Mail Cloud returns a destination outside its allowlist
- **WHEN** a Mail Cloud PDF download URL uses a public HTTPS host outside the former provider allowlist
- **THEN** the system may request that destination subject to the same public-address, timeout, response-size, and redirect controls

#### Scenario: Download URL resolves to a private or otherwise non-public address
- **WHEN** a provider returns an HTTPS download URL whose destination resolves to a private or otherwise non-public address
- **THEN** the system rejects the URL before connecting and records the document extraction as failed

#### Scenario: Redirect targets a private or otherwise non-public address
- **WHEN** a download redirects to a destination that resolves to a private or otherwise non-public address
- **THEN** the system stops the redirect and records the document extraction as failed

#### Scenario: Download destination uses a non-HTTPS scheme
- **WHEN** a provider returns a PDF download URL that does not use HTTPS
- **THEN** the system rejects the URL before making the download request and records the extraction as failed

#### Scenario: Download destination is rejected during refresh
- **WHEN** a provider returns an invalid PDF download destination for a document with previously cached content
- **THEN** the system records the refresh failure and retains the last successfully processed cached entry

### Requirement: Preserve cached entries on refresh failure
The system SHALL retain the last successfully processed version of an entry when a later provider request or extraction attempt fails.

#### Scenario: Provider is unavailable
- **WHEN** a refresh cannot list or download a source file
- **THEN** the system keeps the last successful cached entry and records the refresh failure

### Requirement: Remove a recognized boundary-page advertisement
The system SHALL omit a dedicated Skladchik promotional page from extracted text when it appears as the first or final PDF page. Recognition SHALL support both the Club advertisement and the “куплено на skladchik.ORG” advertisement.

#### Scenario: PDF begins with the purchased-on Skladchik advertisement
- **WHEN** the first page begins with “куплено на” and contains `skladchik.ORG` as the advertisement's destination
- **THEN** the first page is excluded from extracted text and published HTML

#### Scenario: PDF ends with the Skladchik Club advertisement
- **WHEN** the final page contains the `skladchik.ORG`, “Ты еще не в Клубе?”, and “Что такое Клуб «Складчик»?” promotional text
- **THEN** the final page is excluded from the extracted text and published HTML

#### Scenario: A boundary page contains ordinary article text
- **WHEN** a first or final page mentions `skladchik.ORG` without matching either recognized advertisement pattern
- **THEN** the page remains in extracted text and published HTML

### Requirement: Omit Sponsr publication preamble
The system SHALL omit recognizable Sponsr publication chrome and article metadata that precede an identifiable article title, so published content begins with the article title. The article body SHALL remain intact.

#### Scenario: Sponsr metadata precedes the article title
- **WHEN** a PDF contains a Sponsr masthead, author/byline, publication date/time, or reading duration before an identifiable article title
- **THEN** the preamble is omitted, the article title is the first published content, and the article body is preserved

#### Scenario: Remove the Sponsr date and reading duration shown in the publication header
- **WHEN** a Sponsr PDF contains `27 февр 11:44` and a reading duration before the headline `[Философское] «Старые деньги» — FAQ: как правильно считать дивиденды, «бессмертные» портфели и стоицизм`
- **THEN** the published HTML begins with that headline as its first heading, omits both metadata values and the preceding Sponsr chrome, and preserves the content following the headline

#### Scenario: Sponsr PDF title is present in the filename but not the text layer
- **WHEN** the PDF's first extracted heading contains publication date/time and duration, its text layer omits the article title, and its filename contains the `[Философское]` headline
- **THEN** the filename headline is emitted as the first `<h1>`, the publication metadata is removed from extracted text, the article body remains, and XML-invalid control characters do not break RSS

#### Scenario: No recognizable Sponsr preamble and title are present
- **WHEN** a PDF does not contain recognizable Sponsr publication chrome followed by an identifiable article title
- **THEN** leading text is preserved

### Requirement: Omit page labels from extracted text
The system SHALL omit a standalone numeric page label when it matches that page's one-based position among the pages remaining after leading boundary advertisement pages are removed, regardless of where the label appears in the extracted line order. Numbers that are part of content or do not match the page position SHALL be preserved.

#### Scenario: Page starts with its printed page number
- **WHEN** the first non-empty extracted line of a page is only that page's one-based content-page number
- **THEN** the number is omitted from extracted text

#### Scenario: Page number follows extracted page content
- **WHEN** a standalone line containing only the page's one-based content-page number appears after other extracted lines on that page
- **THEN** the number is omitted from extracted text and published HTML

#### Scenario: Leading advertisement page shifts printed numbering
- **WHEN** a PDF begins with a boundary advertisement page that is removed and the first remaining content page prints `1`
- **THEN** that `1` and later matching standalone page numbers are omitted from extracted text and published HTML

#### Scenario: A number appears in page content
- **WHEN** a number appears as part of a line with other content or does not match the page's content-page position
- **THEN** the number is preserved

### Requirement: Join wrapped lines within paragraphs
The system SHALL join line-wrapped text within a paragraph using spaces while preserving paragraph boundaries, including when a paragraph continues across adjacent PDF pages. A large vertical gap alone SHALL NOT split a sentence when the text and layout indicate that the next line continues it. The honorific abbreviation `ув.` followed by an uppercase continuation SHALL be treated as continuation evidence when layout and styling are compatible.

#### Scenario: Prose wraps across extracted lines
- **WHEN** a paragraph is extracted as multiple lines separated by line breaks
- **THEN** those lines are joined into continuous prose with spaces and the paragraph remains a single HTML paragraph

#### Scenario: Separate paragraphs are extracted
- **WHEN** paragraphs are separated in the extracted text
- **THEN** each paragraph remains distinct in the resulting HTML

#### Scenario: A prose paragraph continues across a page boundary
- **WHEN** a page ends mid-sentence and the next page begins with its continuation
- **THEN** the text on both pages is joined with whitespace in one paragraph in extracted text and published HTML

#### Scenario: A paragraph boundary falls at a page boundary
- **WHEN** the PDF layout indicates that the preceding paragraph ends and a new paragraph begins on the next page
- **THEN** the paragraphs remain distinct in extracted text and published HTML

#### Scenario: A sentence continues after a large vertical gap
- **WHEN** a line ends with continuation punctuation such as a comma and the following line begins as a grammatical continuation, with compatible text styling and alignment despite a large vertical gap
- **THEN** both lines remain in one paragraph in extracted text and published HTML

#### Scenario: A wrapped line continues after the honorific abbreviation
- **WHEN** a line ends with `ув.` and the following line begins with an uppercase continuation, as in `... экспресса» ув.` followed by `Александр Романов ...`, with compatible styling and alignment
- **THEN** both lines remain in one paragraph in extracted text and published HTML, even if a large vertical gap would otherwise suggest a paragraph boundary

#### Scenario: A wrapped line continues with the capitalized word after the honorific abbreviation
- **WHEN** a line ends with `ув.` and the following line begins `Собеседника.` as in `... стартового тезиса ув.` followed by `Собеседника. Идея ...`, with compatible styling and alignment
- **THEN** both lines remain in one paragraph in extracted text and published HTML

#### Scenario: An ordinary sentence-ending period does not imply continuation
- **WHEN** a line ends with a period that is not the recognized honorific abbreviation and the PDF layout indicates a paragraph boundary before the next line
- **THEN** the paragraphs remain distinct in extracted text and published HTML

#### Scenario: A large gap marks a genuine paragraph boundary
- **WHEN** the PDF layout indicates that a large vertical gap separates complete prose paragraphs and no continuation evidence applies
- **THEN** the paragraphs remain distinct in extracted text and published HTML

#### Scenario: A lowercase sentence continuation follows a page-ending conjunction
- **WHEN** a page ends with `потому что` and the next page begins with `любой вопрос есть вопрос добра и зла`, as in the supplied PDF, with layout indicating the same paragraph
- **THEN** the extracted text and published HTML contain `потому что любой вопрос есть вопрос добра и зла` in one paragraph

### Requirement: Join parenthetical continuations across page boundaries
The system SHALL keep a prose paragraph continuous across a PDF page boundary when the next page begins with a parenthetical continuation and the text and layout indicate that it continues the preceding paragraph.

#### Scenario: Page continuation begins with a parenthetical phrase
- **WHEN** a page ends mid-sentence and the next page begins with an opening parenthesis followed by lowercase prose, with compatible styling and alignment
- **THEN** both pages' text is joined with whitespace in one paragraph in extracted text and published HTML

#### Scenario: Parenthetical continuation follows a completed sentence
- **WHEN** a page ends with a completed sentence and the next page begins a parenthetical phrase as a new paragraph
- **THEN** the paragraphs remain distinct in extracted text and published HTML

### Requirement: Render PDF headings with semantic levels
The system SHALL identify document titles and section headings from corroborating PDF typography and layout cues and render them using semantic heading tags from `<h1>` through `<h3>` according to their document hierarchy. The document title uses `<h1>`, top-level headings use `<h2>`, and nested headings use `<h3>`.

#### Scenario: Render document title and supported section hierarchy
- **WHEN** a PDF contains a document title and visually distinct section and subsection headings supported by typography and layout cues
- **THEN** the title is rendered as `<h1>`, sections as `<h2>`, and nested headings as `<h3>`

### Requirement: Preserve title and lead as distinct content blocks
The system SHALL preserve a visually distinct document title and introductory lead as separate semantic blocks in extracted HTML, followed by body content in its own paragraphs.

#### Scenario: PDF title is followed by an introductory lead
- **WHEN** a PDF presents a visually distinct title followed by an introductory paragraph and body paragraphs
- **THEN** extracted HTML represents the title, introductory paragraph, and body paragraphs as distinct elements in the same reading order

#### Scenario: Title wraps across visual lines
- **WHEN** a document title spans multiple visual lines before an introductory paragraph
- **THEN** the title lines are combined into one title element and the introductory paragraph remains a separate element

#### Scenario: Long CrimsonAnalytics headline wraps across visual lines
- **WHEN** a CrimsonAnalytics headline longer than 160 characters spans multiple visual lines before an introductory paragraph
- **THEN** the joined headline is emitted as one `<h1>`, and the introductory paragraph remains a separate element

#### Scenario: Supplied CrimsonAnalytics headline is reconstructed as one heading
- **WHEN** the supplied headline begins `[Новостной деск] Последствия ударов по Крымскому мосту` and continues across visual lines through `возобновления «горячей» войны с Ираном`
- **THEN** the complete title `[Новостной деск] Последствия ударов по Крымскому мосту, контуры «плана Кушнера» по продаже «украинского будущего» лондонским покровителям Зеленского и пять причин в пользу возобновления «горячей» войны с Ираном` is emitted once as one `<h1>`, with the lead remaining a separate element

### Requirement: Preserve the complete wrapped CrimsonAnalytics headline
The system SHALL combine the visual lines of the supplied CrimsonAnalytics headline into one `<h1>` and keep the following literal Markdown disclaimer in its own bold paragraph.

#### Scenario: Parse the supplied Coffee with a Banker headline and disclaimer
- **WHEN** the PDF contains the headline beginning `[Кофе с банкиром] Последствия поражения Трампа` and ending `заработках на деклассированных элементах и спонсорах Демпартии.` across wrapped lines, followed by `**это не инвестиционные рекомендации**`
- **THEN** published HTML begins with the complete headline in one `<h1>`, followed by `<p><strong>**это не инвестиционные рекомендации**</strong></p>`

### Requirement: Keep ordinary prose as paragraph content
Font size or spacing alone SHALL NOT cause ordinary prose to be classified as a heading. Text that would otherwise be classified below `<h3>` SHALL remain paragraph content.

#### Scenario: Preserve ordinary prose as paragraphs
- **WHEN** extracted text is ordinary body prose rather than a heading
- **THEN** it remains rendered as paragraph content and is not promoted to a heading

#### Scenario: Keep visually prominent prose as paragraphs
- **WHEN** a body paragraph is larger or more widely spaced than surrounding text but has no corroborating heading cues
- **THEN** the paragraph remains in a `<p>` element and is not promoted to `<h2>` or `<h3>`

#### Scenario: Keep lower-level heading candidates as paragraphs
- **WHEN** typography and layout would classify a line below the supported `<h3>` level
- **THEN** its text is rendered as a paragraph and no `<h4>` element is emitted

#### Scenario: Keep Sponsr body blocks distinct after a recovered filename headline
- **WHEN** a Sponsr PDF has an image-only `[Философское]` headline recovered from its filename, followed by a bold disclaimer, ordinary paragraphs, an indented notice, and more article text
- **THEN** the headline is the first `<h1>`, the disclaimer remains a paragraph with `<strong>`, ordinary text remains in `<p>` elements, the notice is grouped in one `<blockquote>`, and body text is not promoted to `<h2>` or `<h3>` based on font size alone

### Requirement: Combine wrapped heading lines
Visual continuation lines of one logical heading SHALL be combined with whitespace between words.

#### Scenario: Keep a wrapped document title as one heading
- **WHEN** the document title wraps across multiple PDF lines but remains one logical title
- **THEN** all title text is rendered in one `<h1>` with spaces between the wrapped lines, not split into separate heading levels

#### Scenario: Keep a wrapped section headline as one heading
- **WHEN** a top-level headline wraps across multiple PDF lines but remains one logical heading
- **THEN** all headline text is rendered in one `<h2>` with spaces between the wrapped lines

### Requirement: Preserve bold and italic PDF styling
The system SHALL represent bold text with `<strong>` and italic text with `<em>`, including both tags when a text run has both styles, while leaving normally styled text unwrapped. Adjacent text runs with the same style SHALL be merged into continuous semantic tags within a paragraph or heading, including when separated only by a whitespace run; spaces and actual block boundaries SHALL be preserved. For the literal investment disclaimer `**это не инвестиционные рекомендации**`, the marker characters SHALL remain visible inside its generated `<strong>` element.

#### Scenario: Render bold and italic runs
- **WHEN** a PDF text run uses bold or italic styling
- **THEN** the corresponding text is wrapped in `<strong>` or `<em>` in the resulting HTML

#### Scenario: Render text with both styles
- **WHEN** a PDF text run uses both bold and italic styling
- **THEN** the resulting HTML nests `<strong>` and `<em>` around that text

#### Scenario: Preserve normal text
- **WHEN** a PDF text run has neither bold nor italic styling
- **THEN** the resulting HTML does not add emphasis tags around it

#### Scenario: Merge adjacent runs with the same style
- **WHEN** consecutive runs within one paragraph use the same italic style
- **THEN** their text is rendered inside one continuous `<em>` element with intervening spaces preserved

#### Scenario: Merge same-style runs inside a heading
- **WHEN** date and time text inside one heading are emitted as adjacent bold runs separated only by whitespace
- **THEN** both values are rendered inside one `<strong>` element with one space between them

#### Scenario: Preserve nested style changes and paragraph boundaries
- **WHEN** a bold run appears inside an italic paragraph and the document has a paragraph break before the next italic text
- **THEN** bold is nested inside the continuous italic span, and the next paragraph remains a separate paragraph

#### Scenario: Interpret Markdown bold markers in extracted PDF text
- **WHEN** extracted text contains a matched Markdown bold span other than the literal investment disclaimer `**это не инвестиционные рекомендации**`
- **THEN** the marker characters are removed and the enclosed text is emitted inside `<strong>` tags, combined with any PDF font styling without duplicate or fragmented emphasis tags

#### Scenario: Preserve literal markers in the investment disclaimer
- **WHEN** extracted text contains the literal investment disclaimer `**это не инвестиционные рекомендации**`
- **THEN** the full literal span, including both pairs of asterisks, is emitted inside `<strong>` tags without duplicate or fragmented emphasis tags

#### Scenario: Preserve unmatched Markdown marker characters
- **WHEN** extracted text contains unmatched `**` marker characters
- **THEN** the unmatched characters remain ordinary escaped text and do not cause the surrounding content to be dropped or incorrectly emphasized

#### Scenario: Keep an explicit bold disclaimer in a paragraph
- **WHEN** a PDF contains the literal disclaimer `**это не инвестиционные рекомендации**` after its title
- **THEN** it is rendered as `<p><strong>**это не инвестиционные рекомендации**</strong></p>`, not as a heading, with the marker characters preserved

#### Scenario: Preserve a bold word within regular prose
- **WHEN** a PDF paragraph contains a word set in bold between normally styled words
- **THEN** the word is emitted inside `<strong>` while surrounding text remains unwrapped and in the original reading order

### Requirement: Render visually indented notice regions as blockquotes
The system SHALL identify a notice heading such as “Традиционные технические предуведомления:” and its visually indented continuation region from PDF text position and layout. It SHALL wrap that region in a `<blockquote>`, preserve the heading and paragraph boundaries inside it, and end the blockquote when the text returns to the ordinary body margin. It SHALL preserve ordinary body text outside the indented region.

#### Scenario: Render the technical notice and its indented contents as a blockquote
- **WHEN** “Традиционные технические предуведомления:” begins a visually indented region followed by notice paragraphs
- **THEN** the heading and indented notice paragraphs are contained in one `<blockquote>` with their paragraph and inline emphasis markup preserved

#### Scenario: End a blockquote when ordinary body text resumes
- **WHEN** text following an indented notice region returns to the ordinary body margin
- **THEN** the `<blockquote>` ends before that text, which remains outside the blockquote as ordinary document content

### Requirement: Omit standalone Sponsr branding from extracted PDF content
The system SHALL omit the standalone branding line `Кримсон Аналитика – CrimsonAnalytics` from extracted text and published HTML wherever it appears as publication chrome in a PDF.

#### Scenario: Branding appears as its own line on a later page
- **WHEN** an eligible PDF contains the branding line as a standalone line on any page, including after the first page
- **THEN** the line is absent from cached extracted text and published HTML while surrounding article content is preserved

#### Scenario: Branding is part of an article sentence
- **WHEN** an eligible PDF mentions the same branding text as part of a larger article sentence or paragraph
- **THEN** the article text is preserved in extracted text and published HTML

### Requirement: Omit trailing Sponsr navigation after the investment disclaimer
For a Sponsr article PDF with the recognizable investment disclaimer near its end, the system SHALL preserve the complete disclaimer block and omit subsequent series navigation and related-content material from extracted text and published HTML.

#### Scenario: Trailing series label and related article follow the disclaimer
- **WHEN** a Sponsr PDF contains the investment disclaimer followed by a series label, navigation controls, or a related-article teaser
- **THEN** the complete disclaimer remains in extracted text and published HTML, and the subsequent navigation and related-content material is omitted

#### Scenario: Related article continues onto another page
- **WHEN** a related-article teaser begins after the investment disclaimer and its title or date continues onto a following page
- **THEN** the teaser content is omitted across the page boundary while the disclaimer and all preceding article content remain

#### Scenario: No recognizable investment disclaimer is present
- **WHEN** a PDF has no recognizable investment disclaimer near its end
- **THEN** trailing text is preserved and this cleanup does not remove content

### Requirement: Reprocess content when the effective engine changes
The system SHALL include the effective extraction engine and processing revision in cache identity and reprocess affected documents on their next refresh, even if their source fingerprint is unchanged.

#### Scenario: Feed changes engines
- **WHEN** a cached document is refreshed after its effective engine changes
- **THEN** the system extracts it with the newly selected engine instead of reusing the previous engine's cache

#### Scenario: Extraction fails
- **WHEN** the selected engine fails or is unavailable
- **THEN** the system records the failure and continues publishing the last successfully processed content without silently switching engines

#### Scenario: Cached content is reused after a failed refresh
- **WHEN** a later refresh reuses a matching successful cache after an extraction failure
- **THEN** the cached entry remains available in RSS and the document's recovery state reflects the successful reuse

### Requirement: Extract eligible PDFs with pdf_oxide default text extraction
The system SHALL use the Rust `pdf_oxide` default plain-text extraction path for every eligible PDF, without OCR or image embedding, and SHALL NOT switch to another extraction engine when conversion fails.

#### Scenario: Convert an eligible multi-page PDF
- **WHEN** an eligible PDF is refreshed
- **THEN** `pdf_oxide` extracts each page with `PdfDocument::extract_text_lines`, preserving available text, run styling, and page reading order for the existing publication pipeline

#### Scenario: Conversion fails
- **WHEN** `pdf_oxide` cannot open or extract text from a PDF
- **THEN** the system records an ingestion error and preserves the last successfully processed content

#### Scenario: Scanned-only PDF has no extractable text
- **WHEN** a PDF contains only scanned pages and OCR is not enabled
- **THEN** the system reports extraction failure rather than publishing empty or incomplete output

### Requirement: Apply publication processing to extracted text
The system SHALL apply PDF cleanup, promotional classification, publication HTML generation, and safe HTML requirements to text produced by `pdf_oxide`.

#### Scenario: Conversion encounters recognized publication chrome
- **WHEN** `pdf_oxide` extracts text from a PDF containing recognized boundary advertisements or Sponsr publication chrome
- **THEN** the applicable cleanup rules remove that content while preserving article content

#### Scenario: Converted content contains unsafe markup
- **WHEN** extracted text contains strings that resemble HTML or content that could form markup outside the permitted HTML policy
- **THEN** published HTML excludes unsafe elements and attributes

### Requirement: Reprocess content when the processing pipeline changes
The system SHALL include the `pdf_oxide` engine identity and processing revision in cache identity and reprocess affected documents on their next refresh when the processing revision changes, even if their source fingerprint is unchanged.

#### Scenario: Processing revision changes
- **WHEN** a cached document is refreshed after the PDF processing revision changes
- **THEN** the system converts it with `pdf_oxide` instead of reusing content produced by an earlier pipeline revision

#### Scenario: Conversion fails
- **WHEN** `pdf_oxide` fails or is unavailable
- **THEN** the system records the failure and continues publishing the last successfully processed content without silently switching engines

#### Scenario: Cached content is reused after a failed refresh
- **WHEN** a later refresh reuses a matching successful cache after a conversion failure
- **THEN** the cached entry remains available in RSS and the document's recovery state reflects the successful reuse

### Requirement: Preserve spacing and punctuation between extracted PDF words
The system SHALL reconstruct readable text from extracted PDF words without inserting spaces that split compounds or separate punctuation from adjacent text. It SHALL remove a discretionary line-ending hyphen when joining a lowercase continuation on the next visual line, while preserving spaces between distinct words.

#### Scenario: Preserve hyphenated compounds
- **WHEN** extracted PDF words form a hyphenated compound such as `поп-психологии`, `по-идиотски`, or `культурно-техническом`
- **THEN** extracted text and published HTML contain the compound without spaces around its hyphen

#### Scenario: Preserve a compound split at a visual line break
- **WHEN** a visual line break separates a hyphenated word into a line ending such as `по-`, `чем-`, or `объяснения-` and a lowercase continuation such as `товарищески`, `нибудь`, or `мастерключа`
- **THEN** extracted text and published HTML join each word without a space after its hyphen, as `по-товарищески`, `чем-нибудь`, and `объяснения-мастерключа`

#### Scenario: Remove a discretionary line-ending hyphen
- **WHEN** a visual line ends with `пристрасти-` or `неискушен-` and the next line begins with `лись` or `ными`
- **THEN** extracted text and published HTML join the fragments as `пристрастились` and `неискушенными`, without a hyphen or space

#### Scenario: Keep punctuation attached to adjacent text
- **WHEN** extracted PDF words include punctuation such as a comma or closing guillemet
- **THEN** extracted text and published HTML contain no space before that punctuation

#### Scenario: Preserve spaces between distinct words
- **WHEN** extracted PDF words are separated as distinct prose words
- **THEN** extracted text and published HTML include the required inter-word spaces

#### Scenario: Preserve italic styling while reconstructing spacing
- **WHEN** a compound, punctuation mark, or closing guillemet spans differently styled extracted words
- **THEN** text spacing and punctuation remain correct and the applicable emphasis markup is preserved

### Requirement: Preserve conventional prose word spacing
The system SHALL preserve conventional spaces at prose word boundaries when reconstructing PDF text, even when the measured word gap is narrow or absent.

#### Scenario: Parenthesis follows a word in prose
- **WHEN** extracted prose contains a word immediately followed by an opening parenthesis in the PDF word sequence
- **THEN** the reconstructed text separates them with a space, as in `элите (если`, `раз (с точки зрения её авторов) находилась`, and `в сутки (б/с)`

#### Scenario: Parenthesis follows a conjunction in prose
- **WHEN** extracted prose contains a conjunction immediately followed by an opening parenthesis in the PDF word sequence
- **THEN** the reconstructed text separates them with a space, as in `руководства и (возможно)` and `должен был (и мог бы)`

#### Scenario: Parenthesis follows an acronym or phrase in prose
- **WHEN** extracted prose has a parenthetical acronym or explanation immediately after a word
- **THEN** the reconstructed text separates them with a space, as in `Ирана (КСИР)` and `круто (с точки зрения наших оппонентов)`

#### Scenario: Opening guillemet follows a word in prose
- **WHEN** extracted prose has an opening guillemet directly after a word in the PDF word sequence
- **THEN** the reconstructed text separates the word and quote, as in `как «сопливость»`, `про «суть»`, `по «несанкционированным» маршрутам`, and `за „фонды поддержки“`

#### Scenario: Closing quote precedes a following word
- **WHEN** a quoted phrase is followed by a word in the PDF word sequence
- **THEN** the reconstructed text separates the closing quote from that word, as in `„ехать“ из`, `„мир“ на`, `„грязной бомбы“ в`, and `„Грязная бомба“ использует`

#### Scenario: Latin and Cyrillic text meet at a word boundary
- **WHEN** adjacent PDF words cross between Latin and Cyrillic letters and geometry reports a narrow gap
- **THEN** the reconstructed text separates the words, as in `из Rаpporteur` and `такую как Wildberries`

#### Scenario: A letter is followed by a number
- **WHEN** adjacent PDF words place a number directly after a word and geometry reports a narrow gap
- **THEN** the reconstructed text separates them, as in `после 4`, `марте 2026`, and `войны 28`

#### Scenario: A Cyrillic preposition is followed by a number
- **WHEN** a number directly follows a Cyrillic preposition in the PDF word sequence and geometry reports a narrow gap
- **THEN** the reconstructed text separates them, as in `с 8,372`

#### Scenario: Closing punctuation remains adjacent
- **WHEN** a PDF word sequence contains a comma or closing parenthesis after a word
- **THEN** reconstructed text keeps that punctuation adjacent to the preceding word

### Requirement: Remove CrimsonAnalytics publication header before the article title
The system SHALL omit recognizable CrimsonAnalytics branding and publication date and reading-duration metadata that precede an identifiable article title.

#### Scenario: CrimsonAnalytics publication metadata precedes the headline
- **WHEN** a PDF begins with CrimsonAnalytics branding, a publication date and time, a reading duration, and then an identifiable article headline
- **THEN** extracted text and published HTML omit the branding and metadata and begin with the article headline

### Requirement: Preserve lexical hyphens across visual line wraps
The system SHALL retain a lexical hyphen when a hyphenated word is split across visual lines and the hyphen belongs to the compound rather than marking discretionary word wrapping.

#### Scenario: Hyphenated compound wraps between its components
- **WHEN** a PDF line ends with `проповедники-` and the next line begins with `фундаменталисты`
- **THEN** reconstructed text joins the lines as `проповедники-фундаменталисты`

### Requirement: Preserve supported public folder input hosts
The system SHALL continue accepting configured public folder input URLs only on `cloud.mail.ru` and `disk.yandex.ru`, subject to their existing URL shape and validation rules. Download destination host policy SHALL NOT broaden input URL validation.

#### Scenario: Configure a Mail Cloud public folder
- **WHEN** an operator provides a supported HTTPS public folder URL on `cloud.mail.ru`
- **THEN** the system accepts it using the existing Mail Cloud path and validation rules

#### Scenario: Configure a Yandex Disk public folder
- **WHEN** an operator provides a supported HTTPS public folder URL on `disk.yandex.ru`
- **THEN** the system accepts it using the existing Yandex Disk path and validation rules

#### Scenario: Configure a public folder URL on another host
- **WHEN** an operator provides a public folder URL on a host other than `cloud.mail.ru` or `disk.yandex.ru`
- **THEN** the system rejects it even if that host is publicly reachable

### Requirement: Report actionable errors for failed PDFs
When processing a PDF fails, the system SHALL retain and show an actionable error for that specific file, including its filename and the processing stage or concrete failure reason, without exposing credentials or ephemeral download secrets.

#### Scenario: A PDF fails during scheduled refresh
- **WHEN** processing a discovered PDF fails during a scheduled refresh
- **THEN** the refresh result identifies the filename and its specific failure reason instead of reporting only a generic PDF extraction failure

#### Scenario: An operator reviews a failed PDF
- **WHEN** an operator opens the PDF review queue for a failed document
- **THEN** the document displays its filename and actionable stored failure reason

#### Scenario: A failed PDF is retried successfully
- **WHEN** a retry successfully processes a previously failed PDF
- **THEN** the stored failure reason is cleared and the document is no longer reported as needing attention for that failure

### Requirement: Embed optimized PDF images near their original reading positions
The system SHALL include supported embedded PDF images in processed article HTML near the text at their original page positions, preserving page reading order.

#### Scenario: Article contains positioned images
- **WHEN** an eligible PDF contains extractable text and embedded images with page positions
- **THEN** processed HTML includes each successfully processed image near the corresponding text in page reading order

#### Scenario: Image position cannot be determined
- **WHEN** an embedded image has no usable page position
- **THEN** the image is placed after the text for its page and before content from later pages

### Requirement: Optimize inline PDF images for web publication
The system SHALL convert embedded images to a browser-supported lossy format, limit output width to 1200 pixels, compress at quality 80, and remove image metadata before embedding.

#### Scenario: Image exceeds the width limit
- **WHEN** an extracted image is wider than 1200 pixels
- **THEN** the embedded image is resized to no more than 1200 pixels wide while preserving its aspect ratio

#### Scenario: Image already fits the width limit
- **WHEN** an extracted image is 1200 pixels wide or narrower
- **THEN** its width is not increased and it is compressed at quality 80 with metadata removed

#### Scenario: Source image format is not supported for web output
- **WHEN** an extracted image uses a format that the selected web output format cannot encode directly
- **THEN** the system decodes and converts it before lossy compression and embedding

#### Scenario: PDF image uses JPEG 2000 encoding
- **WHEN** an eligible PDF contains an embedded image using the `/JPXDecode` filter
- **THEN** the system decodes the image and processes it through the same sizing, compression, and inline embedding requirements

### Requirement: Keep embedded image markup safe and bounded
The system SHALL publish image data only through validated inline data URIs in the allowed image format, without arbitrary attributes or externally loaded image URLs.

#### Scenario: Processed image is embedded
- **WHEN** processed HTML includes an image
- **THEN** the image source is an inline data URI for the selected image format and the markup contains no executable content or unsafe attributes

#### Scenario: Image processing fails
- **WHEN** an embedded image cannot be decoded, converted, or compressed
- **THEN** the system omits that image and continues publishing the successfully extracted article text and other images
