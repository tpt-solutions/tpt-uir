# tpt-uir-cli

Command-line tool for working with **TPT-UIR** regions.

## Subcommands

```
tpt-uir-cli validate <file> [--dialect gpu|crucible]
tpt-uir-cli print <file>
tpt-uir-cli convert --in <file> --out <file> \
    [--from-format postcard|text|flatbuffers] \
    [--to-format postcard|text|flatbuffers]
```

- `validate` — runs SSA validation (always) plus dialect validation when
  `--dialect` is given. Exits non-zero on any violation.
- `print` — pretty-prints the region as textual TPT-UIR to stdout.
- `convert` — reads one format and writes another. Format is taken from the
  file extension when `--from-format` / `--to-format` are omitted
  (`*.tptuir` → postcard, `*.txt`/`*.uir` → text, `*.fbs`/`*.flat` → flatbuffers).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
