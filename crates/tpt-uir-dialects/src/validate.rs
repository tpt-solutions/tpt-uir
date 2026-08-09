extern crate alloc;

use alloc::vec::Vec;

use tpt_uir_core::Region;

use crate::crucible::validate_crucible_region;
use crate::gpu::validate_gpu_region;

/// Trait implemented by dialect marker types to validate a [`Region`] against
/// the invariants of that dialect.
pub trait ValidateDialect {
    fn validate(region: &Region) -> Result<(), Vec<String>>;
}

impl ValidateDialect for crate::gpu::GpuDialect {
    fn validate(region: &Region) -> Result<(), Vec<String>> {
        validate_gpu_region(region)
    }
}

impl ValidateDialect for crate::crucible::CrucibleDialect {
    fn validate(region: &Region) -> Result<(), Vec<String>> {
        validate_crucible_region(region)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crucible::CrucibleDialect;
    use crate::gpu::GpuDialect;
    use tpt_uir_core::types::{Dimension, ScalarType, ShapeSpec, TensorType};
    use tpt_uir_core::{Block, Region, Type};

    #[test]
    fn gpu_dialect_trait_rejects_fixed() {
        let region = Region {
            blocks: vec![Block {
                arguments: vec![(
                    0u32,
                    Type::Tensor(TensorType {
                        dtype: ScalarType::F32,
                        shape: Some(ShapeSpec {
                            dimensions: vec![Dimension::Fixed(4)],
                        }),
                    }),
                )],
                operations: vec![],
            }],
        };
        assert!(GpuDialect::validate(&region).is_err());
    }

    #[test]
    fn crucible_dialect_trait_accepts_fixed() {
        let region = Region {
            blocks: vec![Block {
                arguments: vec![(
                    0u32,
                    Type::Tensor(TensorType {
                        dtype: ScalarType::F32,
                        shape: Some(ShapeSpec {
                            dimensions: vec![Dimension::Fixed(4096)],
                        }),
                    }),
                )],
                operations: vec![],
            }],
        };
        assert!(CrucibleDialect::validate(&region).is_ok());
    }
}
