# Example files

This directory contains files to test-run unicop on.

These files contain "bad" unicode in various ways.

You can run `unicop` against this directory to see how it reports errors:

```console
$ unicop example-files/
? failed
  × found disallowed character LATIN LETTER RETROFLEX CLICK in identifier
    ╭─[example-files/examples.ts:11:16]
 10 │ // Homoglyph
 11 │ if (environmentǃ=ENV_PROD) {}
    ·                ┬
    ·                ╰── LATIN LETTER RETROFLEX CLICK
 12 │ 
    ╰────
  × found disallowed character HANGUL JUNGSEONG FILLER in
  │ shorthand_property_identifier_pattern
    ╭─[example-files/examples.ts:14:17]
 13 │ // Invisible
 14 │ const { timeout,ᅠ} = req.query;
    ·                 ┬
    ·                 ╰── HANGUL JUNGSEONG FILLER
    ╰────
  ⚠ example-files/homoglyph.go: parse error, results might be incorrect
  × found disallowed character LATIN LETTER RETROFLEX CLICK in identifier
   ╭─[example-files/homoglyph.go:7:16]
 6 │     environment := "..."
 7 │     if environmentǃ= EnvProd {
   ·                   ┬
   ·                   ╰── LATIN LETTER RETROFLEX CLICK
 8 │         // bypass authZ checks in DEV
   ╰────
  × found disallowed character LATIN LETTER RETROFLEX CLICK in identifier
   ╭─[example-files/homoglyph.js:4:18]
 3 │ function isUserAdmin(user) {
 4 │    if(environmentǃ=ENV_PROD){
   ·                  ┬
   ·                  ╰── LATIN LETTER RETROFLEX CLICK
 5 │        // bypass authZ checks in DEV
   ╰────
  × found disallowed character LATIN LETTER RETROFLEX CLICK in identifier
   ╭─[example-files/homoglyph.py:1:12]
 1 │ environmentǃ="PROD"
   ·            ┬
   ·            ╰── LATIN LETTER RETROFLEX CLICK
   ╰────
  × found disallowed character LATIN LETTER RETROFLEX CLICK in
  │ simple_identifier
   ╭─[example-files/homoglyph.swift:1:12]
 1 │ environmentǃ="PROD"
   ·            ┬
   ·            ╰── LATIN LETTER RETROFLEX CLICK
 2 │ 
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

Scanned 1106 unicode code points in 6 files, resulting in 8 rule violations
Failed to scan 1 file

```