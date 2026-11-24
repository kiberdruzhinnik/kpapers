use spellbook::Dictionary;
use std::sync::OnceLock;

const EN_AFF: &str = include_str!("../assets/spellcheck/en/index.aff");
const EN_DIC: &str = include_str!("../assets/spellcheck/en/index.dic");
const RU_AFF: &str = include_str!("../assets/spellcheck/ru/index.aff");
const RU_DIC: &str = include_str!("../assets/spellcheck/ru/index.dic");

static EN_DICTIONARY: OnceLock<Option<Dictionary>> = OnceLock::new();
static RU_DICTIONARY: OnceLock<Option<Dictionary>> = OnceLock::new();

#[derive(Clone, Copy)]
enum Language {
    English,
    Russian,
}

impl Language {
    fn annotation(self) -> &'static str {
        match self {
            Self::English => "corrected from",
            Self::Russian => "исправлено",
        }
    }

    fn dictionary(self) -> Option<&'static Dictionary> {
        let (slot, aff, dic) = match self {
            Self::English => (&EN_DICTIONARY, EN_AFF, EN_DIC),
            Self::Russian => (&RU_DICTIONARY, RU_AFF, RU_DIC),
        };
        slot.get_or_init(|| match Dictionary::new(aff, dic) {
            Ok(dictionary) => Some(dictionary),
            Err(error) => {
                tracing::error!(?error, "local spell-check dictionary failed to load");
                None
            }
        })
        .as_ref()
    }

    fn detect(word: &str) -> Option<Self> {
        let mut has_latin = false;
        let mut has_cyrillic = false;
        for character in word.chars().filter(|character| character.is_alphabetic()) {
            match character as u32 {
                0x0041..=0x007A | 0x00C0..=0x024F => has_latin = true,
                0x0400..=0x052F | 0x1C80..=0x1C8F | 0x2DE0..=0x2DFF | 0xA640..=0xA69F => {
                    has_cyrillic = true;
                }
                _ => return None,
            }
        }
        match (has_latin, has_cyrillic) {
            (true, false) => Some(Self::English),
            (false, true) => Some(Self::Russian),
            _ => None,
        }
    }
}

struct ReviewedCorrection {
    from: &'static str,
    to: &'static str,
    language: Language,
    require_suggestion: bool,
}

// These entries are the user-reviewed examples. Other dictionary
// suggestions remain disabled until they have their own reviewed precision set.
const REVIEWED_CORRECTIONS: &[ReviewedCorrection] = &[
    ReviewedCorrection {
        from: "посвоему",
        to: "по-своему",
        language: Language::Russian,
        require_suggestion: false,
    },
    ReviewedCorrection {
        from: "холлинг",
        to: "холдинг",
        language: Language::Russian,
        require_suggestion: true,
    },
    ReviewedCorrection {
        from: "спаслание",
        to: "послание",
        language: Language::Russian,
        require_suggestion: false,
    },
    ReviewedCorrection {
        from: "своеременно",
        to: "своевременно",
        language: Language::Russian,
        require_suggestion: true,
    },
];

pub(crate) fn correct_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut word_start = None;
    for (index, character) in text.char_indices() {
        if character.is_alphanumeric() {
            word_start.get_or_insert(index);
        } else if let Some(start) = word_start.take() {
            output.push_str(&correct_word(&text[start..index]));
            output.push(character);
        } else {
            output.push(character);
        }
    }
    if let Some(start) = word_start {
        output.push_str(&correct_word(&text[start..]));
    }
    output
}

fn correct_word(word: &str) -> String {
    let Some(language) = Language::detect(word) else {
        return word.to_owned();
    };
    let Some(dictionary) = language.dictionary() else {
        return word.to_owned();
    };
    let reviewed = REVIEWED_CORRECTIONS.iter().find(|correction| {
        correction.language.same(language) && correction.from.to_lowercase() == word.to_lowercase()
    });
    if let Some(correction) = reviewed {
        if !dictionary.check(correction.to) {
            return word.to_owned();
        }
        if correction.require_suggestion {
            let mut suggestions = Vec::new();
            dictionary.suggest(word, &mut suggestions);
            if !suggestions
                .iter()
                .any(|candidate| candidate.to_lowercase() == correction.to.to_lowercase())
            {
                return word.to_owned();
            }
        }

        return format!(
            "{} [{}: {}]",
            preserve_case(word, correction.to),
            language.annotation(),
            word
        );
    }
    if dictionary.check(word) {
        return word.to_owned();
    }

    {
        // Generate suggestions locally to perform a real spell check, but do not
        // auto-apply an unreviewed suggestion based on its rank alone.
        let mut suggestions = Vec::new();
        dictionary.suggest(word, &mut suggestions);
    }
    word.to_owned()
}

impl Language {
    fn same(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::English, Self::English) | (Self::Russian, Self::Russian)
        )
    }
}

fn preserve_case(original: &str, corrected: &str) -> String {
    if original.chars().all(|character| !character.is_lowercase()) {
        corrected.to_uppercase()
    } else if original.chars().next().is_some_and(char::is_uppercase)
        && original
            .chars()
            .skip(1)
            .all(|character| !character.is_uppercase())
    {
        let mut characters = corrected.chars();
        let Some(first) = characters.next() else {
            return corrected.to_owned();
        };
        first.to_uppercase().chain(characters).collect()
    } else {
        corrected.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::correct_text;

    #[test]
    fn reviewed_pdf_corrections_meet_the_precision_floor() {
        let cases = include_str!("../tests/fixtures/spell-correction/evaluation.tsv");
        let mut reviewed = 0usize;
        let mut correct = 0usize;
        for row in cases.lines().filter(|line| !line.starts_with('#')) {
            if row.starts_with("source\t") || row.is_empty() {
                continue;
            }
            let fields = row.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 5, "malformed evaluation row: {row}");
            let [_, extracted, expected, language, class] = fields.as_slice() else {
                unreachable!("row length checked above");
            };
            assert_eq!(*language, "ru");
            let expected_output = format!("{expected} [исправлено: {extracted}]");
            let actual = correct_text(extracted);
            assert_eq!(actual, expected_output, "correction class {class}");
            reviewed += 1;
            correct += usize::from(actual == expected_output);
        }
        assert_eq!(reviewed, 4, "keep the reviewed corpus explicit");
        assert!(correct * 100 >= reviewed * 85);
    }

    #[test]
    fn preserves_valid_words_and_unreviewed_candidates() {
        assert_eq!(
            correct_text("по-своему мини-холдинг success teh").as_str(),
            "по-своему мини-холдинг success teh"
        );
        assert_eq!(
            correct_text("неизвестноедлясловаря").as_str(),
            "неизвестноедлясловаря"
        );
    }

    #[test]
    fn preserves_case_and_surrounding_punctuation() {
        assert_eq!(
            correct_text("ПОСВОЕМУ, Холлинг!").as_str(),
            "ПО-СВОЕМУ [исправлено: ПОСВОЕМУ], Холдинг [исправлено: Холлинг]!"
        );
    }
}
