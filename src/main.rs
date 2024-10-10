use std::path::PathBuf;

use clap::Parser;

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
    print_characters: Option<unicop::CharacterType>,

    /// Enable more verbose output.
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    if args.print_unicode_blocks {
        unicop::print_unicode_blocks(args.verbose);
        return Ok(());
    }
    if let Some(character_type) = args.print_characters {
        unicop::print_character_type(character_type);
        return Ok(());
    }

    let default_config = unicop::get_default_config();
    let mut dispatcher = unicop::RuleDispatcher {
        user_config: None,
        default_config,
    };

    for path in args.paths {
        for entry in walkdir::WalkDir::new(path) {
            match entry {
                Err(err) => eprintln!("{:}", err),
                Ok(entry) if entry.file_type().is_file() => {
                    let entry_path = entry.path();
                    dispatcher.user_config = unicop::get_user_config(entry_path)?;
                    unicop::check_file(&dispatcher, entry_path);
                }
                Ok(_) => {}
            }
        }
    }
    Ok(())
}
