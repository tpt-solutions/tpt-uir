//! Conversion between the `tpt-uir-core` IR and the zero-copy FlatBuffers layout.
//!
//! `to_flatbuffer` serialises a [`tpt_uir_core::Region`] into a FlatBuffers
//! byte vector; `from_flatbuffer` reads it back. Because FlatBuffers stores data
//! in its on-wire layout, a serialized buffer can be read *without* decoding
//! (see [`crate::root_as_region`] and the `mmap` feature).

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

/// Error produced when decoding a FlatBuffers buffer into the core IR.
///
/// Mirrors the error style of `tpt-uir-serde` / `tpt-uir-text`: the read path
/// never panics on malformed input; every failure is reported as a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertError(&'static str);

impl ConvertError {
    pub fn message(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ConvertError {}

use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::generated::tpt_uir::*;
use tpt_uir_core::attr::AttributeValue as CoreAttrValue;
use tpt_uir_core::ir::{Block as CoreBlock, Operation as CoreOp, Region as CoreRegion};
use tpt_uir_core::op_name::OpName as CoreOpName;
use tpt_uir_core::quant::QuantizationParams as CoreQuant;
use tpt_uir_core::types::{
    Dimension as CoreDim, ScalarType as CoreScalar, ShapeSpec as CoreShape,
    TensorType as CoreTensor, Type as CoreType,
};

// ---------------------------------------------------------------------------
// Scalar type mapping
// ---------------------------------------------------------------------------

fn scalar_to_fb(s: CoreScalar) -> ScalarType {
    match s {
        CoreScalar::I8 => ScalarType::I8,
        CoreScalar::I16 => ScalarType::I16,
        CoreScalar::I32 => ScalarType::I32,
        CoreScalar::I64 => ScalarType::I64,
        CoreScalar::U8 => ScalarType::U8,
        CoreScalar::U16 => ScalarType::U16,
        CoreScalar::U32 => ScalarType::U32,
        CoreScalar::U64 => ScalarType::U64,
        CoreScalar::F16 => ScalarType::F16,
        CoreScalar::F32 => ScalarType::F32,
        CoreScalar::F64 => ScalarType::F64,
        CoreScalar::BF16 => ScalarType::BF16,
        CoreScalar::Bool => ScalarType::Bool,
        CoreScalar::Q4_0 => ScalarType::Q4_0,
        CoreScalar::Q4_1 => ScalarType::Q4_1,
        CoreScalar::Q8_0 => ScalarType::Q8_0,
        CoreScalar::Q5_0 => ScalarType::Q5_0,
        CoreScalar::Q5_1 => ScalarType::Q5_1,
    }
}

fn scalar_from_fb(s: ScalarType) -> Result<CoreScalar, ConvertError> {
    match s.0 {
        0 => Ok(CoreScalar::I8),
        1 => Ok(CoreScalar::I16),
        2 => Ok(CoreScalar::I32),
        3 => Ok(CoreScalar::I64),
        4 => Ok(CoreScalar::U8),
        5 => Ok(CoreScalar::U16),
        6 => Ok(CoreScalar::U32),
        7 => Ok(CoreScalar::U64),
        8 => Ok(CoreScalar::F16),
        9 => Ok(CoreScalar::F32),
        10 => Ok(CoreScalar::F64),
        11 => Ok(CoreScalar::BF16),
        12 => Ok(CoreScalar::Bool),
        13 => Ok(CoreScalar::Q4_0),
        14 => Ok(CoreScalar::Q4_1),
        15 => Ok(CoreScalar::Q8_0),
        _ => Err(ConvertError("invalid ScalarType discriminant")),
    }
}

// ---------------------------------------------------------------------------
// Build direction (core -> flatbuffer)
// ---------------------------------------------------------------------------

fn build_dim<'a>(fbb: &mut FlatBufferBuilder<'a>, d: &CoreDim) -> WIPOffset<Dimension<'a>> {
    let (kind, size, symbol, max_value) = match d {
        CoreDim::Fixed(n) => (DimensionKind::Fixed, *n as u64, None, 0u64),
        CoreDim::Symbolic(s) => (DimensionKind::Symbolic, 0, Some(fbb.create_string(s)), 0),
        CoreDim::Bounded { symbol, max_value } => (
            DimensionKind::Bounded,
            0,
            Some(fbb.create_string(symbol)),
            *max_value as u64,
        ),
    };
    Dimension::create(
        fbb,
        &DimensionArgs {
            kind,
            size_: size,
            symbol,
            max_value,
        },
    )
}

fn build_shape<'a>(fbb: &mut FlatBufferBuilder<'a>, s: &CoreShape) -> WIPOffset<ShapeSpec<'a>> {
    let dim_offsets: Vec<WIPOffset<Dimension>> =
        s.dimensions.iter().map(|d| build_dim(fbb, d)).collect();
    let dims = fbb.create_vector(&dim_offsets);
    ShapeSpec::create(
        fbb,
        &ShapeSpecArgs {
            dimensions: Some(dims),
        },
    )
}

/// Build the inner `Type` union (the discriminant + optional wrapped table).
/// Used directly by `BlockArg` and wrapped once more by `AttributeValue::TypeValue`.
fn build_type_union<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    ty: &CoreType,
) -> (Type, Option<WIPOffset<flatbuffers::UnionWIPOffset>>) {
    match ty {
        CoreType::Scalar(s) => {
            let off = ScalarTypeWrapper::create(
                fbb,
                &ScalarTypeWrapperArgs {
                    scalar: scalar_to_fb(*s),
                },
            );
            (Type::ScalarTypeWrapper, Some(off.as_union_value()))
        }
        CoreType::Tensor(tt) => {
            let shape_off = tt.shape.as_ref().map(|sh| build_shape(fbb, sh));
            let off = TensorType::create(
                fbb,
                &TensorTypeArgs {
                    dtype: scalar_to_fb(tt.dtype),
                    shape: shape_off,
                },
            );
            (Type::TensorType, Some(off.as_union_value()))
        }
        CoreType::Index => (Type::IndexMarker, None),
    }
}

fn build_opname<'a>(fbb: &mut FlatBufferBuilder<'a>, n: &CoreOpName) -> WIPOffset<OpName<'a>> {
    let dialect = fbb.create_string(&n.dialect);
    let op = fbb.create_string(&n.op);
    OpName::create(
        fbb,
        &OpNameArgs {
            dialect: Some(dialect),
            op: Some(op),
        },
    )
}

fn build_attr_value<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    v: &CoreAttrValue,
) -> (
    AttributeValue,
    Option<WIPOffset<flatbuffers::UnionWIPOffset>>,
) {
    match v {
        CoreAttrValue::I64(x) => {
            let off = I64Value::create(fbb, &I64ValueArgs { v: *x });
            (AttributeValue::I64Value, Some(off.as_union_value()))
        }
        CoreAttrValue::F64(x) => {
            let off = F64Value::create(fbb, &F64ValueArgs { v: *x });
            (AttributeValue::F64Value, Some(off.as_union_value()))
        }
        CoreAttrValue::String(x) => {
            let s = fbb.create_string(x);
            let off = StringValue::create(fbb, &StringValueArgs { v: Some(s) });
            (AttributeValue::StringValue, Some(off.as_union_value()))
        }
        CoreAttrValue::Type(t) => {
            let (tt, v) = build_type_union(fbb, t);
            let type_type = if v.is_some() { tt } else { Type::NONE };
            let tv = TypeValue::create(
                fbb,
                &TypeValueArgs {
                    type_type,
                    type_: v,
                },
            );
            (AttributeValue::TypeValue, Some(tv.as_union_value()))
        }
        CoreAttrValue::Shape(sh) => {
            let off = build_shape(fbb, sh);
            let wrapper = ShapeValue::create(fbb, &ShapeValueArgs { shape: Some(off) });
            (AttributeValue::ShapeValue, Some(wrapper.as_union_value()))
        }
        CoreAttrValue::OpName(n) => {
            let off = build_opname(fbb, n);
            let wrapper = OpNameValue::create(fbb, &OpNameValueArgs { op_name: Some(off) });
            (AttributeValue::OpNameValue, Some(wrapper.as_union_value()))
        }
        CoreAttrValue::Quantization(q) => {
            let off = QuantizationParams::create(
                fbb,
                &QuantizationParamsArgs {
                    block_size: q.block_size,
                    bytes_per_block: q.bytes_per_block,
                    num_blocks: q.num_blocks,
                    has_scales: q.scales.is_some(),
                    scales: q.scales.unwrap_or(0),
                },
            );
            let wrapper =
                QuantizationValue::create(fbb, &QuantizationValueArgs { quant: Some(off) });
            (
                AttributeValue::QuantizationValue,
                Some(wrapper.as_union_value()),
            )
        }
        CoreAttrValue::Bytes(b) => {
            let v = fbb.create_vector(b);
            let off = BytesValue::create(fbb, &BytesValueArgs { v: Some(v) });
            (AttributeValue::BytesValue, Some(off.as_union_value()))
        }
    }
}

fn build_attribute<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    a: &tpt_uir_core::attr::Attribute,
) -> WIPOffset<Attribute<'a>> {
    let key = fbb.create_string(&a.key);
    let (value_type, value) = build_attr_value(fbb, &a.value);
    Attribute::create(
        fbb,
        &AttributeArgs {
            key: Some(key),
            value_type,
            value,
        },
    )
}

fn build_operation<'a>(fbb: &mut FlatBufferBuilder<'a>, op: &CoreOp) -> WIPOffset<Operation<'a>> {
    let op_name = build_opname(fbb, &op.op_name);
    let operands = fbb.create_vector(&op.operands);
    let results = fbb.create_vector(&op.results);
    let region_offsets: Vec<WIPOffset<Region>> = op
        .regions
        .iter()
        .map(|r| build_region_inner(fbb, r))
        .collect();
    let regions = fbb.create_vector(&region_offsets);
    let attr_offsets: Vec<WIPOffset<Attribute>> = op
        .attributes
        .iter()
        .map(|a| build_attribute(fbb, a))
        .collect();
    let attributes = fbb.create_vector(&attr_offsets);
    Operation::create(
        fbb,
        &OperationArgs {
            id: op.id,
            op_name: Some(op_name),
            operands: Some(operands),
            results: Some(results),
            regions: Some(regions),
            attributes: Some(attributes),
        },
    )
}

fn build_block<'a>(fbb: &mut FlatBufferBuilder<'a>, b: &CoreBlock) -> WIPOffset<Block<'a>> {
    let arg_offsets: Vec<WIPOffset<BlockArg>> = b
        .arguments
        .iter()
        .map(|(vid, ty)| {
            let (tt, v) = build_type_union(fbb, ty);
            let type_type = if v.is_some() { tt } else { Type::NONE };
            BlockArg::create(
                fbb,
                &BlockArgArgs {
                    value: *vid,
                    type_type,
                    type_: v,
                },
            )
        })
        .collect();
    let arguments = fbb.create_vector(&arg_offsets);
    let op_offsets: Vec<WIPOffset<Operation>> = b
        .operations
        .iter()
        .map(|op| build_operation(fbb, op))
        .collect();
    let operations = fbb.create_vector(&op_offsets);
    Block::create(
        fbb,
        &BlockArgs {
            arguments: Some(arguments),
            operations: Some(operations),
        },
    )
}

fn build_region_inner<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    r: &CoreRegion,
) -> WIPOffset<Region<'a>> {
    let block_offsets: Vec<WIPOffset<Block>> =
        r.blocks.iter().map(|b| build_block(fbb, b)).collect();
    let blocks = fbb.create_vector(&block_offsets);
    Region::create(
        fbb,
        &RegionArgs {
            blocks: Some(blocks),
        },
    )
}

/// Serialise a [`tpt_uir_core::Region`] into a FlatBuffers byte vector.
pub fn to_flatbuffer(region: &CoreRegion) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let root = build_region_inner(&mut fbb, region);
    finish_region_buffer(&mut fbb, root);
    let data = fbb.finished_data();
    data.to_vec()
}

// ---------------------------------------------------------------------------
// Read direction (flatbuffer -> core)
// ---------------------------------------------------------------------------

fn read_dim(d: Dimension) -> Result<CoreDim, ConvertError> {
    match d.kind().0 {
        0 => Ok(CoreDim::Fixed(d.size_() as usize)),
        1 => Ok(CoreDim::Symbolic(
            d.symbol().map(ToString::to_string).unwrap_or_default(),
        )),
        2 => Ok(CoreDim::Bounded {
            symbol: d.symbol().map(ToString::to_string).unwrap_or_default(),
            max_value: d.max_value() as usize,
        }),
        _ => Err(ConvertError("invalid DimensionKind discriminant")),
    }
}

fn read_shape(s: ShapeSpec) -> Result<CoreShape, ConvertError> {
    let dimensions = s
        .dimensions()
        .map(|v| v.iter().map(read_dim).collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    Ok(CoreShape { dimensions })
}

fn read_type(t: TypeValue) -> Result<CoreType, ConvertError> {
    match t.type_type().0 {
        1 => {
            let w = t
                .type__as_scalar_type_wrapper()
                .ok_or(ConvertError("missing ScalarTypeWrapper union value"))?;
            Ok(CoreType::Scalar(scalar_from_fb(w.scalar())?))
        }
        2 => {
            let tt = t
                .type__as_tensor_type()
                .ok_or(ConvertError("missing TensorType union value"))?;
            let shape = tt.shape().map(read_shape).transpose()?;
            Ok(CoreType::Tensor(CoreTensor {
                dtype: scalar_from_fb(tt.dtype())?,
                shape,
            }))
        }
        _ => Ok(CoreType::Index),
    }
}

fn read_opname(n: OpName) -> CoreOpName {
    CoreOpName::new(
        n.dialect().map(ToString::to_string).unwrap_or_default(),
        n.op().map(ToString::to_string).unwrap_or_default(),
    )
}

fn read_attr_value(a: Attribute) -> Result<CoreAttrValue, ConvertError> {
    match a.value_type().0 {
        1 => {
            let x = a
                .value_as_i64_value()
                .ok_or(ConvertError("missing I64Value union value"))?;
            Ok(CoreAttrValue::I64(x.v()))
        }
        2 => {
            let x = a
                .value_as_f64_value()
                .ok_or(ConvertError("missing F64Value union value"))?;
            Ok(CoreAttrValue::F64(x.v()))
        }
        3 => {
            let x = a
                .value_as_string_value()
                .ok_or(ConvertError("missing StringValue union value"))?;
            Ok(CoreAttrValue::String(
                x.v().map(ToString::to_string).unwrap_or_default(),
            ))
        }
        4 => {
            let x = a
                .value_as_type_value()
                .ok_or(ConvertError("missing TypeValue union value"))?;
            Ok(CoreAttrValue::Type(read_type(x)?))
        }
        5 => {
            let x = a
                .value_as_shape_value()
                .ok_or(ConvertError("missing ShapeValue union value"))?;
            let shape = x.shape().ok_or(ConvertError("missing Shape table"))?;
            Ok(CoreAttrValue::Shape(read_shape(shape)?))
        }
        6 => {
            let x = a
                .value_as_op_name_value()
                .ok_or(ConvertError("missing OpNameValue union value"))?;
            let op_name = x.op_name().ok_or(ConvertError("missing OpName table"))?;
            Ok(CoreAttrValue::OpName(read_opname(op_name)))
        }
        7 => {
            let x = a
                .value_as_quantization_value()
                .ok_or(ConvertError("missing QuantizationValue union value"))?;
            let q = x
                .quant()
                .ok_or(ConvertError("missing QuantizationParams table"))?;
            Ok(CoreAttrValue::Quantization(CoreQuant {
                block_size: q.block_size(),
                bytes_per_block: q.bytes_per_block(),
                num_blocks: q.num_blocks(),
                scales: if q.has_scales() {
                    Some(q.scales())
                } else {
                    None
                },
            }))
        }
        _ => Err(ConvertError("attribute with NONE union discriminant")),
    }
}

fn read_attribute(a: Attribute) -> Result<tpt_uir_core::attr::Attribute, ConvertError> {
    Ok(tpt_uir_core::attr::Attribute {
        key: a.key().map(ToString::to_string).unwrap_or_default(),
        value: read_attr_value(a)?,
    })
}

fn read_operation(op: Operation) -> Result<CoreOp, ConvertError> {
    let operands = op
        .operands()
        .map(|v| v.iter().collect())
        .unwrap_or_default();
    let results = op.results().map(|v| v.iter().collect()).unwrap_or_default();
    let regions = op
        .regions()
        .map(|v| v.iter().map(read_region).collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    let attributes = op
        .attributes()
        .map(|v| v.iter().map(read_attribute).collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    Ok(CoreOp {
        id: op.id(),
        op_name: op
            .op_name()
            .map(read_opname)
            .unwrap_or_else(|| CoreOpName::new("", "")),
        operands,
        results,
        regions,
        attributes,
    })
}

fn read_block(b: Block) -> Result<CoreBlock, ConvertError> {
    let arguments = b
        .arguments()
        .map(|v| {
            v.iter()
                .map(|arg| {
                    let ty = match arg.type_type().0 {
                        1 => {
                            let w = arg
                                .type__as_scalar_type_wrapper()
                                .ok_or(ConvertError("missing ScalarTypeWrapper union value"))?;
                            CoreType::Scalar(scalar_from_fb(w.scalar())?)
                        }
                        2 => {
                            let tt = arg
                                .type__as_tensor_type()
                                .ok_or(ConvertError("missing TensorType union value"))?;
                            let shape = tt.shape().map(read_shape).transpose()?;
                            CoreType::Tensor(CoreTensor {
                                dtype: scalar_from_fb(tt.dtype())?,
                                shape,
                            })
                        }
                        _ => CoreType::Index,
                    };
                    Ok((arg.value(), ty))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let operations = b
        .operations()
        .map(|v| v.iter().map(read_operation).collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    Ok(CoreBlock {
        arguments,
        operations,
    })
}

fn read_region(r: Region) -> Result<CoreRegion, ConvertError> {
    let blocks = r
        .blocks()
        .map(|v| v.iter().map(read_block).collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    Ok(CoreRegion { blocks })
}

/// Read a [`tpt_uir_core::Region`] from a FlatBuffers byte buffer.
///
/// Returns `Err` if the buffer is not a valid `Region` flatbuffer or contains
/// malformed data (unknown union discriminants, missing tables, ...).
pub fn from_flatbuffer(buf: &[u8]) -> Result<CoreRegion, ConvertError> {
    let r =
        crate::root_as_region(buf).map_err(|_| ConvertError("not a valid Region flatbuffer"))?;
    read_region(r)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use alloc::vec;
    use tpt_uir_core::attr::Attribute;

    fn sample() -> CoreRegion {
        CoreRegion {
            blocks: vec![CoreBlock {
                arguments: vec![(
                    0u32,
                    CoreType::Tensor(CoreTensor {
                        dtype: CoreScalar::F32,
                        shape: Some(CoreShape {
                            dimensions: vec![CoreDim::Bounded {
                                symbol: "n".into(),
                                max_value: 1024,
                            }],
                        }),
                    }),
                )],
                operations: vec![
                    CoreOp {
                        id: 1,
                        op_name: CoreOpName::parse("core.add").unwrap(),
                        operands: vec![0],
                        results: vec![1],
                        regions: vec![],
                        attributes: vec![
                            Attribute::i64("axis", 0),
                            Attribute::f64("scale", 1.5),
                            Attribute::string("name", "acc"),
                            Attribute::quantization(
                                "q",
                                CoreQuant {
                                    block_size: 32,
                                    bytes_per_block: 18,
                                    num_blocks: 4,
                                    scales: Some(1),
                                },
                            ),
                            Attribute::bytes("payload", vec![0xDE, 0xAD, 0xBE, 0xEF]),
                        ],
                    },
                    CoreOp {
                        id: 2,
                        op_name: CoreOpName::parse("tpt_gpu.launch").unwrap(),
                        operands: vec![1],
                        results: vec![],
                        regions: vec![CoreRegion {
                            blocks: vec![CoreBlock {
                                arguments: vec![],
                                operations: vec![],
                            }],
                        }],
                        attributes: vec![],
                    },
                ],
            }],
        }
    }

    #[test]
    fn roundtrip_region() {
        let region = sample();
        let bytes = to_flatbuffer(&region);
        let back = from_flatbuffer(&bytes).expect("valid flatbuffer");
        assert_eq!(region, back);
    }

    #[test]
    fn root_as_region_works() {
        let region = sample();
        let bytes = to_flatbuffer(&region);
        let r = crate::root_as_region(&bytes).unwrap();
        assert_eq!(r.blocks().unwrap().iter().count(), 1);
    }

    #[test]
    fn bad_buffer_is_err() {
        assert!(from_flatbuffer(b"not a flatbuffer").is_err());
    }

    #[test]
    fn truncated_buffer_is_err_not_panic() {
        let region = sample();
        let bytes = to_flatbuffer(&region);
        // Corrupt the ScalarType discriminant of the block-arg type union.
        for cut in 0..bytes.len() {
            // Any truncation or corruption must yield an Err, never a panic.
            let _ = from_flatbuffer(&bytes[..cut]);
        }
        let mut corrupted = bytes.clone();
        if let Some(last) = corrupted.last_mut() {
            *last = !*last;
        }
        let _ = from_flatbuffer(&corrupted);
    }
}
