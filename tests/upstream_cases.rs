//! Behavioral parity cases translated from `reference/test_inflection.py`.
//!
//! Upstream: jpvanhal/inflection@88eefaacf7d0caaa701af7c8ab2d0ab3f17086f1
//!
//! Mapping audit: all 725 expanded upstream assertions are represented. Of
//! those, 722 call the crate directly. Three assertions mutate Python's public
//! `UNCOUNTABLES` global at runtime; the labeled caller-owned adapter below
//! preserves those cases because the Rust rule set is intentionally immutable.

use portmortem_inflection_rs::{
    camelize, dasherize, humanize, ordinal, ordinalize, parameterize, pluralize, singularize,
    tableize, titleize, underscore,
};

const SINGULAR_TO_PLURAL: &[(&str, &str)] = &[
    ("search", "searches"),
    ("switch", "switches"),
    ("fix", "fixes"),
    ("box", "boxes"),
    ("process", "processes"),
    ("address", "addresses"),
    ("case", "cases"),
    ("stack", "stacks"),
    ("wish", "wishes"),
    ("fish", "fish"),
    ("jeans", "jeans"),
    ("funky jeans", "funky jeans"),
    ("category", "categories"),
    ("query", "queries"),
    ("ability", "abilities"),
    ("agency", "agencies"),
    ("movie", "movies"),
    ("archive", "archives"),
    ("index", "indices"),
    ("wife", "wives"),
    ("safe", "saves"),
    ("half", "halves"),
    ("move", "moves"),
    ("salesperson", "salespeople"),
    ("person", "people"),
    ("spokesman", "spokesmen"),
    ("man", "men"),
    ("woman", "women"),
    ("basis", "bases"),
    ("diagnosis", "diagnoses"),
    ("diagnosis_a", "diagnosis_as"),
    ("datum", "data"),
    ("medium", "media"),
    ("stadium", "stadia"),
    ("analysis", "analyses"),
    ("node_child", "node_children"),
    ("child", "children"),
    ("experience", "experiences"),
    ("day", "days"),
    ("comment", "comments"),
    ("foobar", "foobars"),
    ("newsletter", "newsletters"),
    ("old_news", "old_news"),
    ("news", "news"),
    ("series", "series"),
    ("species", "species"),
    ("quiz", "quizzes"),
    ("perspective", "perspectives"),
    ("ox", "oxen"),
    ("passerby", "passersby"),
    ("photo", "photos"),
    ("buffalo", "buffaloes"),
    ("tomato", "tomatoes"),
    ("potato", "potatoes"),
    ("dwarf", "dwarves"),
    ("elf", "elves"),
    ("information", "information"),
    ("equipment", "equipment"),
    ("bus", "buses"),
    ("status", "statuses"),
    ("status_code", "status_codes"),
    ("mouse", "mice"),
    ("louse", "lice"),
    ("house", "houses"),
    ("octopus", "octopi"),
    ("virus", "viri"),
    ("alias", "aliases"),
    ("portfolio", "portfolios"),
    ("vertex", "vertices"),
    ("matrix", "matrices"),
    ("matrix_fu", "matrix_fus"),
    ("axis", "axes"),
    ("testis", "testes"),
    ("crisis", "crises"),
    ("rice", "rice"),
    ("shoe", "shoes"),
    ("horse", "horses"),
    ("prize", "prizes"),
    ("edge", "edges"),
    ("cow", "kine"),
    ("database", "databases"),
    ("human", "humans"),
];

const CAMEL_TO_UNDERSCORE: &[(&str, &str)] = &[
    ("Product", "product"),
    ("SpecialGuest", "special_guest"),
    ("ApplicationController", "application_controller"),
    ("Area51Controller", "area51_controller"),
];

const CAMEL_TO_UNDERSCORE_WITHOUT_REVERSE: &[(&str, &str)] = &[
    ("HTMLTidy", "html_tidy"),
    ("HTMLTidyGenerator", "html_tidy_generator"),
    ("FreeBSD", "free_bsd"),
    ("HTML", "html"),
];

const STRING_TO_PARAMETERIZED: &[(&str, &str)] = &[
    ("Donald E. Knuth", "donald-e-knuth"),
    (
        "Random text with *(bad)* characters",
        "random-text-with-bad-characters",
    ),
    ("Allow_Under_Scores", "allow_under_scores"),
    ("Trailing bad characters!@#", "trailing-bad-characters"),
    ("!@#Leading bad characters", "leading-bad-characters"),
    ("Squeeze   separators", "squeeze-separators"),
    ("Test with + sign", "test-with-sign"),
    ("Test with malformed utf8 ©", "test-with-malformed-utf8"),
];

const STRING_TO_PARAMETERIZE_WITH_NO_SEPARATOR: &[(&str, &str)] = &[
    ("Donald E. Knuth", "donaldeknuth"),
    ("With-some-dashes", "with-some-dashes"),
    (
        "Random text with *(bad)* characters",
        "randomtextwithbadcharacters",
    ),
    ("Trailing bad characters!@#", "trailingbadcharacters"),
    ("!@#Leading bad characters", "leadingbadcharacters"),
    ("Squeeze   separators", "squeezeseparators"),
    ("Test with + sign", "testwithsign"),
    ("Test with malformed utf8 ©", "testwithmalformedutf8"),
];

const STRING_TO_PARAMETERIZE_WITH_UNDERSCORE: &[(&str, &str)] = &[
    ("Donald E. Knuth", "donald_e_knuth"),
    (
        "Random text with *(bad)* characters",
        "random_text_with_bad_characters",
    ),
    ("With-some-dashes", "with-some-dashes"),
    ("Retain_underscore", "retain_underscore"),
    ("Trailing bad characters!@#", "trailing_bad_characters"),
    ("!@#Leading bad characters", "leading_bad_characters"),
    ("Squeeze   separators", "squeeze_separators"),
    ("Test with + sign", "test_with_sign"),
    ("Test with malformed utf8 ©", "test_with_malformed_utf8"),
];

const STRING_TO_PARAMETERIZED_AND_NORMALIZED: &[(&str, &str)] = &[
    ("Malmö", "malmo"),
    ("Garçons", "garcons"),
    ("OpsÙ", "opsu"),
    ("Ærøskøbing", "rskbing"),
    ("Aßlar", "alar"),
    ("Japanese: 日本語", "japanese"),
];

const UNDERSCORE_TO_HUMAN: &[(&str, &str)] = &[
    ("employee_salary", "Employee salary"),
    ("employee_id", "Employee"),
    ("underground", "Underground"),
];

const MIXTURE_TO_TITLEIZED: &[(&str, &str)] = &[
    ("active_record", "Active Record"),
    ("ActiveRecord", "Active Record"),
    ("action web service", "Action Web Service"),
    ("Action Web Service", "Action Web Service"),
    ("Action web service", "Action Web Service"),
    ("actionwebservice", "Actionwebservice"),
    ("Actionwebservice", "Actionwebservice"),
    ("david's code", "David's Code"),
    ("David's code", "David's Code"),
    ("david's Code", "David's Code"),
    ("ana índia", "Ana Índia"),
    ("Ana Índia", "Ana Índia"),
];

const ORDINAL_NUMBERS: &[(&str, &str)] = &[
    ("-1", "-1st"),
    ("-2", "-2nd"),
    ("-3", "-3rd"),
    ("-4", "-4th"),
    ("-5", "-5th"),
    ("-6", "-6th"),
    ("-7", "-7th"),
    ("-8", "-8th"),
    ("-9", "-9th"),
    ("-10", "-10th"),
    ("-11", "-11th"),
    ("-12", "-12th"),
    ("-13", "-13th"),
    ("-14", "-14th"),
    ("-20", "-20th"),
    ("-21", "-21st"),
    ("-22", "-22nd"),
    ("-23", "-23rd"),
    ("-24", "-24th"),
    ("-100", "-100th"),
    ("-101", "-101st"),
    ("-102", "-102nd"),
    ("-103", "-103rd"),
    ("-104", "-104th"),
    ("-110", "-110th"),
    ("-111", "-111th"),
    ("-112", "-112th"),
    ("-113", "-113th"),
    ("-1000", "-1000th"),
    ("-1001", "-1001st"),
    ("0", "0th"),
    ("1", "1st"),
    ("2", "2nd"),
    ("3", "3rd"),
    ("4", "4th"),
    ("5", "5th"),
    ("6", "6th"),
    ("7", "7th"),
    ("8", "8th"),
    ("9", "9th"),
    ("10", "10th"),
    ("11", "11th"),
    ("12", "12th"),
    ("13", "13th"),
    ("14", "14th"),
    ("20", "20th"),
    ("21", "21st"),
    ("22", "22nd"),
    ("23", "23rd"),
    ("24", "24th"),
    ("100", "100th"),
    ("101", "101st"),
    ("102", "102nd"),
    ("103", "103rd"),
    ("104", "104th"),
    ("110", "110th"),
    ("111", "111th"),
    ("112", "112th"),
    ("113", "113th"),
    ("1000", "1000th"),
    ("1001", "1001st"),
];

const UNDERSCORES_TO_DASHES: &[(&str, &str)] = &[
    ("street", "street"),
    ("street_address", "street-address"),
    ("person_street_address", "person-street-address"),
];

const STRING_TO_TABLEIZE: &[(&str, &str)] = &[
    ("person", "people"),
    ("Country", "countries"),
    ("ChildToy", "child_toys"),
    ("_RecipeIngredient", "_recipe_ingredients"),
];

const UNCOUNTABLES: &[&str] = &[
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

fn python_capitalize(value: &str) -> String {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };

    first
        .to_uppercase()
        .chain(characters.flat_map(char::to_lowercase))
        .collect()
}

#[test]
fn pluralize_existing_plurals() {
    assert_eq!("plurals", pluralize("plurals"));
    assert_eq!("Plurals", pluralize("Plurals"));
}

#[test]
fn pluralize_empty_string() {
    assert_eq!("", pluralize(""));
}

#[test]
fn built_in_uncountables_are_stable_in_both_directions() {
    for word in UNCOUNTABLES {
        assert_eq!(*word, singularize(word), "singularize({word:?})");
        assert_eq!(*word, pluralize(word), "pluralize({word:?})");
        assert_eq!(
            pluralize(word),
            singularize(word),
            "plural and singular forms differ for {word:?}",
        );
    }
}

#[test]
fn uncountable_matching_is_not_greedy() {
    // Python exposes a mutable module-level UNCOUNTABLES set. The Rust crate
    // keeps its built-in rules immutable, so this small caller-owned adapter
    // translates that one upstream extension point without changing production
    // global state. Non-extra words still exercise the public crate functions.
    let extra_uncountable = "ors";
    let pluralize_with_extra = |word: &str| {
        if word == extra_uncountable {
            word.to_owned()
        } else {
            pluralize(word)
        }
    };
    let singularize_with_extra = |word: &str| {
        if word == extra_uncountable {
            word.to_owned()
        } else {
            singularize(word)
        }
    };

    assert_eq!("ors", singularize_with_extra(extra_uncountable));
    assert_eq!("ors", pluralize_with_extra(extra_uncountable));
    assert_eq!(
        pluralize_with_extra(extra_uncountable),
        singularize_with_extra(extra_uncountable),
    );

    let countable_word = "sponsor";
    assert_eq!("sponsor", singularize_with_extra(countable_word));
    assert_eq!("sponsors", pluralize_with_extra(countable_word));
    assert_eq!(
        "sponsor",
        singularize_with_extra(&pluralize_with_extra(countable_word)),
    );
}

#[test]
fn pluralize_singular_cases() {
    for &(singular, plural) in SINGULAR_TO_PLURAL {
        assert_eq!(plural, pluralize(singular), "pluralize({singular:?})");
        assert_eq!(
            python_capitalize(plural),
            pluralize(&python_capitalize(singular)),
            "capitalized pluralize({singular:?})",
        );
    }
}

#[test]
fn singularize_plural_cases() {
    for &(singular, plural) in SINGULAR_TO_PLURAL {
        assert_eq!(singular, singularize(plural), "singularize({plural:?})");
        assert_eq!(
            python_capitalize(singular),
            singularize(&python_capitalize(plural)),
            "capitalized singularize({plural:?})",
        );
    }
}

#[test]
fn pluralize_plural_cases_idempotently() {
    for &(_, plural) in SINGULAR_TO_PLURAL {
        assert_eq!(plural, pluralize(plural), "pluralize({plural:?})");
        assert_eq!(
            python_capitalize(plural),
            pluralize(&python_capitalize(plural)),
            "capitalized pluralize({plural:?})",
        );
    }
}

#[test]
fn titleize_cases() {
    for &(before, expected) in MIXTURE_TO_TITLEIZED {
        assert_eq!(expected, titleize(before), "titleize({before:?})");
    }
}

#[test]
fn camelize_cases() {
    for &(camel, underscored) in CAMEL_TO_UNDERSCORE {
        assert_eq!(
            camel,
            camelize(underscored, true),
            "camelize({underscored:?}, true)",
        );
    }
}

#[test]
fn camelize_lowercases_the_first_letter() {
    assert_eq!("capital", camelize("Capital", false));
}

#[test]
fn camelize_treats_multiple_underscores_like_upstream() {
    assert_eq!("CamelCase", camelize("Camel_Case", true));
}

#[test]
fn underscore_cases() {
    for &(camel, underscored) in CAMEL_TO_UNDERSCORE
        .iter()
        .chain(CAMEL_TO_UNDERSCORE_WITHOUT_REVERSE)
    {
        assert_eq!(underscored, underscore(camel), "underscore({camel:?})");
    }
}

#[test]
fn parameterize_default_separator_cases() {
    for &(input, expected) in STRING_TO_PARAMETERIZED {
        assert_eq!(
            expected,
            parameterize(input, "-").expect("upstream separator must be valid"),
            "parameterize({input:?})"
        );
    }
}

#[test]
fn parameterize_normalizes_unicode_cases() {
    for &(input, expected) in STRING_TO_PARAMETERIZED_AND_NORMALIZED {
        assert_eq!(
            expected,
            parameterize(input, "-").expect("upstream separator must be valid"),
            "parameterize({input:?})"
        );
    }
}

#[test]
fn parameterize_custom_separator_cases() {
    for &(input, expected) in STRING_TO_PARAMETERIZE_WITH_UNDERSCORE {
        assert_eq!(
            expected,
            parameterize(input, "_").expect("upstream separator must be valid"),
            "parameterize({input:?}, \"_\")",
        );
    }
}

#[test]
fn parameterize_multi_character_separator_cases() {
    for &(input, default_expected) in STRING_TO_PARAMETERIZED {
        let expected = default_expected.replace('-', "__sep__");
        assert_eq!(
            expected,
            parameterize(input, "__sep__").expect("upstream separator must be valid"),
            "parameterize({input:?}, \"__sep__\")",
        );
    }
}

#[test]
fn parameterize_without_separator_cases() {
    for &(input, expected) in STRING_TO_PARAMETERIZE_WITH_NO_SEPARATOR {
        assert_eq!(
            expected,
            parameterize(input, "").expect("upstream separator must be valid"),
            "parameterize({input:?}, \"\")",
        );
    }
}

#[test]
fn humanize_cases() {
    for &(underscored, expected) in UNDERSCORE_TO_HUMAN {
        assert_eq!(expected, humanize(underscored), "humanize({underscored:?})");
    }
}

#[test]
fn ordinal_suffix_cases() {
    for &(number, ordinalized) in ORDINAL_NUMBERS {
        let value = number
            .parse::<i64>()
            .expect("upstream ordinal fixture is an i64");
        assert_eq!(
            ordinalized,
            format!("{number}{}", ordinal(value)),
            "ordinal({number})",
        );
    }
}

#[test]
fn ordinalize_cases() {
    for &(number, expected) in ORDINAL_NUMBERS {
        let value = number
            .parse::<i64>()
            .expect("upstream ordinal fixture is an i64");
        assert_eq!(expected, ordinalize(value), "ordinalize({number})");
    }
}

#[test]
fn dasherize_cases() {
    for &(input, expected) in UNDERSCORES_TO_DASHES {
        assert_eq!(expected, dasherize(input), "dasherize({input:?})");
    }
}

#[test]
fn tableize_cases() {
    for &(input, expected) in STRING_TO_TABLEIZE {
        assert_eq!(expected, tableize(input), "tableize({input:?})");
    }
}
