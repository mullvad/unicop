use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;
use miette::{miette, LabeledSpan, NamedSource, Severity};
use unic_ucd_name::Name;

use crate::config::{CodeType, Config, Language};
use crate::rules::{CharacterType, Decision, RuleSet};
use crate::unicode_notation::char_to_unicode_notation;

mod config;
mod rules;
mod unicode_blocks;
mod unicode_notation;

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

#[derive(Debug, clap::Parser)]
#[command(arg_required_else_help = true)]
struct Args {
    /// One or more files or directories to scan. Directories are scanned recursively.
    paths: Vec<PathBuf>,

    /// Print the names of all the Unicode blocks that this tool recognizes, then exits.
    ///
    /// Enable verbose output to also print the code point ranges for each block.
    #[arg(long)]
    print_unicode_blocks: bool,

    /// Print the character(s) in the given character type, then exits.
    ///
    /// As argument you can specify anything you can add to the allow end deny lists in the
    /// config file. For example:
    ///
    /// `--print-characters "Mathematical Operators"` will print all unicode code points
    /// in that block.
    ///
    /// `--print-characters U+100..U+1ff` will print all characters between 100 and 1ff (hex)
    #[arg(long)]
    print_characters: Option<CharacterType>,

    /// Enable more verbose output.
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    if args.print_unicode_blocks {
        for (&name, range) in &unicode_blocks::UNICODE_BLOCKS {
            print!("{name}");
            if args.verbose {
                let range_start = char_to_unicode_notation(*range.start());
                let range_end = char_to_unicode_notation(*range.end());
                print!(": {range_start}..{range_end}");
            }
            println!();
        }
        return Ok(());
    }

    if let Some(character_type) = args.print_characters {
        match character_type {
            CharacterType::CodePoint(c) => print_char_range(c..=c),
            CharacterType::Range(range) => print_char_range(range),
            CharacterType::Bidi => print_char_range(rules::BIDI_CHARACTERS.iter().copied()),
            CharacterType::Block(block) => print_char_range(block.clone()),
            CharacterType::Anything => print_char_range(char::MIN..=char::MAX),
        }
        return Ok(());
    }

    let default_config = get_default_config();
    let mut dispatcher = RuleDispatcher {
        user_config: None,
        default_config,
    };

    for path in args.paths {
        for entry in walkdir::WalkDir::new(path) {
            match entry {
                Err(err) => eprintln!("{:}", err),
                Ok(entry) if entry.file_type().is_file() => {
                    let entry_path = entry.path();
                    dispatcher.user_config = get_user_config(entry_path)?;
                    check_file(&dispatcher, entry_path);
                }
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
        let mut labels = Vec::new();
        if log::log_enabled!(log::Level::Debug) {
            let query = tree_sitter::Query::new(&lang.grammar(), "(ERROR) @error").unwrap();
            let mut cursor = tree_sitter::QueryCursor::new();
            for (r#match, _) in cursor.captures(&query, tree.root_node(), src.as_bytes()) {
                for capture in r#match.captures {
                    labels.push(LabeledSpan::at(
                        capture.node.start_byte()..capture.node.end_byte(),
                        "Error",
                    ));
                }
            }
        }
        let report = miette!(
            severity = Severity::Warning,
            labels = labels,
            "{}: parse error, results might be incorrect",
            filename
        )
        .with_source_code(named_source.clone());
        print!("{:?}", report);
    }
    for (off, ch) in src.char_indices() {
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
            "found disallowed character {} in {}",
            chname,
            node.kind()
        )
        .with_source_code(named_source.clone());
        print!("{:?}", report);
    }
}

fn get_user_config(path: &Path) -> anyhow::Result<Option<Config>> {
    let absolute_path = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve absolute path for {}", path.display()))?;
    let mut config_dir = if absolute_path.is_file() {
        // If scanning a file, then check for the config file in the same directory.
        absolute_path.parent().unwrap()
    } else {
        // And if scanning a dir, look for the config file in the dir.
        &absolute_path
    };

    loop {
        let config_path = config_dir.join("unicop.toml");

        match std::fs::read_to_string(&config_path) {
            Ok(config_str) => {
                log::debug!(
                    "Using config {} for scan path {}",
                    config_path.display(),
                    absolute_path.display()
                );
                break toml::from_str(&config_str).context("Failed to parse config");
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => match config_dir.parent() {
                Some(parent_dir) => config_dir = parent_dir,
                None => {
                    log::debug!("No user config for scan path {}", absolute_path.display());
                    break Ok(None);
                }
            },
            Err(e) => break Err(e).context("Failed to read config file"),
        }
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
                Language::Go,
                config::LanguageRules {
                    paths: Some(vec![glob::Pattern::new("**/*.go").unwrap()]),
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
            (
                Language::Swift,
                config::LanguageRules {
                    paths: Some(vec![glob::Pattern::new("**/*.swift").unwrap()]),
                    rules: Default::default(),
                },
            ),
            (
                Language::Typescript,
                config::LanguageRules {
                    paths: Some(vec![glob::Pattern::new("**/*.ts").unwrap()]),
                    rules: Default::default(),
                },
            ),
        ]),
    }
}

/// Prints to stdout, one line per character in the iterator.
/// The format is to first print the unicode notation followed
/// by the actual character glyph followed by the name if we know of a name.
fn print_char_range(range: impl Iterator<Item = char>) {
    for c in range {
        let code_point = char_to_unicode_notation(c);
        let char_name = match Name::of(c) {
            Some(name) => format!(" ({name})"),
            None => "".to_owned(),
        };
        println!("{code_point}: '{c}'{char_name}");
    }
}
