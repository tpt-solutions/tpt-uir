"""tpt_uir — Python bindings over the tpt-uir-ffi C ABI (ctypes, no build step).

Requires the native library produced by `cargo build --release -p tpt-uir-ffi`:

    from tpt_uir import Region
    with Region.load("model.tptuir") as region:
        print(region.block_count(), region.op_name(0, 0))
"""

from .ffi import TptUirError, load_library
from .region import Region

__all__ = ["Region", "TptUirError", "load_library"]
