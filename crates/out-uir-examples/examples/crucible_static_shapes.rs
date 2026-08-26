//! Contrast case: the Crucible dialect requires fully static (`Fixed`) shapes,
//! unlike the GPU dialect which uses `Bounded`/`None`. This example builds a
//! static-shaped Crucible region and validates it.

use tpt_uir_core::attr::Attribute;
use tpt_uir_core::builder::{BlockBuilder, OpBuilder, RegionBuilder};
use tpt_uir_core::types::{Dimension, ScalarType, ShapeSpec, TensorType, Type};
use tpt_uir_core::OpName;
use tpt_uir_dialects::crucible::target_ip_block;
use tpt_uir_dialects::{CrucibleDialect, CrucibleOp, ValidateDialect};

fn main() {
    let region = RegionBuilder::new()
        .block(
            BlockBuilder::new()
                .arg(
                    0,
                    Type::Tensor(TensorType {
                        dtype: ScalarType::F32,
                        shape: Some(ShapeSpec {
                            dimensions: vec![Dimension::Fixed(1024), Dimension::Fixed(4096)],
                        }),
                    }),
                )
                .op(CrucibleOp::build(
                    1,
                    OpName::parse("tpt_crucible.map_flash").unwrap(),
                    vec![0],
                    vec![1],
                    vec![Attribute::dialect_version(1)],
                    vec![],
                ))
                .op(OpBuilder::new(2)
                    .name(OpName::parse("tpt_crucible.route_fpga").unwrap())
                    .operand(1)
                    .result(2)
                    .attribute(target_ip_block("dsp_slice_4"))
                    .build())
                .build(),
        )
        .build();

    tpt_uir_core::validate_region(&region).expect("SSA must be valid");
    CrucibleDialect::validate_all(&region).expect("Crucible dialect must be valid");

    let text = tpt_uir_text::to_text(&region);
    let parsed = tpt_uir_text::parse_text(&text).expect("parse must succeed");
    assert_eq!(region, parsed, "text round-trip must be lossless");

    println!("{}", tpt_uir_text::Pretty(&region));
}
