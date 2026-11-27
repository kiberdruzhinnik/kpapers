# Spec Delta

## MODIFIED Requirements

### Requirement: Preserve lexical hyphens across visual line wraps
The system SHALL retain lexical hyphens when a hyphenated Russian word or compound is split across visual lines or adjacent PDF pages, without adding whitespace around the hyphen. It SHALL remove a line-ending hyphen when Russian lexical data identifies the joined fragments as a word and the hyphen represents discretionary wrapping.

#### Scenario: Russian lexical compound wraps between its components
- **WHEN** a PDF line ends with `проповедники-` and the next line begins with `фундаменталисты`
- **THEN** reconstructed text joins the lines as `проповедники-фундаменталисты`

#### Scenario: Russian hyphenated adverb wraps between its components
- **WHEN** a visual line or page ends with `во-` and the next begins with `вторых`
- **THEN** extracted text and published HTML preserve the hyphen and contain `во-вторых`

#### Scenario: Other lowercase Russian lexical compounds wrap within a page or across a page boundary
- **WHEN** a visual line or page ends with `какой-` before `либо`, or `государств-` before `членов`
- **THEN** extracted text and published HTML preserve the lexical hyphen and join each continuation without whitespace, as `какой-либо` and `государств-членов`

#### Scenario: Capitalized Russian lexical compounds wrap across visual lines
- **WHEN** a visual line ends with `Украина-` before `Россия`, `Косиняк-` before `Камыш`, or `Эр-` before `Рияде`
- **THEN** extracted text and published HTML preserve the lexical hyphen and join the components without whitespace as `Украина-Россия`, `Косиняк-Камыш`, and `Эр-Рияде`

#### Scenario: Russian discretionary word hyphens wrap across a visual line
- **WHEN** a visual line ends with `пристрасти-` before `лись` or `неискушен-` before `ными`, and Russian lexical data identifies the joined word without a hyphen
- **THEN** extracted text and published HTML join the fragments without a hyphen or whitespace as `пристрастились` and `неискушенными`

#### Scenario: Russian lexical data identifies a discretionary hyphen in an unlisted word
- **WHEN** a visual line ends with a Russian word fragment and a hyphen before a continuation, and Russian lexical data identifies the concatenated form as a word while the hyphenated form is not lexical
- **THEN** extracted text and published HTML remove the discretionary hyphen and join the fragments without whitespace

#### Scenario: Russian lexical data identifies a lexical hyphen in an unlisted compound
- **WHEN** a visual line ends with a Russian lexical component and a hyphen before a continuation, and Russian lexical data identifies the hyphenated form as lexical
- **THEN** extracted text and published HTML preserve the hyphen and join the components without whitespace

#### Scenario: Hyphen reconstruction works across a page boundary
- **WHEN** a lexical or discretionary Russian hyphen break occurs between adjacent PDF pages
- **THEN** extracted text and published HTML apply the same lexical classification and joining behavior as for a break within a page
