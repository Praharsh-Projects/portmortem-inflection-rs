#![forbid(unsafe_code)]
//! English word inflection and identifier transformation utilities.
//!
//! The behavior and rule ordering mirror the public functions in Python's
//! `inflection` package. All functions are deterministic.

use regex::{Captures, Regex};
use std::borrow::Cow;
use std::fmt;
use std::sync::OnceLock;
use unicode_general_category::{get_general_category, GeneralCategory};
use unicode_normalization::UnicodeNormalization;

const UNCOUNTABLES: [&str; 9] = [
    "equipment",
    "fish",
    "information",
    "jeans",
    "money",
    "rice",
    "series",
    "sheep",
    "species",
];

// `_irregular` prepends rules at Python module import time. These arrays store
// the resulting order, followed by the reference package's base rules.
const PLURAL_RULES: [(&str, &str); 41] = [
    (r"(?i)(z)ombies$", "${1}ombies"),
    (r"(?i)(z)ombie$", "${1}ombies"),
    (r"k[iI][nN][eE]$", "kine"),
    (r"K[iI][nN][eE]$", "Kine"),
    (r"c[oO][wW]$", "kine"),
    (r"C[oO][wW]$", "Kine"),
    (r"(?i)(m)oves$", "${1}oves"),
    (r"(?i)(m)ove$", "${1}oves"),
    (r"(?i)(s)exes$", "${1}exes"),
    (r"(?i)(s)ex$", "${1}exes"),
    (r"(?i)(c)hildren$", "${1}hildren"),
    (r"(?i)(c)hild$", "${1}hildren"),
    (r"(?i)(h)umans$", "${1}umans"),
    (r"(?i)(h)uman$", "${1}umans"),
    (r"(?i)(m)en$", "${1}en"),
    (r"(?i)(m)an$", "${1}en"),
    (r"(?i)(p)eople$", "${1}eople"),
    (r"(?i)(p)erson$", "${1}eople"),
    (r"(?i)(quiz)$", "${1}zes"),
    (r"(?i)^(oxen)$", "${1}"),
    (r"(?i)^(ox)$", "${1}en"),
    (r"(?i)(m|l)ice$", "${1}ice"),
    (r"(?i)(m|l)ouse$", "${1}ice"),
    (r"(?i)(passer)s?by$", "${1}sby"),
    (r"(?i)(matr|vert|ind)(?:ix|ex)$", "${1}ices"),
    (r"(?i)(x|ch|ss|sh)$", "${1}es"),
    (r"(?i)([^aeiouy]|qu)y$", "${1}ies"),
    (r"(?i)(hive)$", "${1}s"),
    (r"(?i)([lr])f$", "${1}ves"),
    (r"(?i)([^f])fe$", "${1}ves"),
    (r"(?i)sis$", "ses"),
    (r"(?i)([ti])a$", "${1}a"),
    (r"(?i)([ti])um$", "${1}a"),
    (r"(?i)(buffal|potat|tomat)o$", "${1}oes"),
    (r"(?i)(bu)s$", "${1}ses"),
    (r"(?i)(alias|status)$", "${1}es"),
    (r"(?i)(octop|vir)i$", "${1}i"),
    (r"(?i)(octop|vir)us$", "${1}i"),
    (r"(?i)^(ax|test)is$", "${1}es"),
    (r"(?i)s$", "s"),
    (r"$", "s"),
];

const SINGULAR_RULES: [(&str, &str); 42] = [
    (r"(?i)(z)ombies$", "${1}ombie"),
    (r"k[iI][nN][eE]$", "cow"),
    (r"K[iI][nN][eE]$", "Cow"),
    (r"(?i)(m)oves$", "${1}ove"),
    (r"(?i)(s)exes$", "${1}ex"),
    (r"(?i)(c)hildren$", "${1}hild"),
    (r"(?i)(h)umans$", "${1}uman"),
    (r"(?i)(m)en$", "${1}an"),
    (r"(?i)(p)eople$", "${1}erson"),
    (r"(?i)(database)s$", "${1}"),
    (r"(?i)(quiz)zes$", "${1}"),
    (r"(?i)(matr)ices$", "${1}ix"),
    (r"(?i)(vert|ind)ices$", "${1}ex"),
    (r"(?i)(passer)sby$", "${1}by"),
    (r"(?i)^(ox)en", "${1}"),
    (r"(?i)(alias|status)(es)?$", "${1}"),
    (r"(?i)(octop|vir)(us|i)$", "${1}us"),
    (r"(?i)^(a)x[ie]s$", "${1}xis"),
    (r"(?i)(cris|test)(is|es)$", "${1}is"),
    (r"(?i)(shoe)s$", "${1}"),
    (r"(?i)(o)es$", "${1}"),
    (r"(?i)(bus)(es)?$", "${1}"),
    (r"(?i)(m|l)ice$", "${1}ouse"),
    (r"(?i)(x|ch|ss|sh)es$", "${1}"),
    (r"(?i)(m)ovies$", "${1}ovie"),
    (r"(?i)(s)eries$", "${1}eries"),
    (r"(?i)([^aeiouy]|qu)ies$", "${1}y"),
    (r"(?i)([lr])ves$", "${1}f"),
    (r"(?i)(tive)s$", "${1}"),
    (r"(?i)(hive)s$", "${1}"),
    (r"(?i)([^f])ves$", "${1}fe"),
    (r"(?i)(t)he(sis|ses)$", "${1}hesis"),
    (r"(?i)(s)ynop(sis|ses)$", "${1}ynopsis"),
    (r"(?i)(p)rogno(sis|ses)$", "${1}rognosis"),
    (r"(?i)(p)arenthe(sis|ses)$", "${1}arenthesis"),
    (r"(?i)(d)iagno(sis|ses)$", "${1}iagnosis"),
    (r"(?i)(b)a(sis|ses)$", "${1}asis"),
    (r"(?i)(a)naly(sis|ses)$", "${1}nalysis"),
    (r"(?i)([ti])a$", "${1}um"),
    (r"(?i)(n)ews$", "${1}ews"),
    (r"(?i)(ss)$", "${1}"),
    (r"(?i)s$", ""),
];

struct Rule {
    regex: Regex,
    replacement: &'static str,
}

/// An invalid Python-compatible replacement template supplied as a separator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterizeError {
    code: &'static str,
    message: String,
}

impl ParameterizeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            code: "oracle_error",
            message: message.into(),
        }
    }

    fn reference_index(message: impl Into<String>) -> Self {
        Self {
            code: "reference_index_error",
            message: message.into(),
        }
    }

    /// Returns the JSONL oracle-compatible error classification.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for ParameterizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ParameterizeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReplacementPart {
    Literal(String),
    WholeMatch,
}

fn compile_rules(rules: &'static [(&'static str, &'static str)]) -> Vec<Rule> {
    rules
        .iter()
        .map(|(source_pattern, replacement)| {
            let pattern = expand_python_ignore_case_i(source_pattern);
            Rule {
                regex: Regex::new(&pattern).expect("reference inflection rule must compile"),
                replacement,
            }
        })
        .collect()
}

/// Python documents four extra Unicode matches for ASCII classes under
/// `re.IGNORECASE`: İ, ı, ſ, and K. Rust regex already handles the simple
/// single-code-point folds (including ı, ſ, and K), but İ lowercases to two
/// code points and therefore needs an explicit alternative wherever a Python
/// rule contains ASCII `i`. Keep this at rule-compilation time so captures and
/// replacements continue to preserve the original input spelling.
fn expand_python_ignore_case_i(pattern: &str) -> Cow<'_, str> {
    let Some(body) = pattern.strip_prefix("(?i)") else {
        return Cow::Borrowed(pattern);
    };

    let mut expanded = String::with_capacity(pattern.len() + 8);
    expanded.push_str("(?i)");
    let mut in_class = false;
    let mut class_contains_i = false;
    let mut escaped = false;

    for character in body.chars() {
        if escaped {
            expanded.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            expanded.push(character);
            escaped = true;
            continue;
        }
        if character == '[' && !in_class {
            in_class = true;
            class_contains_i = false;
            expanded.push(character);
            continue;
        }
        if character == ']' && in_class {
            if class_contains_i {
                expanded.push_str("İı");
            }
            in_class = false;
            expanded.push(character);
            continue;
        }
        if matches!(character, 'i' | 'I') {
            if in_class {
                class_contains_i = true;
                expanded.push(character);
            } else {
                expanded.push_str("[iİı]");
            }
            continue;
        }
        expanded.push(character);
    }

    Cow::Owned(expanded)
}

fn plural_rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| compile_rules(&PLURAL_RULES))
}

fn singular_rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| compile_rules(&SINGULAR_RULES))
}

fn apply_first_rule<'a>(value: &'a str, rules: &[Rule]) -> Cow<'a, str> {
    for rule in rules {
        // Python's `$` also matches immediately before one trailing newline.
        // Its zero-width fallback therefore inserts at both positions.
        if rule.regex.as_str() == "$" {
            return if let Some(prefix) = value.strip_suffix('\n') {
                Cow::Owned(format!("{prefix}s\ns"))
            } else {
                Cow::Owned(format!("{value}s"))
            };
        }
        if rule.regex.is_match(value) {
            return Cow::Owned(rule.regex.replace_all(value, rule.replacement).into_owned());
        }
        if let Some(prefix) = value.strip_suffix('\n') {
            if rule.regex.is_match(prefix) {
                let mut replaced = rule
                    .regex
                    .replace_all(prefix, rule.replacement)
                    .into_owned();
                replaced.push('\n');
                return Cow::Owned(replaced);
            }
        }
    }
    Cow::Borrowed(value)
}

/// Converts an underscored identifier to upper or lower camel case.
#[must_use]
pub fn camelize(value: &str, uppercase_first_letter: bool) -> String {
    static CAMEL_BOUNDARY: OnceLock<Regex> = OnceLock::new();
    let regex = CAMEL_BOUNDARY
        .get_or_init(|| Regex::new(r"(?:^|_)(.)").expect("camelize regex must compile"));
    let upper = regex
        .replace_all(value, |captures: &Captures<'_>| {
            python_uppercase(&captures[1])
        })
        .into_owned();

    if uppercase_first_letter || value.is_empty() {
        return upper;
    }

    let lowered_first = value.chars().next().map_or_else(String::new, |character| {
        python_lowercase(&character.to_string())
    });
    let upper_without_first = upper
        .char_indices()
        .nth(1)
        .map_or("", |(index, _)| &upper[index..]);
    lowered_first + upper_without_first
}

/// Replaces underscores with dashes.
#[must_use]
pub fn dasherize(value: &str) -> String {
    value.replace('_', "-")
}

/// Converts an underscored identifier into a human-readable label.
#[must_use]
pub fn humanize(value: &str) -> String {
    let without_id = if let Some(prefix) = value.strip_suffix("_id") {
        prefix.to_owned()
    } else if let Some(prefix) = value.strip_suffix("_id\n") {
        format!("{prefix}\n")
    } else {
        value.to_owned()
    };
    let with_spaces = without_id.replace('_', " ");
    // Python's Unicode `re.IGNORECASE` makes `[a-z]` match the ASCII letters
    // plus İ, ı, ſ, and K. Rust regex intentionally avoids multi-character
    // case folds such as İ -> i + combining dot, so reproduce that documented
    // Python set explicitly and then apply Unicode lowercase mapping.
    let mut lowered = String::with_capacity(with_spaces.len());
    for character in with_spaces.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, 'İ' | 'ı' | 'ſ' | 'K') {
            push_case_mapping(
                &mut lowered,
                character,
                unicode_case_mapping::to_lowercase(character),
            );
        } else {
            lowered.push(character);
        }
    }
    uppercase_initial_word_character(&lowered)
}

/// Returns the English ordinal suffix for an integer.
#[must_use]
pub const fn ordinal(number: i64) -> &'static str {
    let absolute = number.unsigned_abs();
    match absolute % 100 {
        11..=13 => "th",
        _ => match absolute % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    }
}

/// Formats an integer with its English ordinal suffix.
#[must_use]
pub fn ordinalize(number: i64) -> String {
    format!("{number}{}", ordinal(number))
}

/// Converts arbitrary text into a lower-case URL component.
pub fn parameterize(value: &str, separator: &str) -> Result<String, ParameterizeError> {
    let replacement = parse_replacement_template(separator)?;
    let ascii = transliterate(value);
    let mut parameterized = String::with_capacity(ascii.len());
    let bytes = ascii.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let allowed = bytes[index].is_ascii_alphanumeric() || matches!(bytes[index], b'-' | b'_');
        if allowed {
            parameterized.push(char::from(bytes[index]));
            index += 1;
            continue;
        }

        let start = index;
        while index < bytes.len()
            && !bytes[index].is_ascii_alphanumeric()
            && !matches!(bytes[index], b'-' | b'_')
        {
            index += 1;
        }
        expand_replacement(&mut parameterized, &replacement, &ascii[start..index]);
    }

    if !separator.is_empty() {
        // Keep the reference's exact regex construction: `{2,}` applies to
        // the final regex atom rather than to a grouped multi-character
        // separator. This is externally observable for separators such as
        // `__sep__` next to an existing underscore.
        parameterized = squeeze_separator(&parameterized, separator);
        parameterized = trim_separator_ignore_case(&parameterized, separator);
    }

    Ok(parameterized.to_lowercase())
}

fn push_literal_part(parts: &mut Vec<ReplacementPart>, literal: &mut String) {
    if !literal.is_empty() {
        parts.push(ReplacementPart::Literal(std::mem::take(literal)));
    }
}

fn parse_replacement_template(
    replacement: &str,
) -> Result<Vec<ReplacementPart>, ParameterizeError> {
    let characters = replacement.chars().collect::<Vec<_>>();
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];
        if character != '\\' {
            literal.push(character);
            index += 1;
            continue;
        }

        let escape_start = index;
        index += 1;
        let Some(escaped) = characters.get(index).copied() else {
            return Err(ParameterizeError::new(format!(
                "bad escape (end of pattern) at position {escape_start}"
            )));
        };
        index += 1;

        // CPython's replacement tokenizer maintains one token of look-ahead,
        // so an immediately following terminal backslash is reported before
        // the current escape is interpreted.
        if index + 1 == characters.len() && characters[index] == '\\' {
            return Err(ParameterizeError::new(format!(
                "bad escape (end of pattern) at position {index}"
            )));
        }

        match escaped {
            'g' => {
                if characters.get(index) != Some(&'<') {
                    return Err(ParameterizeError::new(format!(
                        "missing < at position {index}"
                    )));
                }
                index += 1;
                let name_start = index;
                while index < characters.len() && characters[index] != '>' {
                    if index + 1 == characters.len() && characters[index] == '\\' {
                        return Err(ParameterizeError::new(format!(
                            "bad escape (end of pattern) at position {index}"
                        )));
                    }
                    index += 1;
                }
                if index == characters.len() {
                    let label = if name_start == index {
                        "missing group name"
                    } else {
                        "missing >, unterminated name"
                    };
                    return Err(ParameterizeError::new(format!(
                        "{label} at position {index}"
                    )));
                }
                let name = characters[name_start..index].iter().collect::<String>();
                index += 1;
                if name == "0" {
                    push_literal_part(&mut parts, &mut literal);
                    parts.push(ReplacementPart::WholeMatch);
                } else if !name.is_empty()
                    && name.chars().all(|character| character.is_ascii_digit())
                {
                    return Err(ParameterizeError::new(format!(
                        "invalid group reference {} at position {name_start}",
                        name.parse::<u128>().unwrap_or(u128::MAX)
                    )));
                } else if name.is_empty() {
                    return Err(ParameterizeError::new(format!(
                        "missing group name at position {name_start}"
                    )));
                } else if python_identifier(&name) {
                    return Err(ParameterizeError::reference_index(format!(
                        "unknown group name '{name}'"
                    )));
                } else {
                    return Err(ParameterizeError::new(format!(
                        "bad character in group name '{name}' at position {name_start}"
                    )));
                }
            }
            '0' => {
                let mut digits = String::from('0');
                for _ in 0..2 {
                    if characters
                        .get(index)
                        .is_some_and(|character| matches!(character, '0'..='7'))
                    {
                        digits.push(characters[index]);
                        index += 1;
                    } else {
                        break;
                    }
                }
                let value = u32::from_str_radix(&digits, 8)
                    .expect("a validated octal replacement must parse");
                literal.push(char::from_u32(value).expect("an octal byte must be a scalar"));
            }
            '1'..='9' => {
                let mut digits = String::from(escaped);
                if characters
                    .get(index)
                    .is_some_and(|character| character.is_ascii_digit())
                {
                    digits.push(characters[index]);
                    index += 1;
                }
                let is_octal = digits
                    .chars()
                    .all(|character| matches!(character, '0'..='7'))
                    && characters
                        .get(index)
                        .is_some_and(|character| matches!(character, '0'..='7'));
                if is_octal {
                    digits.push(characters[index]);
                    index += 1;
                    let value = u32::from_str_radix(&digits, 8)
                        .expect("a validated octal replacement must parse");
                    if value > 0o377 {
                        return Err(ParameterizeError::new(format!(
                            "octal escape value \\{digits} outside of range 0-0o377 at position {escape_start}"
                        )));
                    }
                    literal.push(
                        char::from_u32(value).expect("an in-range octal byte must be a scalar"),
                    );
                } else {
                    return Err(ParameterizeError::new(format!(
                        "invalid group reference {} at position {}",
                        digits.parse::<u128>().unwrap_or(u128::MAX),
                        escape_start + 1
                    )));
                }
            }
            'a' => literal.push('\u{0007}'),
            'b' => literal.push('\u{0008}'),
            'f' => literal.push('\u{000C}'),
            'n' => literal.push('\n'),
            'r' => literal.push('\r'),
            't' => literal.push('\t'),
            'v' => literal.push('\u{000B}'),
            '\\' => literal.push('\\'),
            character if character.is_ascii_alphabetic() => {
                return Err(ParameterizeError::new(format!(
                    "bad escape \\{character} at position {escape_start}"
                )));
            }
            character => {
                literal.push('\\');
                literal.push(character);
            }
        }
    }

    push_literal_part(&mut parts, &mut literal);
    Ok(parts)
}

fn expand_replacement(output: &mut String, parts: &[ReplacementPart], matched: &str) {
    for part in parts {
        match part {
            ReplacementPart::Literal(value) => output.push_str(value),
            ReplacementPart::WholeMatch => output.push_str(matched),
        }
    }
}

fn squeeze_separator(value: &str, separator: &str) -> String {
    let last = separator
        .chars()
        .next_back()
        .expect("a nonempty separator must have a final character");
    let mut minimum = String::with_capacity(separator.len() + last.len_utf8());
    minimum.push_str(separator);
    minimum.push(last);
    let last_text = last.to_string();
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0;

    while let Some(relative) = value[cursor..].find(&minimum) {
        let start = cursor + relative;
        output.push_str(&value[cursor..start]);
        output.push_str(separator);
        cursor = start + minimum.len();
        while value[cursor..].starts_with(&last_text) {
            cursor += last_text.len();
        }
    }
    output.push_str(&value[cursor..]);
    output
}

fn trim_separator_ignore_case(value: &str, separator: &str) -> String {
    let without_start = strip_prefix_ignore_case(value, separator).unwrap_or(value);
    strip_suffix_ignore_case(without_start, separator)
        .unwrap_or(without_start)
        .to_owned()
}

fn strip_prefix_ignore_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let mut value_characters = value.char_indices();
    let mut consumed = 0;
    for expected in prefix.chars() {
        let (index, actual) = value_characters.next()?;
        if !python_ignore_case_equal(actual, expected) {
            return None;
        }
        consumed = index + actual.len_utf8();
    }
    Some(&value[consumed..])
}

fn strip_suffix_ignore_case<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    let mut value_characters = value.char_indices().rev();
    let mut start = value.len();
    for expected in suffix.chars().rev() {
        let (index, actual) = value_characters.next()?;
        if !python_ignore_case_equal(actual, expected) {
            return None;
        }
        start = index;
    }
    Some(&value[..start])
}

/// Returns the plural form selected by the reference rule set.
#[must_use]
pub fn pluralize(value: &str) -> String {
    if value.is_empty()
        || UNCOUNTABLES
            .iter()
            .any(|uncountable| value.eq_ignore_ascii_case(uncountable))
    {
        return value.to_owned();
    }
    apply_first_rule(value, plural_rules()).into_owned()
}

/// Returns the singular form selected by the reference rule set.
#[must_use]
pub fn singularize(value: &str) -> String {
    if UNCOUNTABLES.iter().any(|uncountable| {
        ends_with_ignore_case(value, uncountable).is_some_and(|start| {
            start == 0
                || !value[..start]
                    .chars()
                    .next_back()
                    .is_some_and(python_word_character)
        })
    }) {
        return value.to_owned();
    }
    apply_first_rule(value, singular_rules()).into_owned()
}

/// Converts a model-style identifier to its plural table name.
#[must_use]
pub fn tableize(value: &str) -> String {
    pluralize(&underscore(value))
}

/// Converts an identifier or phrase into a human-readable title.
#[must_use]
pub fn titleize(value: &str) -> String {
    let humanized = humanize(&underscore(value));
    let titled = python_like_title(&humanized);
    capitalize_python_word_boundaries(&titled)
}

/// Applies NFKD normalization and drops all non-ASCII code points.
#[must_use]
pub fn transliterate(value: &str) -> String {
    value.nfkd().filter(char::is_ascii).collect()
}

/// Converts CamelCase, kebab-case, or mixed identifiers to snake case.
#[must_use]
pub fn underscore(value: &str) -> String {
    static ACRONYM_BOUNDARY: OnceLock<Regex> = OnceLock::new();
    static LOWER_BOUNDARY: OnceLock<Regex> = OnceLock::new();
    let acronym = ACRONYM_BOUNDARY.get_or_init(|| {
        Regex::new(r"([A-Z]+)([A-Z][a-z])").expect("underscore acronym regex must compile")
    });
    let lower = LOWER_BOUNDARY.get_or_init(|| {
        Regex::new(r"([a-z\d])([A-Z])").expect("underscore case regex must compile")
    });

    let first = acronym.replace_all(value, "${1}_${2}");
    let second = lower.replace_all(&first, "${1}_${2}");
    second.replace('-', "_").to_lowercase()
}

fn push_case_mapping<const N: usize>(output: &mut String, original: char, mapping: [u32; N]) {
    let mut mapped = false;
    for point in mapping.into_iter().filter(|point| *point != 0) {
        output.push(char::from_u32(point).expect("Unicode case data must contain scalar values"));
        mapped = true;
    }
    if !mapped {
        output.push(original);
    }
}

fn python_lowercase(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        push_case_mapping(
            &mut output,
            character,
            unicode_case_mapping::to_lowercase(character),
        );
    }
    output
}

fn python_uppercase(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        push_case_mapping(
            &mut output,
            character,
            unicode_case_mapping::to_uppercase(character),
        );
    }
    output
}

fn python_word_character(character: char) -> bool {
    character == '_'
        || matches!(
            get_general_category(character),
            GeneralCategory::UppercaseLetter
                | GeneralCategory::LowercaseLetter
                | GeneralCategory::TitlecaseLetter
                | GeneralCategory::ModifierLetter
                | GeneralCategory::OtherLetter
                | GeneralCategory::DecimalNumber
                | GeneralCategory::LetterNumber
                | GeneralCategory::OtherNumber
        )
}

fn python_cased(character: char) -> bool {
    static CASED: OnceLock<Regex> = OnceLock::new();
    let regex =
        CASED.get_or_init(|| Regex::new(r"\A\p{Cased}\z").expect("Cased property must compile"));
    let mut encoded = [0; 4];
    regex.is_match(character.encode_utf8(&mut encoded))
}

fn python_identifier(value: &str) -> bool {
    static IDENTIFIER: OnceLock<Regex> = OnceLock::new();
    IDENTIFIER
        .get_or_init(|| {
            Regex::new(r"\A(?:_|\p{XID_Start})(?:_|\p{XID_Continue})*\z")
                .expect("identifier property must compile")
        })
        .is_match(value)
}

fn simple_case_fold(character: char) -> char {
    unicode_case_mapping::case_folded(character)
        .and_then(|point| char::from_u32(point.get()))
        .unwrap_or(character)
}

fn python_ignore_case_equal(left: char, right: char) -> bool {
    if left == right || simple_case_fold(left) == simple_case_fold(right) {
        return true;
    }

    const PYTHON_EXTRA_GROUPS: [&str; 3] = ["iIİı", "sSſ", "kKK"];
    PYTHON_EXTRA_GROUPS
        .iter()
        .any(|group| group.contains(left) && group.contains(right))
}

fn ends_with_ignore_case(value: &str, suffix: &str) -> Option<usize> {
    let mut value_characters = value.char_indices().rev();
    let mut start = value.len();
    for expected in suffix.chars().rev() {
        let (index, actual) = value_characters.next()?;
        if !python_ignore_case_equal(actual, expected) {
            return None;
        }
        start = index;
    }
    Some(start)
}

fn uppercase_initial_word_character(value: &str) -> String {
    let Some(first) = value.chars().next() else {
        return String::new();
    };
    if !python_word_character(first) {
        return value.to_owned();
    }

    let mut output = String::with_capacity(value.len());
    push_case_mapping(
        &mut output,
        first,
        unicode_case_mapping::to_uppercase(first),
    );
    output.push_str(&value[first.len_utf8()..]);
    output
}

fn python_like_title(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut characters = value.char_indices().peekable();
    while let Some((start, character)) = characters.next() {
        if !python_cased(character) {
            output.push(character);
            continue;
        }

        let mut end = start + character.len_utf8();
        while characters
            .peek()
            .is_some_and(|(_, next)| python_cased(*next))
        {
            let (index, next) = characters
                .next()
                .expect("a peeked cased character must be present");
            end = index + next.len_utf8();
        }

        let run = &value[start..end];
        push_case_mapping(
            &mut output,
            character,
            unicode_case_mapping::to_titlecase(character),
        );
        let lowered = run.to_lowercase();
        let first_lowered = character.to_string().to_lowercase();
        output.push_str(
            lowered
                .strip_prefix(&first_lowered)
                .expect("lowercasing a run must start with its first mapping"),
        );
    }
    output
}

fn capitalize_python_word_boundaries(value: &str) -> String {
    let characters = value.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(value.len());
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        let previous_is_word = index > 0 && python_word_character(characters[index - 1]);

        if python_word_character(character) && !previous_is_word {
            push_case_mapping(
                &mut output,
                character,
                unicode_case_mapping::to_titlecase(character),
            );
            index += 1;
        } else if character == '\''
            && previous_is_word
            && characters
                .get(index + 1)
                .copied()
                .is_some_and(python_word_character)
        {
            output.push(character);
            let next = characters[index + 1];
            push_case_mapping(&mut output, next, unicode_case_mapping::to_lowercase(next));
            index += 2;
        } else {
            output.push(character);
            index += 1;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_examples_match_reference() {
        assert_eq!(camelize("device_type", true), "DeviceType");
        assert_eq!(camelize("device_type", false), "deviceType");
        assert_eq!(dasherize("street_address"), "street-address");
        assert_eq!(humanize("employee_id"), "Employee");
        assert_eq!(ordinal(-1021), "st");
        assert_eq!(ordinalize(-11), "-11th");
        assert_eq!(
            parameterize("Donald E. Knuth", "-").expect("valid separator"),
            "donald-e-knuth"
        );
        assert_eq!(pluralize("CamelOctopus"), "CamelOctopi");
        assert_eq!(singularize("CamelOctopi"), "CamelOctopus");
        assert_eq!(tableize("RawScaledScorer"), "raw_scaled_scorers");
        assert_eq!(titleize("david's code"), "David's Code");
        assert_eq!(transliterate("älämölö"), "alamolo");
        assert_eq!(underscore("HTMLTidyGenerator"), "html_tidy_generator");
    }

    #[test]
    fn ordinal_handles_entire_i64_domain() {
        assert_eq!(ordinal(i64::MIN), "th");
        assert_eq!(ordinal(i64::MAX), "th");
    }

    #[test]
    fn rules_match_python_ignore_case_expansions_for_capital_i_with_dot() {
        assert_eq!(pluralize("dİY"), "dİYs");
        assert_eq!(pluralize("İum"), "İa");
        assert_eq!(pluralize("quİz"), "quİzzes");
        assert_eq!(pluralize("chİld"), "children");
        assert_eq!(singularize("movİes"), "movie");
        assert_eq!(singularize("thesİs"), "thesis");
    }

    #[test]
    fn multi_character_separator_preserves_reference_regex_semantics() {
        // The pinned Python regex applies `{2,}` to the separator's final
        // atom, so an adjacent source underscore is consumed. This surprising
        // behavior is retained and documented as an upstream finding.
        assert_eq!(
            parameterize("a _b", "__sep__").expect("valid separator"),
            "a__sep__b"
        );
        assert_eq!(
            parameterize("x/_y", "__sep__").expect("valid separator"),
            "x__sep__y"
        );
    }

    #[test]
    fn unicode_tables_and_word_boundaries_match_python_314() {
        assert_eq!(std::char::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(unicode_case_mapping::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(unicode_general_category::UNICODE_VERSION, (16, 0, 0));
        assert_eq!(unicode_normalization::UNICODE_VERSION, (16, 0, 0));

        assert_eq!(humanize("\u{0345}"), "\u{0345}");
        assert_eq!(humanize("ⓐ"), "ⓐ");
        assert_eq!(titleize("Ⓐx"), "ⒶX");
        assert_eq!(titleize("a\u{0345}b"), "A\u{0345}B");
        assert_eq!(titleize("a'Ⓐ"), "A'Ⓐ");
        assert_eq!(titleize("\u{02B0}a"), "\u{02B0}a");
        assert_eq!(titleize("a \u{019B}"), "A \u{A7DC}");
        assert_eq!(titleize("a \u{0264}"), "A \u{A7CB}");
        assert_eq!(titleize("ΟΣ"), "Ος");
        assert_eq!(underscore("ΟΣ"), "ος");
        assert_eq!(underscore("ΟΣΑ"), "οσα");

        assert_eq!(singularize("\u{0345}species"), "\u{0345}species");
        assert_eq!(singularize("ⓐspecies"), "ⓐspecies");
        assert_eq!(singularize("\u{200C}species"), "\u{200C}species");
        assert_eq!(singularize("serİes"), "serİes");
        assert_eq!(singularize("serıes"), "serıes");
        assert_eq!(singularize("specİes"), "specİes");
    }

    #[test]
    fn parameterize_handles_python_replacement_templates_without_panicking() {
        assert_eq!(parameterize("x y", "Ö").expect("valid separator"), "xöy");
        assert_eq!(
            parameterize("x y", "İ").expect("valid separator"),
            "xi\u{0307}y"
        );
        assert_eq!(
            parameterize("a b", r"\n").expect("valid replacement escape"),
            "a\nb"
        );
        assert_eq!(
            parameterize("a b", r"\g<0>").expect("valid whole-match reference"),
            "a b"
        );
        assert_eq!(
            parameterize("abc", &"a".repeat(2_000_000)).expect("large literal separator"),
            "abc"
        );
        assert_eq!(
            parameterize("abc", r"\1")
                .expect_err("group one does not exist")
                .to_string(),
            "invalid group reference 1 at position 1"
        );
    }
}
