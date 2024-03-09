# shellcheck-all

Shellcheck-all is a wrapper around the amazing
[Shellcheck](https://www.shellcheck.net/) software. It adds features to
streamline using Shellcheck for code review and continuous integration.

Shellcheck-all extends Shellcheck to support

- Running Shellcheck in parallel
- Merging and sorting the parallelized Shellcheck output into a single report
- Recursing through directories respecting gitignore using the [ignore library](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore)
- Searching for valid shell scripts by file extensions and shebangs using the [file-format library](https://github.com/mmalecot/file-format)

# Usage

Shellcheck-all is meant to be drop-in with Shellcheck, and supports the same
command line arguments for specifying Shellcheck configuration.

To Shellcheck a repo, run the following.

> shellcheck-all --format=json1 ./

Currently Shellcheck-all only supports the Shellcheck JSON1 format.

# TODO
- Add --version that reports version of Shellcheck and Shellcheck-all
- Add support for the rest of Shellcheck flags
- Support other formats then Json1
- Add github builds
- Publish to Cargo
