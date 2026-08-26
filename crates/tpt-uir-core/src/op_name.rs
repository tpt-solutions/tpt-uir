use alloc::string::String;

/// A namespaced operation name of the form `"dialect.op"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpName {
    pub dialect: String,
    pub op: String,
}

impl OpName {
    pub fn new(dialect: impl Into<String>, op: impl Into<String>) -> Self {
        OpName {
            dialect: dialect.into(),
            op: op.into(),
        }
    }

    /// Parse a `"dialect.op"` string into an [`OpName`].
    ///
    /// Returns `Err` if there is no single `.` separator, or if either side is
    /// empty.
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        match s.split_once('.') {
            Some((d, o)) if !d.is_empty() && !o.is_empty() => Ok(OpName::new(d, o)),
            _ => Err("invalid OpName: expected \"dialect.op\""),
        }
    }
}

impl core::fmt::Display for OpName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}", self.dialect, self.op)
    }
}

// Core dialect
pub const CORE_ADD: &str = "core.add";
pub const CORE_MUL: &str = "core.mul";
pub const CORE_LOAD: &str = "core.load";
pub const CORE_STORE: &str = "core.store";

// GPU dialect
pub const TPT_GPU_LAUNCH: &str = "tpt_gpu.launch";
pub const TPT_GPU_THREAD_ID: &str = "tpt_gpu.thread_id";
pub const TPT_GPU_BARRIER: &str = "tpt_gpu.barrier";
pub const TPT_GPU_ALLOC: &str = "tpt_gpu.alloc";
pub const TPT_GPU_DEALLOC: &str = "tpt_gpu.dealloc";

// Crucible dialect — hardware placement/routing
pub const TPT_CRUCIBLE_MAP_FLASH: &str = "tpt_crucible.map_flash";
pub const TPT_CRUCIBLE_ROUTE_FPGA: &str = "tpt_crucible.route_fpga";
pub const TPT_CRUCIBLE_ANALOG_CONV: &str = "tpt_crucible.analog_conv";

// Crucible dialect — compute ops (mirrors tpt_crucible_common::ops::Op)
pub const TPT_CRUCIBLE_INPUT: &str = "tpt_crucible.input";
pub const TPT_CRUCIBLE_CONSTANT: &str = "tpt_crucible.constant";
pub const TPT_CRUCIBLE_OUTPUT: &str = "tpt_crucible.output";
pub const TPT_CRUCIBLE_CAST: &str = "tpt_crucible.cast";
pub const TPT_CRUCIBLE_ADD: &str = "tpt_crucible.add";
pub const TPT_CRUCIBLE_SUB: &str = "tpt_crucible.sub";
pub const TPT_CRUCIBLE_MUL: &str = "tpt_crucible.mul";
pub const TPT_CRUCIBLE_DIV: &str = "tpt_crucible.div";
pub const TPT_CRUCIBLE_NEG: &str = "tpt_crucible.neg";
pub const TPT_CRUCIBLE_EXP: &str = "tpt_crucible.exp";
pub const TPT_CRUCIBLE_LOG: &str = "tpt_crucible.log";
pub const TPT_CRUCIBLE_SQRT: &str = "tpt_crucible.sqrt";
pub const TPT_CRUCIBLE_SIN: &str = "tpt_crucible.sin";
pub const TPT_CRUCIBLE_COS: &str = "tpt_crucible.cos";
pub const TPT_CRUCIBLE_TANH: &str = "tpt_crucible.tanh";
pub const TPT_CRUCIBLE_ERF: &str = "tpt_crucible.erf";
pub const TPT_CRUCIBLE_RELU: &str = "tpt_crucible.relu";
pub const TPT_CRUCIBLE_SIGMOID: &str = "tpt_crucible.sigmoid";
pub const TPT_CRUCIBLE_SILU: &str = "tpt_crucible.silu";
pub const TPT_CRUCIBLE_GELU: &str = "tpt_crucible.gelu";
pub const TPT_CRUCIBLE_MATMUL: &str = "tpt_crucible.matmul";
pub const TPT_CRUCIBLE_SOFTMAX: &str = "tpt_crucible.softmax";
pub const TPT_CRUCIBLE_RMS_NORM: &str = "tpt_crucible.rms_norm";
pub const TPT_CRUCIBLE_LAYER_NORM: &str = "tpt_crucible.layer_norm";
pub const TPT_CRUCIBLE_ATTENTION: &str = "tpt_crucible.attention";
pub const TPT_CRUCIBLE_ROPE: &str = "tpt_crucible.rope";
pub const TPT_CRUCIBLE_EMBEDDING: &str = "tpt_crucible.embedding";
pub const TPT_CRUCIBLE_TRANSPOSE: &str = "tpt_crucible.transpose";
pub const TPT_CRUCIBLE_RESHAPE: &str = "tpt_crucible.reshape";
pub const TPT_CRUCIBLE_CONCAT: &str = "tpt_crucible.concat";

// Memory dialect
pub const TPT_MEMORY_SCOPE_BEGIN: &str = "tpt_memory.scope_begin";
pub const TPT_MEMORY_ALLOC: &str = "tpt_memory.alloc";
pub const TPT_MEMORY_SCOPE_END: &str = "tpt_memory.scope_end";
