# Tasks

## 1. Parenthetical spacing

- [x] 1.1 Update PDF word reconstruction to preserve conventional prose spacing before opening parentheses and guillemets, after closing quotes, and across Latin/Cyrillic and letter/digit boundaries; verify with the reported examples including `элите (если`, `Ирана (КСИР)`, `сутки (б/с)`, `по «несанкционированным»`, `„Грязная бомба“ использует`, `из Rаpporteur`, `такую как Wildberries`, `войны 28`, and `с 8,372`.
- [x] 1.2 Add regression coverage for comma and closing-parenthesis adjacency; verify the focused PDF processing tests pass.
- [x] 1.3 Preserve lexical hyphens in known compounds split across lines, including `проповедники-фундаменталисты`; verify reconstructed text and HTML retain the hyphen.

## 2. CrimsonAnalytics title processing

- [x] 2.1 Recognize and remove the CrimsonAnalytics branding and publication metadata prefix; verify first-page text starts with the headline and excludes the header.
- [x] 2.2 Merge the supplied long wrapped headline (`[Новостной деск] Последствия ударов по Крымскому мосту, контуры «плана Кушнера» ... «горячей» войны с Ираном`) and render it once as one `<h1>` without merging the lead or body; verify the headline, lead, and following paragraph remain distinct in HTML.
- [x] 2.3 Add regression coverage for the supplied title and update the extraction pipeline revision; verify the focused PDF processing tests pass and the cache fingerprint reflects the new revision.
