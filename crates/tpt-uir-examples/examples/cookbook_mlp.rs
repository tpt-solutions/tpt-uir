//! Cookbook example: a larger, realistic graph — a two-layer MLP with a fused
//! quantized matmul, bounded shapes, a nested tile loop, memory-scope ops, and
//! a full round-trip through all three wire formats.
//!
//! Run with `cargo run --example cookbook_mlp -p tpt-uir-examples`.

use tpt_uir_core::attr::Attribute;
use tpt_uir_core::builder::{BlockBuilder, OpBuilder, RegionBuilder};
use tpt_uir_core::quant::QuantizationParams;
use tpt_uir_core::types::{Dimension, ScalarType, ShapeSpec, TensorType, Type};
use tpt_uir_core::OpName;

fn hidden_ty(n_max: usize) -> Type {
    Type::Tensor(TensorType {
        dtype: ScalarType::F32,
        shape: Some(ShapeSpec {
            dimensions: vec![
                Dimension::Bounded {
                    symbol: "batch".into(),
                    max_value: n_max,
                },
                Dimension::Bounded {
                    symbol: "hidden".into(),
                    max_value: 4096,
                },
            ],
        }),
    })
}

fn main() {
    // Nested control-flow-like region: the per-tile inner loop of the GEMM.
    let tile_loop = RegionBuilder::new()
        .block(
            BlockBuilder::new()
                .arg(20, Type::Index)
                .op(OpBuilder::new(21)
                    .name(OpName::parse("core.load").unwrap())
                    .operand(20)
                    .result(22)
                    .attribute(Attribute::i64("alignment", 64))
                    .build())
                .op(OpBuilder::new(23)
                    .name(OpName::parse("core.mul").unwrap())
                    .operand(22)
                    .operand(22)
                    .result(24)
                    .build())
                .build(),
        )
        .build();

    let mlp = build_mlp(tile_loop);

    // SSA validation across all nested regions.
    tpt_uir_core::validate_region(&mlp).expect("SSA must be valid");

    // Full wire round-trip through all three formats.
    let postcard_bytes = tpt_uir_serde::serialize_region(&mlp).expect("postcard encode");
    let from_postcard =
        tpt_uir_serde::deserialize_region(&postcard_bytes).expect("postcard decode");
    assert_eq!(mlp, from_postcard);

    let fb_bytes = tpt_uir_flatbuffers::to_flatbuffer(&mlp);
    let from_fb = tpt_uir_flatbuffers::from_flatbuffer(&fb_bytes).expect("flatbuffers decode");
    assert_eq!(mlp, from_fb);

    let text = tpt_uir_text::to_text(&mlp);
    let parsed = tpt_uir_text::parse_text(&text).expect("text parse");
    assert_eq!(mlp, parsed, "text round-trip must be lossless");

    println!(
        "MLP graph: {} bytes postcard, {} bytes flatbuffers",
        postcard_bytes.len(),
        fb_bytes.len()
    );
    println!("{}", tpt_uir_text::Pretty(&mlp));
}

/// Build the MLP graph: fused quantized GEMM, memory arena scope, GPU launch.
fn build_mlp(tile_loop: tpt_uir_core::Region) -> tpt_uir_core::Region {
    RegionBuilder::new()
        .block(
            BlockBuilder::new()
                // Layer 0 input: [batch, hidden] activations.
                .arg(0, hidden_ty(512))
                .arg(1, hidden_ty(512))
                // Fused, block-quantized matmul producing Q8_0 logits.
                .op(OpBuilder::new(2)
                    .name(OpName::parse("core.mul").unwrap())
                    .operand(0)
                    .operand(1)
                    .result(3)
                    .attribute(Attribute::quantization(
                        "weights_q",
                        QuantizationParams {
                            block_size: 32,
                            bytes_per_block: 18,
                            num_blocks: 128_000,
                            scales: Some(1),
                        },
                    ))
                    .attribute(Attribute::op_dialect_version(1))
                    .region(tile_loop)
                    .build())
                .op(OpBuilder::new(4)
                    .name(OpName::parse("core.add").unwrap())
                    .operand(3)
                    .operand(3)
                    .result(5)
                    .build())
                .op(OpBuilder::new(6)
                    .name(OpName::parse("tpt_memory.scope_begin").unwrap())
                    .attribute(Attribute::string("arena", "activation_scratch"))
                    .build())
                .op(OpBuilder::new(7)
                    .name(OpName::parse("tpt_memory.alloc").unwrap())
                    .operand(5)
                    .result(8)
                    .attribute(Attribute::i64("bytes", 4096))
                    .build())
                .op(OpBuilder::new(9)
                    .name(OpName::parse("tpt_memory.scope_end").unwrap())
                    .build())
                .op(OpBuilder::new(10)
                    .name(OpName::parse("tpt_gpu.launch").unwrap())
                    .operand(8)
                    .result(11)
                    .attribute(Attribute::string("kernel", "softmax_q8"))
                    .build())
                .op(OpBuilder::new(12)
                    .name(OpName::parse("core.store").unwrap())
                    .operand(11)
                    .build())
                .build(),
        )
        .build()
}
