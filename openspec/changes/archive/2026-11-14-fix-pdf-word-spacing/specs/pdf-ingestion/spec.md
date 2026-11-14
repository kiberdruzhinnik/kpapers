# Spec Delta

## ADDED Requirements

### Requirement: Preserve spacing and punctuation between extracted PDF words
The system SHALL reconstruct readable text from extracted PDF words without inserting spaces that split compounds or separate punctuation from adjacent text, while preserving spaces between distinct words.

#### Scenario: Preserve hyphenated compounds
- **WHEN** extracted PDF words form a hyphenated compound such as `поп-психологии`, `по-идиотски`, or `культурно-техническом`
- **THEN** extracted text and published HTML contain the compound without spaces around its hyphen

#### Scenario: Keep punctuation attached to adjacent text
- **WHEN** extracted PDF words include punctuation such as a comma or closing guillemet
- **THEN** extracted text and published HTML contain no space before that punctuation

#### Scenario: Preserve spaces between distinct words
- **WHEN** extracted PDF words are separated as distinct prose words
- **THEN** extracted text and published HTML include the required inter-word spaces

#### Scenario: Preserve italic styling while reconstructing spacing
- **WHEN** a compound, punctuation mark, or closing guillemet spans differently styled extracted words
- **THEN** text spacing and punctuation remain correct and the applicable emphasis markup is preserved
