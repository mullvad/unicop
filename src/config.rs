use core::fmt;
use std::collections::HashMap;
use std::str::FromStr;

use crate::rules::{CharacterType, RuleSet};

#[derive(Debug)]
pub struct InvalidCharacterType(String);

impl fmt::Display for InvalidCharacterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}' is not a valid character type", self.0)
    }
}

impl std::error::Error for InvalidCharacterType {}

impl<'de> serde::Deserialize<'de> for CharacterType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for CharacterType {
    type Err = InvalidCharacterType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "bidi" {
            return Ok(Self::Bidi);
        }
        if s == "*" {
            return Ok(Self::Anything);
        }
        for block in unic_ucd_block::BlockIter::new() {
            if block.name == s {
                return Ok(Self::Block(block));
            }
        }
        if let Some((low, high)) = s.split_once("..") {
            let low = unicode_notation_to_char(low)?;
            let high = unicode_notation_to_char(high)?;
            return Ok(Self::Range(unic_char_range::CharRange { low, high }));
        }
        unicode_notation_to_char(s).map(Self::CodePoint)
    }
}

fn unicode_notation_to_char(unicode_notation: &str) -> Result<char, InvalidCharacterType> {
    let parse = |unicode_notation: &str| -> Option<char> {
        let hex_str_number = unicode_notation.strip_prefix("U+")?;
        let int_number = u32::from_str_radix(hex_str_number, 16).ok()?;
        char::from_u32(int_number)
    };
    parse(unicode_notation).ok_or_else(|| InvalidCharacterType(unicode_notation.to_owned()))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeType {
    Comment,
    StringLiteral,
    Identifiers,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    Rust,
    Javascript,
    Python,
}

static RUST_CODE_TYPES: phf::Map<&'static str, CodeType> = phf::phf_map! {
    "doc_comment" => CodeType::Comment,
    "line_comment" => CodeType::Comment,
    "string_content" => CodeType::StringLiteral,
    "char_literal" => CodeType::StringLiteral,
};

static JAVASCRIPT_CODE_TYPES: phf::Map<&'static str, CodeType> = phf::phf_map! {
    "comment" => CodeType::Comment,
    "block_comment" => CodeType::Comment,
    "string_fragment" => CodeType::StringLiteral,
};

static PYTHON_CODE_TYPES: phf::Map<&'static str, CodeType> = phf::phf_map! {
    "string_content" => CodeType::StringLiteral,
    "comment" => CodeType::Comment,
};

impl Language {
    pub fn lookup_code_type(&self, tree_sitter_code_type: &str) -> Option<CodeType> {
        match self {
            Language::Javascript => JAVASCRIPT_CODE_TYPES.get(tree_sitter_code_type).copied(),
            Language::Rust => RUST_CODE_TYPES.get(tree_sitter_code_type).copied(),
            Language::Python => PYTHON_CODE_TYPES.get(tree_sitter_code_type).copied(),
        }
    }

    pub fn grammar(&self) -> tree_sitter::Language {
        match self {
            Language::Javascript => tree_sitter_javascript::language(),
            Language::Python => tree_sitter_python::language(),
            Language::Rust => tree_sitter_rust::language(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Default, serde::Deserialize)]
pub struct ConfigRules {
    #[serde(default)]
    pub default: RuleSet,
    #[serde(flatten)]
    pub code_type_rules: HashMap<CodeType, RuleSet>,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize)]
pub struct LanguageRules {
    // None = Inherit default path globs
    // Some([]) = No paths will ever match this language
    // Some([...]) = Match every file against these glob patterns.
    //               Run this language parser if at least one matches.
    #[serde(default, deserialize_with = "deserialize_pattern")]
    pub paths: Option<Vec<glob::Pattern>>,
    #[serde(flatten)]
    pub rules: ConfigRules,
}

fn deserialize_pattern<'de, D>(deserializer: D) -> Result<Option<Vec<glob::Pattern>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<Vec<String>> = serde::Deserialize::deserialize(deserializer)?;
    match s {
        None => Ok(None),
        Some(v) => {
            let res = v
                .iter()
                .map(|s| glob::Pattern::new(&s))
                .collect::<Result<Vec<glob::Pattern>, _>>()
                .map_err(serde::de::Error::custom)?;
            Ok(Some(res))
        }
    }
}

#[derive(Debug, Eq, PartialEq, Default, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub global: ConfigRules,
    #[serde(default)]
    pub language: HashMap<Language, LanguageRules>,
}

#[cfg(test)]
mod tests {
    use unic_char_range::CharRange;
    use unic_ucd_block::BlockIter;

    use super::*;
    use crate::rules::*;

    #[test]
    fn empty_config() {
        let config: Config = toml::from_str("").unwrap();
        let expected_config = Config {
            global: ConfigRules {
                default: RuleSet {
                    allow: vec![],
                    deny: vec![],
                },
                code_type_rules: HashMap::new(),
            },
            language: HashMap::new(),
        };
        assert_eq!(config, expected_config);
    }

    #[test]
    #[should_panic]
    fn invalid_language() {
        static INVALID_LANGUAGE: &str = "[language.nonon]";
        let _config: Config = toml::from_str(INVALID_LANGUAGE).unwrap();
    }

    #[test]
    fn some_config() {
        let config: Config = toml::from_str(
            r#"
[global.default]
allow = ["U+1234"]
deny = ["*"]

[global.comment]
allow = ["*"]
deny = ["bidi"]

[language.rust]
paths = ["**/*.rs"]

[language.rust.default]
allow = ["Tibetan", "U+9000"]
deny = ["U+5000..U+5004"]

[language.rust.string-literal]
deny = ["Tibetan"]
"#,
        )
        .unwrap();

        let tibetan_block = BlockIter::new().find(|b| b.name == "Tibetan").unwrap();

        let expected_config = Config {
            global: ConfigRules {
                default: RuleSet {
                    allow: vec![CharacterType::CodePoint('\u{1234}')],
                    deny: vec![CharacterType::Anything],
                },
                code_type_rules: HashMap::from([(
                    CodeType::Comment,
                    RuleSet {
                        allow: vec![CharacterType::Anything],
                        deny: vec![CharacterType::Bidi],
                    },
                )]),
            },
            language: HashMap::from([(
                Language::Rust,
                LanguageRules {
                    paths: Some(vec![glob::Pattern::new("**/*.rs").unwrap()]),
                    rules: ConfigRules {
                        default: RuleSet {
                            allow: vec![
                                CharacterType::Block(tibetan_block),
                                CharacterType::CodePoint('\u{9000}'),
                            ],
                            deny: vec![CharacterType::Range(CharRange {
                                low: '\u{5000}',
                                high: '\u{5004}',
                            })],
                        },
                        code_type_rules: HashMap::from([(
                            CodeType::StringLiteral,
                            RuleSet {
                                allow: vec![],
                                deny: vec![CharacterType::Block(tibetan_block)],
                            },
                        )]),
                    },
                },
            )]),
        };
        assert_eq!(config, expected_config);
    }
}
