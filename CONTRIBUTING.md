# Notes for developers

Please read this before hacking on/contributing to `unicop`.


## Run tests

The directory [`example-files/`](example-files/) contain files for various programming languages and with various
"bad" unicode in them. These can be used to test out `unicop` while developing.

Automatic tests also run against these to verify the output of `unicop` stays consistent, and the correct
issues are raised.

## Git commit style

Please follow the guidelines at https://cbea.ms/git-commit/ for git commit message style. There is a CI
job that will enforce this.

## Updating unicode blocks

Use the `generate-unicode-blocks-consts` script to update character blocks constants from Unicode Consortium data:
