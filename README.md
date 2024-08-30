# Unicop

## Usage

```sh,ignore
unicop [FILES]...
```

Where `[FILES]...` is a list of files or directory to check.

## Example

```console
$ unicop example-files/homoglyph.js example-files/invisible.js
  × found non-ascii character LATIN LETTER RETROFLEX CLICK in identifier
   ╭─[example-files/homoglyph.js:4:18]
 3 │ function isUserAdmin(user) {
 4 │    if(environmentǃ=ENV_PROD){
   ·                  ┬
   ·                  ╰── LATIN LETTER RETROFLEX CLICK
 5 │        // bypass authZ checks in DEV
   ╰────

  × found non-ascii character HANGUL JUNGSEONG FILLER in
  │ shorthand_property_identifier_pattern
   ╭─[example-files/invisible.js:2:20]
 1 │ app.get('/network_health', async (req, res) => {
 2 │    const { timeout,ᅠ} = req.query;
   ·                    ┬
   ·                    ╰── HANGUL JUNGSEONG FILLER
 3 │    const checkCommands = [
   ╰────

  × found non-ascii character HANGUL JUNGSEONG FILLER in identifier
   ╭─[example-files/invisible.js:5:38]
 4 │        'ping -c 1 google.com',
 5 │        'curl -s http://example.com/',ᅠ
   ·                                      ┬
   ·                                      ╰── HANGUL JUNGSEONG FILLER
 6 │    ];
   ╰────


```

## Configuring unicop

When scanning a file, unicop will look for a `unicop.toml` config file in the same directory as the file being scanned. If one does not exist it check in the parent directory. It keeps looking up the directory tree until finding a config file or reaching the filesystem root. If no config file is found, the unicop defaults are used.

### Defaults

The default configuration is to allow all unicode code points except bidirectional (bidi) characters in
comments and in string literals. In all other places only ASCII characters are allowed.

### Example configuration

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

[language.javascript]
# Override the default path glob, only scan .js files
paths = ["**/*.js"]
# Allow everything except bidi characters everywhere.
default = {
  allow = ["*"],
  deny = ["bidi"],
}

# For python, override what files to scan, but keep the default allow/deny rules
[language.python]
paths = ["./build", "run-tests", "*.py"]
```


## Contributing to unicop

Please see the [contribution](CONTRIBUTING.md) documentation for details on how to understand, build and test
this program, as well as submitting changes.
