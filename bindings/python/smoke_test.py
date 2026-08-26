"""Smoke test for the ctypes bindings (run after building tpt-uir-ffi).

Usage: python smoke_test.py
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(HERE))

from tpt_uir import Region, TptUirError  # noqa: E402

# Same postcard fixture as crates/tpt-uir-ffi/examples/basic.c:
BYTES = bytes(
    [
        0x01, 0x02, 0x00, 0x02, 0x01, 0x02, 0x01, 0x01, 0x07,
        ord("t"), ord("p"), ord("t"), ord("_"), ord("g"),
        ord("p"), ord("u"), 0x06,
        ord("l"), ord("a"), ord("u"), ord("n"), ord("c"), ord("h"),
        0x02, 0x00, 0x01, 0x00, 0x00, 0x00,
    ]
)


def main() -> int:
    # Make sure the native library exists; build it if not.
    dll = (
        "tpt_uir_ffi.dll"
        if sys.platform == "win32"
        else "libtpt_uir_ffi.dylib"
        if sys.platform == "darwin"
        else "libtpt_uir_ffi.so"
    )
    lib_path = os.path.join(ROOT, "target", "release", dll)
    if not os.path.exists(lib_path):
        print("building tpt-uir-ffi (release) ...")
        subprocess.run(
            ["cargo", "build", "--release", "-p", "tpt-uir-ffi"],
            cwd=ROOT,
            check=True,
        )

    with Region.load_bytes(BYTES) as region:
        assert region.block_count() == 1
        assert region.op_count(0) == 1
        name = region.op_name(0, 0)
        assert name == "tpt_gpu.launch", name
        assert region.operand_count(0) == 2
        assert region.operand(0, 1) == 1
        assert region.validate_ssa()
        assert region.validate_dialect("tpt_gpu")
        assert not region.validate_dialect("crucible")
        print(f"ok: {name}, ssa valid")

    try:
        Region.load_bytes(b"\xff\xff\xff\xff")
        raise AssertionError("expected decode failure")
    except TptUirError as exc:
        print(f"ok: bad input rejected ({exc})")

    return 0


if __name__ == "__main__":
    sys.exit(main())
