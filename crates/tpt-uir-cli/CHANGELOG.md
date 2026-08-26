# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-10

### Added
- Initial release of the TPT-UIR command-line tool.
- `validate` subcommand: runs SSA validation (always) plus dialect validation
  when `--dialect gpu|crucible` is given.
- `print` subcommand: pretty-prints a region as textual TPT-UIR to stdout.
- `convert` subcommand: reads one format and writes another (postcard, text,
  flatbuffers), with format inferred from file extension when flags are omitted.
- `fmt` subcommand: rewrites textual files into canonical printer form;
  `--check` exits non-zero without writing if the file is not already canonical.
- `diff` subcommand: structural comparison of two regions in any format mix
  (`--format-a`/`--format-b`); exits `0` when identical, `1` with `-`/`+`
  operation lines otherwise.
