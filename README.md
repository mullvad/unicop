# Unicop

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
