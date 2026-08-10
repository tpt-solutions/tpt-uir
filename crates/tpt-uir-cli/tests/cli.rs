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
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

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
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

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
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));

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
