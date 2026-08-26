use std::process::Command;

use tpt_uir_core::op_name::OpName;
use tpt_uir_core::{Block, Operation, Region, Type};
use tpt_uir_serde::{serialize_region, write_tptuir};

fn sample() -> Region {
    Region {
        blocks: vec![Block {
            arguments: vec![(0u32, Type::Index)],
            operations: vec![Operation {
                id: 1,
                op_name: OpName::parse("core.add").unwrap(),
                operands: vec![0],
                results: vec![1],
                regions: vec![],
                attributes: vec![],
            }],
        }],
    }
}

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tpt-uir-cli"))
}

#[test]
fn cli_validate_ok() {
    let region = sample();
    let path = std::env::temp_dir().join("tpt_uir_cli_test.tptuir");
    write_tptuir(&path, &region).unwrap();

    let out = bin().arg("validate").arg(&path).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn cli_print_contains_op() {
    let region = sample();
    let bytes = serialize_region(&region).unwrap();
    let path = std::env::temp_dir().join("tpt_uir_cli_print.tptuir");
    std::fs::write(&path, bytes).unwrap();

    let out = bin().arg("print").arg(&path).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("core.add"), "got: {}", stdout);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn cli_convert_roundtrip() {
    let region = sample();
    let bytes = serialize_region(&region).unwrap();
    let in_path = std::env::temp_dir().join("tpt_uir_cli_in.tptuir");
    std::fs::write(&in_path, bytes).unwrap();
    let txt_path = std::env::temp_dir().join("tpt_uir_cli_out.txt");
    let fbs_path = std::env::temp_dir().join("tpt_uir_cli_out.fbs");

    // postcard -> text
    let out = bin()
        .arg("convert")
        .arg("--in")
        .arg(&in_path)
        .arg("--out")
        .arg(&txt_path)
        .arg("--to-format")
        .arg("text")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // text -> flatbuffers
    let out = bin()
        .arg("convert")
        .arg("--in")
        .arg(&txt_path)
        .arg("--out")
        .arg(&fbs_path)
        .arg("--from-format")
        .arg("text")
        .arg("--to-format")
        .arg("flatbuffers")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // flatbuffers -> postcard and compare
    let fbs_bytes = std::fs::read(&fbs_path).unwrap();
    let decoded = tpt_uir_flatbuffers::from_flatbuffer(&fbs_bytes).expect("valid flatbuffer");
    assert_eq!(region, decoded);

    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&txt_path);
    let _ = std::fs::remove_file(&fbs_path);
}

#[test]
fn cli_validate_rejects_bad_ssa() {
    // Operand 5 is undefined -> SSA validation must fail.
    let bad = Region {
        blocks: vec![Block {
            arguments: vec![],
            operations: vec![Operation {
                id: 1,
                op_name: OpName::parse("core.add").unwrap(),
                operands: vec![5],
                results: vec![],
                regions: vec![],
                attributes: vec![],
            }],
        }],
    };
    let path = std::env::temp_dir().join("tpt_uir_cli_bad.tptuir");
    write_tptuir(&path, &bad).unwrap();

    let out = bin().arg("validate").arg(&path).output().unwrap();
    assert!(!out.status.success());

    let _ = std::fs::remove_file(&path);
}

fn write_text(path: &std::path::Path, region: &Region) -> String {
    let text = tpt_uir_text::to_text(region);
    std::fs::write(path, &text).unwrap();
    text
}

#[test]
fn cli_fmt_check_ok_on_canonical() {
    let region = sample();
    let path = std::env::temp_dir().join("tpt_uir_fmt_ok.uir");
    write_text(&path, &region);

    let out = bin().arg("fmt").arg("--check").arg(&path).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn cli_fmt_reformats_noncanonical() {
    let region = sample();
    // Same IR with trailing whitespace / different spacing.
    let canonical = tpt_uir_text::to_text(&region);
    let path = std::env::temp_dir().join("tpt_uir_fmt_fix.uir");
    std::fs::write(&path, format!("{}\n\n", canonical)).unwrap();

    // --check must fail on non-canonical input.
    let out = bin().arg("fmt").arg("--check").arg(&path).output().unwrap();
    assert!(!out.status.success());

    // Plain fmt must rewrite the file to canonical form.
    let out = bin().arg("fmt").arg(&path).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(after, canonical);

    let _ = std::fs::remove_file(&path);
}

#[test]
fn cli_diff_identical_regions() {
    let region = sample();
    let a = std::env::temp_dir().join("tpt_uir_diff_a.uir");
    let b = std::env::temp_dir().join("tpt_uir_diff_b.tptuir");
    write_text(&a, &region);
    // Same region in a *different* wire format must still be identical.
    let bytes = serialize_region(&region).unwrap();
    std::fs::write(&b, bytes).unwrap();

    let out = bin()
        .arg("diff")
        .arg(&a)
        .arg("--format-b")
        .arg("postcard")
        .arg(&b)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("regions identical"), "got: {}", stdout);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn cli_diff_reports_differences() {
    let mut modified = sample();
    modified.blocks[0].operations[0].operands.push(0);

    let a = std::env::temp_dir().join("tpt_uir_diff_c.uir");
    let b = std::env::temp_dir().join("tpt_uir_diff_d.uir");
    write_text(&a, &sample());
    write_text(&b, &modified);

    let out = bin().arg("diff").arg(&a).arg(&b).output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("regions differ"), "got: {}", stdout);
    assert!(stdout.contains("op 1 core.add"), "got: {}", stdout);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}
