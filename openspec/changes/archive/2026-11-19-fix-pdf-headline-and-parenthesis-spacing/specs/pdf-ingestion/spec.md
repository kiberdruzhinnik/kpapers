# Spec Delta

## ADDED Requirements

### Requirement: Preserve conventional prose word spacing
The system SHALL preserve conventional spaces at prose word boundaries when reconstructing PDF text, even when the measured word gap is narrow or absent. This includes boundaries before parenthetical phrases and opening guillemets, after closing quotes before a following word, between Latin and Cyrillic text, and between letters and digits.

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

## MODIFIED Requirements

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
