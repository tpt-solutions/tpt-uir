//! GPU-dialect example: a bounded-shape, thread-indexed elementwise add.
//!
//! Mirrors `crucible_static_shapes.rs` for the GPU dialect: shapes must be
//! absent or use `Dimension::Bounded`, and the kernel body is expressed as a
//! nested region guarded by `tpt_gpu.launch` / `tpt_gpu.barrier`.

use tpt_uir_core::attr::Attribute;
use tpt_uir_core::builder::{BlockBuilder, OpBuilder, RegionBuilder};
use tpt_uir_core::types::{Dimension, ScalarType, ShapeSpec, TensorType, Type};
use tpt_uir_core::OpName;
use tpt_uir_dialects::gpu::GpuOp;
use tpt_uir_dialects::{GpuDialect, ValidateDialect};

fn bounded_vec_ty(max_len: u64) -> Type {
    Type::Tensor(TensorType {
        dtype: ScalarType::F32,
        shape: Some(ShapeSpec {
            dimensions: vec![Dimension::Bounded {
                symbol: "n".into(),
                max_value: max_len as usize,
            }],
        }),
    })
}

fn main() {
    // Kernel body: read the thread index and barrier at the end of launch.
    let kernel = RegionBuilder::new()
        .block(
            BlockBuilder::new()
                .op(OpBuilder::new(10)
                    .name(OpName::parse("tpt_gpu.thread_id").unwrap())
                    .result(11)
                    .attribute(Attribute::i64("axis", 0))
                    .build())
                .op(OpBuilder::new(12)
                    .name(OpName::parse("tpt_gpu.barrier").unwrap())
                    .build())
                .build(),
        )
        .build();

    let region = RegionBuilder::new()
        .block(
            BlockBuilder::new()
                .arg(0, bounded_vec_ty(4096))
                .arg(1, bounded_vec_ty(4096))
                .op(GpuOp::build(
                    2,
                    OpName::parse("tpt_gpu.launch").unwrap(),
                    vec![0, 1],
                    vec![3],
                    vec![
                        Attribute::shape(
                            "grid",
                            ShapeSpec {
                                dimensions: vec![Dimension::Bounded {
                                    symbol: "blocks".into(),
                                    max_value: 128,
                                }],
                            },
                        ),
                        Attribute::dialect_version(1),
                    ],
                    vec![kernel],
                )
                .expect("GPU op must satisfy dialect invariants"))
                .op(OpBuilder::new(4)
                    .name(OpName::parse("tpt_gpu.dealloc").unwrap())
                    .operand(0)
                    .build())
                .build(),
        )
        .build();

    tpt_uir_core::validate_region(&region).expect("SSA must be valid");
    GpuDialect::validate_all(&region).expect("GPU dialect must be valid");

    let text = tpt_uir_text::to_text(&region);
    let parsed = tpt_uir_text::parse_text(&text).expect("parse must succeed");
    assert_eq!(region, parsed, "text round-trip must be lossless");

    println!("{}", tpt_uir_text::Pretty(&region));
}
