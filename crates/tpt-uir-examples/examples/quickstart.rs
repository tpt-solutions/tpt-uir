//! Quick-start tour of TPT-UIR.
//!
//! Builds a region with the fluent builders, runs SSA + dialect validation,
//! attaches quantization and dialect-version metadata, then round-trips the
//! region through the postcard, FlatBuffers, and textual formats.

use tpt_uir_core::attr::Attribute;
use tpt_uir_core::builder::{BlockBuilder, OpBuilder, RegionBuilder};
use tpt_uir_core::quant::QuantizationParams;
use tpt_uir_core::types::{Dimension, ScalarType, ShapeSpec, TensorType, Type};
use tpt_uir_core::OpName;
use tpt_uir_dialects::{GpuDialect, ValidateDialect};

fn main() {
    let region = RegionBuilder::new()
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
                        "weights",
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
        .build();

    // SSA well-formedness + GPU dialect invariants (Bounded/None shapes).
    tpt_uir_core::validate_region(&region).expect("SSA must be valid");
    GpuDialect::validate_all(&region).expect("GPU dialect must be valid");

    // Postcard (compact, self-describing) round-trip.
    let postcard = tpt_uir_serde::serialize_region(&region).unwrap();
    let from_postcard = tpt_uir_serde::deserialize_region(&postcard).unwrap();
    assert_eq!(
        region, from_postcard,
        "postcard round-trip must be lossless"
    );

    // FlatBuffers (zero-copy on the wire) round-trip.
    let fb = tpt_uir_flatbuffers::to_flatbuffer(&region);
    let from_fb = tpt_uir_flatbuffers::from_flatbuffer(&fb).expect("valid flatbuffer");
    assert_eq!(region, from_fb, "flatbuffers round-trip must be lossless");

    // Textual format round-trip.
    let text = tpt_uir_text::to_text(&region);
    let from_text = tpt_uir_text::parse_text(&text).expect("parse must succeed");
    assert_eq!(region, from_text, "text round-trip must be lossless");

    println!("{}", tpt_uir_text::Pretty(&region));
    println!(
        "round-trips OK: postcard={} bytes, flatbuffers={} bytes, text={} bytes",
        postcard.len(),
        fb.len(),
        text.len()
    );
}
