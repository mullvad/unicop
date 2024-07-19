use std::env;
use std::fs;
use std::path::Path;

use miette::{miette, LabeledSpan, NamedSource, Severity};
use unic_ucd_name::Name;

mod config;
mod rules;

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        args = vec![String::from(".")]
    }
    for arg in args {
        for entry in walkdir::WalkDir::new(arg) {
            match entry {
                Err(err) => eprintln!("{:}", err),
                Ok(entry) if entry.file_type().is_file() => check_file(entry.path()),
                Ok(_) => {}
            }
        }
    }
}

fn check_file(path: &Path) {
    let Some(lang) = detect_language(path) else {
        return;
    };
    let filename = path.display().to_string();
    let src = fs::read_to_string(path).unwrap();
    let named_source = NamedSource::new(&filename, src.clone());
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&lang).expect("Error loading grammar");
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
        let kind = node.kind();
        if kind == "comment" || kind == "string_fragment" {
            continue;
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

// Tree-sitter grammars include some configurations to help decide whether the language applies to
// a given file.
// Unfortunately, neither the language-detection algorithm nor the configurations are included in
// the Rust crates. So for now we have a simplified language-detection with hard-coded
// configurations.
// See https://tree-sitter.github.io/tree-sitter/syntax-highlighting#language-detection
fn detect_language(path: &Path) -> Option<tree_sitter::Language> {
    match path.extension()?.to_str()? {
        // https://github.com/tree-sitter/tree-sitter-javascript/blob/master/package.json
        "js" | "mjs" | "cjs" | "jsx" => Some(tree_sitter_javascript::language()),
        // https://github.com/tree-sitter/tree-sitter-python/blob/master/package.json
        "py" => Some(tree_sitter_python::language()),
        _ => None,
    }
}
