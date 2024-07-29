use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

use anyhow::Context;
use config::CodeType;
use config::Config;
use config::Language;
use miette::{miette, LabeledSpan, NamedSource, Severity};
use rules::Decision;
use rules::RuleSet;
use unic_ucd_name::Name;

mod config;
mod rules;
mod unicode_blocks;

// Replaces the previous idea of "RuleChain"s.
struct RuleDispatcher {
    user_config: Option<Config>,
    default_config: Config,
}

impl RuleDispatcher {
    pub fn language(&self, filepath: &Path) -> Option<Language> {
        if let Some(userconf) = &self.user_config {
            if let Some(lang) = Self::language_for_config(userconf, filepath) {
                return Some(lang);
            }
        }
        if let Some(lang) = Self::language_for_config(&self.default_config, filepath) {
            return Some(lang);
        }
        None
    }

    fn language_for_config(config: &Config, filepath: &Path) -> Option<Language> {
        for (lang, langconf) in &config.language {
            if let Some(paths) = &langconf.paths {
                for glob in paths {
                    if glob.matches_path(filepath) {
                        return Some(*lang);
                    }
                }
            }
        }
        None
    }

    pub fn decision(&self, c: char, language: Language, code_type: Option<CodeType>) -> Decision {
        if let Some(user_config) = &self.user_config {
            if let Some(decision) = Self::decision_for_config(user_config, c, language, code_type) {
                return decision;
            }
        }
        if let Some(decision) =
            Self::decision_for_config(&self.default_config, c, language, code_type)
        {
            return decision;
        }
        Decision::Deny
    }

    // Rulechain:
    // 1. Code type specific ruleset for specific language
    // 2. Default ruleset for specific language
    // 3. Code type specific ruleset in global section
    // 4. Default rules in global section
    fn decision_for_config(
        config: &Config,
        c: char,
        language: Language,
        code_type: Option<CodeType>,
    ) -> Option<Decision> {
        if let Some(language_rules) = config.language.get(&language) {
            // 1.
            if let Some(language_code_type_rules) =
                code_type.and_then(|ct| language_rules.rules.code_type_rules.get(&ct))
            {
                if let Some(decision) = language_code_type_rules.decision(c) {
                    return Some(decision);
                }
            }
            // 2.
            if let Some(decision) = language_rules.rules.default.decision(c) {
                return Some(decision);
            }
        }
        // 3.
        if let Some(global_code_type_rules) =
            code_type.and_then(|ct| config.global.code_type_rules.get(&ct))
        {
            if let Some(decision) = global_code_type_rules.decision(c) {
                return Some(decision);
            }
        }
        // 4.
        if let Some(decision) = config.global.default.decision(c) {
            return Some(decision);
        }
        // This config does not have any opinion on this character
        None
    }
}

fn main() -> anyhow::Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        args = vec![String::from(".")]
    }

    let default_config = get_default_config();
    let user_config = get_user_config()?;
    let dispatcher = RuleDispatcher {
        user_config,
        default_config,
    };

    for arg in args {
        for entry in walkdir::WalkDir::new(arg) {
            match entry {
                Err(err) => eprintln!("{:}", err),
                Ok(entry) if entry.file_type().is_file() => check_file(&dispatcher, entry.path()),
                Ok(_) => {}
            }
        }
    }
    Ok(())
}

fn check_file(dispatcher: &RuleDispatcher, path: &Path) {
    let Some(lang) = dispatcher.language(path) else {
        return;
    };
    let filename = path.display().to_string();
    let src = fs::read_to_string(path).unwrap();
    let named_source = NamedSource::new(&filename, src.clone());
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang.grammar())
        .expect("Error loading grammar");
    let tree = parser.parse(&src, None).unwrap();
    if tree.root_node().has_error() {
        println!(
            "{:?}",
            miette!(severity = Severity::Warning, "{}: parse error", filename)
                .with_source_code(named_source.clone())
        );
    }
    for (off, ch) in src.char_indices() {
        if ch.is_ascii() {
            continue;
        }
        let node = tree
            .root_node()
            .named_descendant_for_byte_range(off, off + ch.len_utf8())
            .unwrap();
        let tskind = node.kind();
        let code_type = lang.lookup_code_type(tskind);
        match dispatcher.decision(ch, lang, code_type) {
            Decision::Allow => continue,
            Decision::Deny => {}
        }
        let chname = Name::of(ch).unwrap();
        let report = miette!(
            labels = vec![LabeledSpan::at(
                off..off + ch.len_utf8(),
                chname.to_string()
            )],
            "found non-ascii character {} in {}",
            chname,
            node.kind()
        )
        .with_source_code(named_source.clone());
        println!("{:?}", report);
    }
}

fn get_user_config() -> anyhow::Result<Option<Config>> {
    match std::fs::read_to_string("./unicop.toml") {
        Ok(config_str) => toml::from_str(&config_str).context("Failed to parse config"),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).context("Failed to read config file"),
    }
}

/// Comments and string literals allow all unicode except Bidi characters,
/// all other kinds of code deny all unicode.
fn get_default_config() -> Config {
    Config {
        global: config::ConfigRules {
            default: RuleSet {
                allow: vec![rules::CharacterType::Block(&unicode_blocks::BASIC_LATIN)],
                deny: vec![],
            },
            code_type_rules: [
                (
                    config::CodeType::Comment,
                    RuleSet {
                        allow: vec![rules::CharacterType::Anything],
                        deny: vec![rules::CharacterType::Bidi],
                    },
                ),
                (
                    config::CodeType::StringLiteral,
                    RuleSet {
                        allow: vec![rules::CharacterType::Anything],
                        deny: vec![rules::CharacterType::Bidi],
                    },
                ),
            ]
            .into_iter()
            .collect(),
        },
        language: HashMap::from([
            (
                Language::Rust,
                config::LanguageRules {
                    paths: Some(vec![glob::Pattern::new("**/*.rs").unwrap()]),
                    rules: Default::default(),
                },
            ),
            (
                Language::Python,
                config::LanguageRules {
                    paths: Some(vec![glob::Pattern::new("**/*.py").unwrap()]),
                    rules: Default::default(),
                },
            ),
            (
                Language::Javascript,
                config::LanguageRules {
                    paths: Some(vec![
                        glob::Pattern::new("**/*.js").unwrap(),
                        glob::Pattern::new("**/*.mjs").unwrap(),
                        glob::Pattern::new("**/*.cjs").unwrap(),
                        glob::Pattern::new("**/*.jsx").unwrap(),
                    ]),
                    rules: Default::default(),
                },
            ),
        ]),
    }
}
