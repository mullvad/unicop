# Unicop

Unicop is a tool that scans source code and detects unwanted unicode code points.
This is used to detect and prevent attacks such as [Trojan Source] and attacks where invisible characters
and homoglyphs smuggle backdoors into the program. Here are some examples of articles
on the topic:

* [PortSwigger](https://portswigger.net/daily-swig/smuggling-hidden-backdoors-into-javascript-with-homoglyphs-and-invisible-unicode-characters)
* [Bleeping Computer](https://www.bleepingcomputer.com/news/security/invisible-characters-could-be-hiding-backdoors-in-your-javascript-code/)
* [Certitude](https://certitude.consulting/blog/en/invisible-backdoor/)

[Trojan Source]: https://en.wikipedia.org/wiki/Trojan_Source

## Background and motivation

This tool was written because before it, the only two options to handle the above issues were to:
1. Not care. or,
2. Only allow ASCII in all your source code.

But this is a bit limiting. The above attack vectors should be taken serious, but there are many
legitimate use cases for non-ASCII unicode code points in source code. Some common uses
of non-ASCII unicode in source code:
* Use math symbols and other special symbols in comments to explain how the code works.
* Use various languages in comments and tests for code that deal with localization.
* Write comments in non-English for software developed by people who don't have English
  as their native language.
* Use other languages than English in string literals, for software that target non-English
  speaking users.
* Use emojis, math symbols, box drawing symbols etc in string literals for software that
  want to output these symbols for some reason. For example to draw a fancy terminal UI.

## Installation

If you already have Rust on your machine, you can install `unicop` directly with cargo:
```ignore
cargo install --locked unicop
```

## Usage

The intended use case is to run this tool in the same places as where you would run automatic
code analysis, CVE scanners and similar. Probably in your [CI pipeline](#running-in-ci).
But it can of course also be ran locally.

```sh,ignore
unicop [PATHS]...
```

Where `[PATHS]...` is a list of files or directory to check.

### Example usage

```console
$ unicop example-files/homoglyph.js example-files/invisible.js example-files/not-utf-8-file.ts
? failed
  × found disallowed character LATIN LETTER RETROFLEX CLICK in identifier
   ╭─[example-files/homoglyph.js:4:18]
 3 │ function isUserAdmin(user) {
 4 │    if(environmentǃ=ENV_PROD){
   ·                  ┬
   ·                  ╰── LATIN LETTER RETROFLEX CLICK
 5 │        // bypass authZ checks in DEV
   ╰────
  × found disallowed character HANGUL JUNGSEONG FILLER in
  │ shorthand_property_identifier_pattern
   ╭─[example-files/invisible.js:2:20]
 1 │ app.get('/network_health', async (req, res) => {
 2 │    const { timeout,ᅠ} = req.query;
   ·                    ┬
   ·                    ╰── HANGUL JUNGSEONG FILLER
 3 │    const checkCommands = [
   ╰────
  × found disallowed character HANGUL JUNGSEONG FILLER in identifier
   ╭─[example-files/invisible.js:5:38]
 4 │        'ping -c 1 google.com',
 5 │        'curl -s http://example.com/',ᅠ
   ·                                      ┬
   ·                                      ╰── HANGUL JUNGSEONG FILLER
 6 │    ];
   ╰────
Error while scanning example-files/not-utf-8-file.ts: Failed to read file (stream did not contain valid UTF-8)

Scanned 486 unicode code points in 2 files, resulting in 3 rule violations
Failed to scan 1 file

```

## Running in CI

This is an example of how you can run `unicop` in Github Actions for your repository:

```yml
---
name: Malicious unicode scanning
on:
  pull_request:
  push:
  workflow_dispatch:

jobs:
  unicop:
    # Unicop is not platform specific in any way. The results should be identical
    # for any input source code, no matter what platform it runs on.
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup default stable
          rustup update stable

      - name: Install unicop
        run: |
          cargo install --locked unicop
          unicop --version

      - name: Check for unwanted unicode
        run: unicop --verbose .
```

## Configuration

By default, the tool allows anything, except [bidirectional control characters] in comments and
string literals. In all other code, only ASCII characters are allowed.

You can configure this behavior in a `unicop.toml` file. `unicop` will use the first `unicop.toml`
file it can find, starting in the same directory as the file that is being scanned,
and then traversing up to the parent directory.

There are global config rules and there are language specific config rules. The global rules
are used when the language specific rules for the scanned code does not explicitly allow
or deny a specific code point.

Within each language specific rule section there are default rules and rules for specific
code types. The default rules are used for code that is not of a type that has type specific
rules, or where the type specific rules does not have an allow or deny verdict about the scanned
code point.

The rule evaluation order is:

1. Code type specific rules for the specific programming language being scanned
2. Default rules for the specific programming language being scanned
3. Code type specific rules in the global section
4. Default rules in the global section

The language specific sections can also configure glob patterns for what filenames should
be scanned with that language's parser and rules. These have sane defaults for most languages,
and should only need to be changed if you name your files something unconventional.

[bidirectional control characters]: https://en.wikipedia.org/wiki/Unicode_control_characters#Bidirectional_text_control

### Example config

Here is an example config file. Maybe not a sane default for most projects. This is mostly
just showcasing what you can configure.

```toml
# Define global rules that apply as a fallback when there are no language specific rules
# or those language specific rules don't make a decision about a code point.
[global]
# In general, only allow ascii, denying all non-ascii code points by default.
default = { allow = ["ascii"] }
# Be a bit more forgiving in comments and string literals. But still deny bidirectional
# modifiers, to avoid attacks where code is made to look like it is inside a comment or string,
# but it actually is not.
comment = { allow = ["*"], deny = ["bidi"] }
string-literal = { allow = ["*"], deny = ["bidi"] }


[language.rust]
# In Rust comments, allow ascii, unicode currency symbols and the thumbs up emoji,
# nothing else. This means Rust comments allow less stuff than comments in other
# languages, in this config.
comment = { allow = ["ascii", "Currency Symbols", "U+1F44D"], deny = ["*"]}

# For every code point not in a comment, Rust falls back to the `global` settings above.


[language.python]
# Custom paths for python. Look at all *.py-files, but also look at 'build' and 'run-tests'
# in the root path
paths = ["**/*.py", "./build", "run-tests"]

# Since there are no special rules for python defined here, evaluation falls back
# to the rules in `global` above.
```

## Contributing to unicop

Please see the [contribution](CONTRIBUTING.md) documentation for details on how to understand, build and test
this program, as well as submitting changes.
