# Unicop

## Usage

```sh,ignore
unicop [FILES]...
```

Where `[FILES]...` is a list of files or directory to check.

## Example

```console
$ unicop example-files/homoglyph.js example-files/invisible.js
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

```

## Contributing to unicop

Please see the [contribution](CONTRIBUTING.md) documentation for details on how to understand, build and test
this program, as well as submitting changes.

## Todo

Things left to implement to make this usable

* Recursively scan a directory. Check all files matching some criteria (extension matching compatible parsers?)
* Add language detection machinery (mapping from file extension to tree-sitter parser)
* Some way to specify an allowlist and denylist of unicode code points per language parser. This should have
  sane defaults: Comments and string literals allow all unicode except Bidi characters, all other kinds of code deny all unicode.

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

# For everything not comments, Rust falls back to the `global` settings above.


[language.python]
# Custom paths for python. Look at all *.py-files, but also look at 'build' and 'run-tests'
# in the root path
paths = ["**/*.py", "./build", "run-tests"]

# Since there are no special rules for python defined here, evaluation falls back
# to the rules in `global` above.
```
