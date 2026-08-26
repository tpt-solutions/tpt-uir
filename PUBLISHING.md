# Crates.io Publishing Log

Tracks the state of each crate's publication to [crates.io](https://crates.io).
Update this file whenever a new version of any crate is published.

Publish order matters: dependents must wait for their `path` dependencies to
land on the registry index before they can be packaged.

| Crate | Version | Status | Published | Notes |
|---|---|---|---|---|
| tpt-uir-core | 0.1.0 | published | 2026-08-27 | no internal deps |
| tpt-uir-serde | 0.1.0 | published | 2026-08-27 | depends on tpt-uir-core |
| tpt-uir-text | 0.1.0 | published | 2026-08-27 | depends on tpt-uir-core |
| tpt-uir-flatbuffers | 0.1.0 | published | 2026-08-27 | depends on tpt-uir-core |
| tpt-uir-dialects | 0.1.0 | published | 2026-08-27 | depends on tpt-uir-core |
| tpt-uir-ffi | 0.1.0 | published | 2026-08-27 | depends on tpt-uir-core, tpt-uir-serde |
| tpt-uir-cli | 0.1.0 | published | 2026-08-27 | depends on tpt-uir-core, tpt-uir-serde, tpt-uir-text, tpt-uir-flatbuffers, tpt-uir-dialects |

Not published (internal/dev-only, `publish = false`):
- out-uir-examples
- out-uir-wasm
