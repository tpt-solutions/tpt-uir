# tpt-uir-cli

[![CI](https://github.com/tpt-solutions/tpt-uir/actions/workflows/ci.yml/badge.svg)](https://github.com/tpt-solutions/tpt-uir/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Command-line tool for working with **TPT-UIR** regions.

## Installation

Not yet on crates.io — build from source:

```sh
git clone https://github.com/tpt-solutions/tpt-uir
cd tpt-uir
cargo install --path crates/tpt-uir-cli
# or, without installing:
cargo run --release -p tpt-uir-cli -- --help
```

## Subcommands

```
tpt-uir-cli validate <file> [--dialect gpu|crucible]
tpt-uir-cli print <file>
tpt-uir-cli convert --in <file> --out <file> \
    [--from-format postcard|text|flatbuffers] \
    [--to-format postcard|text|flatbuffers]
tpt-uir-cli fmt <file.txt|file.uir> [--check]
tpt-uir-cli diff <file-a> <file-b> \
    [--format-a postcard|text|flatbuffers] \
    [--format-b postcard|text|flatbuffers]
```

- `validate` — runs SSA validation (always) plus dialect validation when
  `--dialect` is given. Exits non-zero on any violation.
- `print` — pretty-prints the region as textual TPT-UIR to stdout.
- `convert` — reads one format and writes another. Format is taken from the
  file extension when `--from-format` / `--to-format` are omitted
  (`*.tptuir` → postcard, `*.txt`/`*.uir` → text, `*.fbs`/`*.flat` → flatbuffers).
- `fmt` — rewrites a textual file into the canonical printer output. With
  `--check`, no file is written; it exits non-zero if the file is not already
  canonical (CI-friendly).
- `diff` — structurally compares two regions (any mix of formats via
  `--format-a`/`--format-b`). Prints `regions identical` and exits `0`, or
  prints `-`/`+` lines for operations that differ and exits `1`.

See [`CHANGELOG.md`](CHANGELOG.md) for release notes.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
