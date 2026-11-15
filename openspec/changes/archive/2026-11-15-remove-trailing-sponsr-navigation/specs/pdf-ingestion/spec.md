# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Preserve spacing and punctuation between extracted PDF words
The system SHALL reconstruct readable text from extracted PDF words without inserting spaces that split compounds or separate punctuation from adjacent text, while preserving spaces between distinct words.

#### Scenario: Preserve hyphenated compounds
- **WHEN** extracted PDF words form a hyphenated compound such as `поп-психологии`, `по-идиотски`, or `культурно-техническом`
- **THEN** extracted text and published HTML contain the compound without spaces around its hyphen

#### Scenario: Preserve a compound split at a visual line break
- **WHEN** a visual line break separates a hyphenated word into a line ending such as `по-`, `чем-`, or `объяснения-` and a lowercase continuation such as `товарищески`, `нибудь`, or `мастерключа`
- **THEN** extracted text and published HTML join each word without a space after its hyphen, as `по-товарищески`, `чем-нибудь`, and `объяснения-мастерключа`

#### Scenario: Keep punctuation attached to adjacent text
- **WHEN** extracted PDF words include punctuation such as a comma or closing guillemet
- **THEN** extracted text and published HTML contain no space before that punctuation

#### Scenario: Preserve spaces between distinct words
- **WHEN** extracted PDF words are separated as distinct prose words
- **THEN** extracted text and published HTML include the required inter-word spaces

#### Scenario: Preserve italic styling while reconstructing spacing
- **WHEN** a compound, punctuation mark, or closing guillemet spans differently styled extracted words
- **THEN** text spacing and punctuation remain correct and the applicable emphasis markup is preserved
