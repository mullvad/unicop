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
  × found disallowed character RIGHT-TO-LEFT EMBEDDING in comment
   ╭─[example-files/hello-trojan-source.c:3:19]
 2 │ int main() {
 3 │     // Will print ‫Hello World to the terminal
   ·                   ┬
   ·                   ╰── RIGHT-TO-LEFT EMBEDDING
 4 │    printf("Hello World! 👋");
   ╰────
  × found disallowed character RIGHT-TO-LEFT EMBEDDING in comment
   ╭─[example-files/hello-trojan-source.cpp:4:19]
 3 │ int main() {
 4 │     // Will print ‫Hello World to the terminal
   ·                   ┬
   ·                   ╰── RIGHT-TO-LEFT EMBEDDING
 5 │     std::cout << "Hello World! 👋";
   ╰────
  × found disallowed character RIGHT-TO-LEFT EMBEDDING in line_comment
   ╭─[example-files/hello-trojan-source.kt:1:4]
 1 │ // ‫Hello world in Kotlin
   ·    ┬
   ·    ╰── RIGHT-TO-LEFT EMBEDDING
 2 │ 
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

Scanned 1454 unicode code points in 9 files, resulting in 11 rule violations
Failed to scan 1 file

```