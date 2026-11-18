# Spec Delta

## MODIFIED Requirements

### Requirement: Preserve spacing and punctuation between extracted PDF words
The system SHALL reconstruct readable text from extracted PDF words without inserting spaces that split compounds or separate punctuation from adjacent text. It SHALL remove a line-ending hyphen when joining a lowercase continuation on the next visual line, while preserving spaces between distinct words.

#### Scenario: Preserve hyphenated compounds
- **WHEN** extracted PDF words form a hyphenated compound such as `поп-психологии`, `по-идиотски`, or `культурно-техническом`
- **THEN** extracted text and published HTML contain the compound without spaces around its hyphen

#### Scenario: Preserve a compound split at a visual line break
- **WHEN** a visual line separates a hyphenated compound such as `по-`, `чем-`, or `объяснения-` from a lowercase continuation such as `товарищески`, `нибудь`, or `мастерключа`
- **THEN** extracted text and published HTML join each compound with its hyphen and no surrounding spaces, as `по-товарищески`, `чем-нибудь`, and `объяснения-мастерключа`

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
