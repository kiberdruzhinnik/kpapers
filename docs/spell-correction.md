# Local PDF text spell correction

PDF extraction runs a deterministic spell correction pass on the local machine. It uses Spellbook 0.4.2 (MPL-2.0) with bundled Hunspell dictionaries; document text and dictionary lookups are not sent to a service. Spellbook's alpha status and limits on advanced Hunspell features mean the dictionaries are candidate/checking resources, not confidence scores.

## Bundled resources

| Resource | Source | Source version | License |
| --- | --- | --- | --- |
| English `index.aff` and `index.dic` | [wooorm/dictionaries English data](https://github.com/wooorm/dictionaries/tree/main/dictionaries/en) | en_US 2020.12.07 | MIT AND BSD; upstream notice is bundled |
| Russian `index.aff` and `index.dic` | [wooorm/dictionaries Russian data](https://github.com/wooorm/dictionaries/tree/main/dictionaries/ru) | pinned source snapshot | BSD-3-Clause; upstream notice is bundled |

The shipped file hashes (SHA-256) are:

- English `index.aff`: `8ae1f19d4840d957728ad90555d5a8dff6cc5c046279c95ff0c00fc0a0136c7b`
- English `index.dic`: `f0b1a234bd178bdd01875b2a392a9647f888b8fe879f79c52aae62c2759b3647`
- Russian `index.aff`: `38ce7d4af78e211e9bafe4bf7e3d6a2c420591136cb738ec6648f8fdf6524cd7`
- Russian `index.dic`: `f6047416a0204adbecf3a451b874ec8a97ee37e2cbc714466ef04d8dbcc0d6fc`

## Enabled rules and evaluation

Only these exact Russian forms are currently corrected, with their original spelling shown adjacent to the replacement in published text:

| Extracted form | Replacement | Class | Reviewed examples | Observed precision |
| --- | --- | --- | ---: | ---: |
| `посвоему` | `по-своему` | missing hyphen | 1 | 1/1 (100%) |
| `холлинг` | `холдинг` | letter substitution | 1 | 1/1 (100%) |
| `спаслание` | `послание` | letter substitution | 1 | 1/1 (100%) |
| `своеременно` | `своевременно` | letter substitution | 1 | 1/1 (100%) |

Together, the supplied example set is 4/4 (100%). This is a seed set, not a representative evaluation corpus; it does not establish that any general correction class meets the 85% threshold. These are exact-form reviewed exceptions only. The `холлинг` and `своеременно` replacements must also appear in local dictionary suggestions; the other two reviewed forms use exact approved mappings because the dictionary does not suggest the expected form for every extraction-specific error. English corrections and all other dictionary candidates stay unchanged until reviewed examples establish at least 85% precision for their language and correction class. Dictionary suggestion rank is not interpreted as a probability.

Corrections are applied to extracted styled runs before full text, first-page text, HTML, and Markdown are generated. The PDF processing revision is part of cache identity, so this behavior regenerates cached output. RSS and Markdown export use the annotated canonical HTML.
