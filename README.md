# Unicop

## Usage

```sh,ignore
unicop [FILES]...
```

Where `[FILES]...` is a list of files or directory to check, default: `.`.

## Example

```console
$ unicop examples/homoglyph.js examples/invisible.js
  × found non-ascii character LATIN LETTER RETROFLEX CLICK in identifier
   ╭─[examples/homoglyph.js:4:18]
 3 │ function isUserAdmin(user) {
 4 │    if(environmentǃ=ENV_PROD){
   ·                  ┬
   ·                  ╰── LATIN LETTER RETROFLEX CLICK
 5 │        // bypass authZ checks in DEV
   ╰────

  × found non-ascii character HANGUL JUNGSEONG FILLER in
  │ shorthand_property_identifier_pattern
   ╭─[examples/invisible.js:2:20]
 1 │ app.get('/network_health', async (req, res) => {
 2 │    const { timeout,ᅠ} = req.query;
   ·                    ┬
   ·                    ╰── HANGUL JUNGSEONG FILLER
 3 │    const checkCommands = [
   ╰────

  × found non-ascii character HANGUL JUNGSEONG FILLER in identifier
   ╭─[examples/invisible.js:5:38]
 4 │        'ping -c 1 google.com',
 5 │        'curl -s http://example.com/',ᅠ
   ·                                      ┬
   ·                                      ╰── HANGUL JUNGSEONG FILLER
 6 │    ];
   ╰────


```

## Todo

Things left to implement to make this usable

* Recursively scan a directory. Check all files matching some criteria (extension matching compatible parsers?)
* Add language detection machinery (mapping from file extension to tree-sitter parser)
* Some way to specify an allowlist and denylist of unicode code points per language parser. This should have
  sane defaults: Comments and string literals allow all unicode except Bidi characters, all other kinds of code deny all unicode.

```toml
[global]
default = {
  allow = ["ascii"]
}
comment = {
  allow = ["*"]
  deny = ["bidi"]
}
string-literal = {
  allow = ["*"]
  deny = ["bidi"]
}

[language.rust]
default = {
  allow = ["emoji"]
  deny = []
}

comment = {
  allow = ["u+1234"],
  deny = ["bidi"],
}
string-literal = {
  allow = ["u+1234"],
  deny = ["bidi"],
}
identifiers = {
  deny = ["u+90"]
}

[language.javascript]
paths = ["**/*.js"]
default = {
  allow = ["unicode"],
  deny = ["bidi"],
}

[language.python]
paths = ["./build", "run-tests", "*.py"]
```

