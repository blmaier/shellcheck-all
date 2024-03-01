# shellcheck-all

Shellcheck-all is a wrapper around the amazing
[Shellcheck](https://www.shellcheck.net/) software. Shellcheck is great for
scanning indivudal scripts, however in a static analsis system Shellcheck can
be cumbersome to use. Shellcheck does not have a way to search a directory for
scripts, requiring the user to manually search and run it. Shellcheck also runs
fully single-threaded, which can take minutes to run on a large repo.

Shellcheck-all addresses these issues. When pointed at a repo, it scans for all
shell scripts by looking for file extensions and shebangs (e.g. #!/bin/sh). It
batches the scripts and runs multiple instances of Shellcheck in parallel,
finally merging all their output together into a single report.
