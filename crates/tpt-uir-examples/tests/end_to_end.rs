//! End-to-end pipeline assertion test:
//! build -> validate -> serialize -> deserialize -> print -> parse -> equality,
//! exercised across every supported format.

use tpt_uir_core::attr::Attribute;
use tpt_uir_core::builder::{BlockBuilder, OpBuilder, RegionBuilder};
use tpt_uir_core::quant::QuantizationParams;
use tpt_uir_core::types::{Dimension, ScalarType, ShapeSpec, TensorType, Type};
use tpt_uir_core::OpName;
use tpt_uir_dialects::{GpuDialect, ValidateDialect};

fn sample_region() -> tpt_uir_core::Region {
    RegionBuilder::new()
        .block(
            BlockBuilder::new()
                .arg(
                    0,
                    Type::Tensor(TensorType {
                        dtype: ScalarType::F32,
                        shape: Some(ShapeSpec {
                            dimensions: vec![Dimension::Bounded {
                                symbol: "n".to_string(),
                                max_value: 1024,
                            }],
                        }),
                    }),
                )
                .arg(1, Type::Index)
                .op(OpBuilder::new(1)
                    .name(OpName::parse("tpt_gpu.launch").unwrap())
                    .operand(0)
                    .operand(1)
                    .result(2)
                    .attribute(Attribute::dialect_version(1))
                    .attribute(Attribute::quantization(
                        "q",
                        QuantizationParams {
                            block_size: 32,
                            bytes_per_block: 18,
                            num_blocks: 4,
                            scales: Some(1),
                        },
                    ))
                    .build())
                .build(),
        )
        .build()
}

#[test]
fn end_to_end_postcard() {
    let region = sample_region();
    tpt_uir_core::validate_region(&region).expect("SSA valid");
    GpuDialect::validate_all(&region).expect("gpu dialect valid");

    let bytes = tpt_uir_serde::serialize_region(&region).unwrap();
    let back = tpt_uir_serde::deserialize_region(&bytes).unwrap();
    assert_eq!(region, back);
}

#[test]
fn end_to_end_flatbuffers() {
    let region = sample_region();

    let bytes = tpt_uir_flatbuffers::to_flatbuffer(&region);
    let back = tpt_uir_flatbuffers::from_flatbuffer(&bytes).expect("valid flatbuffer");
    assert_eq!(region, back);
}

#[test]
fn end_to_end_text() {
    let region = sample_region();

    let text = tpt_uir_text::to_text(&region);
    let back = tpt_uir_text::parse_text(&text).expect("parse ok");
    assert_eq!(region, back);

    // Stable printer: text -> parse -> print -> parse is identical.
    let text2 = tpt_uir_text::to_text(&back);
    assert_eq!(tpt_uir_text::parse_text(&text2).unwrap(), back);
}
