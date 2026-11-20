# Spec Delta

## MODIFIED Requirements

### Requirement: Preserve lexical hyphens across visual line wraps
The system SHALL retain lexical hyphens when a hyphenated compound is split across visual lines or adjacent PDF pages, without adding whitespace around the hyphen. It SHALL remove a line-ending hyphen only when that hyphen marks discretionary word wrapping.

#### Scenario: Hyphenated compound wraps between its components
- **WHEN** a PDF line ends with `проповедники-` and the next line begins with `фундаменталисты`
- **THEN** reconstructed text joins the lines as `проповедники-фундаменталисты`

#### Scenario: Lowercase lexical compounds wrap within a page or across a page boundary
- **WHEN** a visual line or page ends with `какой-` before `либо`, or `государств-` before `членов`
- **THEN** extracted text and published HTML preserve the lexical hyphen and join each continuation without whitespace, as `какой-либо` and `государств-членов`

#### Scenario: Capitalized lexical compounds wrap across visual lines
- **WHEN** a visual line ends with `Украина-` before `Россия`, `Косиняк-` before `Камыш`, or `Эр-` before `Рияде`
- **THEN** extracted text and published HTML preserve the lexical hyphen and join the components without whitespace as `Украина-Россия`, `Косиняк-Камыш`, and `Эр-Рияде`

#### Scenario: A discretionary word hyphen wraps across a visual line
- **WHEN** a visual line ends with `пристрасти-` before `лись` or `неискушен-` before `ными`, and the hyphen marks word wrapping rather than a lexical compound
- **THEN** extracted text and published HTML join the word fragments without a hyphen or whitespace as `пристрастились` and `неискушенными`
