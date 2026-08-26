# Contributing to TPT-UIR

Thanks for your interest in contributing! This project accepts bug reports and
issue-based contributions only — there is no formal RFC/PR-proposal process yet.

## Before you open an issue

1. Search [existing issues](https://github.com/tpt-solutions/tpt-uir/issues) to
   avoid duplicates.
2. For bugs, use the
   [bug report template](.github/ISSUE_TEMPLATE/bug_report.md) and include a
   minimal reproduction where possible.

## Environment & quality gates

Any code change must pass all CI gates locally before you push. These are
exactly the commands CI runs:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --no-default-features -- -D warnings
cargo test --workspace
cargo test --doc --workspace
cargo build --workspace --no-default-features
```

See [`AGENTS.md`](AGENTS.md) for repo layout, hard wire-format constraints, and
CLI usage. Key rules to keep in mind:

- **The postcard wire format is a contract** consumed by sibling repos — never
  renumber enum variants; add new variants only at the end.
- **`no_std` is required** for `core`/`serde`/`flatbuffers` crates.
- **FlatBuffers codegen is manual** with `flatc 24.3.25` exactly.

## Pull requests

- Keep changes focused; one logical change per PR.
- Ensure every gate above passes and add/update tests for behavior changes.
- Update the relevant crate `CHANGELOG.md` entries.

## License

By contributing you agree that your contributions are licensed under
[MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
