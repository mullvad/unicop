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
        Some(char::from_u32(int_number)?)
    };
    parse(unicode_notation).ok_or_else(|| InvalidCharacterType(unicode_notation.to_owned()))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CodeType {
    Comment,
    StringLiteral,
    Identifiers,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Language {
    Rust,
    Javascript,
    Python,
}

#[derive(Debug, Eq, PartialEq, Default, serde::Deserialize)]
struct ConfigRules {
    #[serde(default)]
    default: RuleSet,
    #[serde(flatten)]
    code_type_rules: HashMap<CodeType, RuleSet>,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize)]
struct LanguageRules {
    #[serde(default)]
    paths: Vec<String>,
    #[serde(flatten)]
    rules: ConfigRules,
}

#[derive(Debug, Eq, PartialEq, Default, serde::Deserialize)]
struct Config {
    #[serde(default)]
    global: ConfigRules,
    #[serde(default)]
    language: HashMap<Language, LanguageRules>,
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
        let config: Config = toml::from_str(INVALID_LANGUAGE).unwrap();
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
                    paths: vec![],
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