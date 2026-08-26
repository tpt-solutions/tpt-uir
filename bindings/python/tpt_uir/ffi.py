"""ctypes loader and error type for the tpt-uir-ffi C ABI.

Error codes follow the C contract:
0 = ok, 1 = bad pointer/handle/index, 2 = decode error, 3 = interior NUL.
"""

import ctypes
import os
import sys
from ctypes.util import find_library

_ERROR_MESSAGES = {
    1: "bad pointer, handle or index",
    2: "postcard decode error",
    3: "string contains an interior NUL byte",
}

_BINDINGS_DONE = False


class TptUirError(RuntimeError):
    """Raised when an FFI call returns a non-zero error code."""

    def __init__(self, function: str, code: int) -> None:
        detail = _ERROR_MESSAGES.get(code, f"unknown error code {code}")
        super().__init__(f"{function} failed with code {code}: {detail}")
        self.code = code


def _bind(lib: ctypes.CDLL) -> None:
    """Attach argtypes/restype declarations (idempotent)."""
    global _BINDINGS_DONE
    if _BINDINGS_DONE:
        return
    c_int, c_void_p = ctypes.c_int, ctypes.c_void_p
    size_p = ctypes.POINTER(ctypes.c_size_t)
    u32_p = ctypes.POINTER(ctypes.c_uint32)
    char_pp = ctypes.POINTER(ctypes.c_char_p)
    void_pp = ctypes.POINTER(c_void_p)
    sz = ctypes.c_size_t

    lib.tpt_uir_load.argtypes = [ctypes.POINTER(ctypes.c_ubyte), sz, void_pp]
    lib.tpt_uir_load.restype = c_int
    lib.tpt_uir_free.argtypes = [c_void_p]
    lib.tpt_uir_block_count.argtypes = [c_void_p, size_p]
    lib.tpt_uir_block_count.restype = c_int
    lib.tpt_uir_op_count.argtypes = [c_void_p, sz, size_p]
    lib.tpt_uir_op_count.restype = c_int
    lib.tpt_uir_op_dialect.argtypes = [c_void_p, sz, sz, char_pp]
    lib.tpt_uir_op_dialect.restype = c_int
    lib.tpt_uir_op_op.argtypes = [c_void_p, sz, sz, char_pp]
    lib.tpt_uir_op_op.restype = c_int
    lib.tpt_uir_op_operand_count.argtypes = [c_void_p, sz, sz, size_p]
    lib.tpt_uir_op_operand_count.restype = c_int
    lib.tpt_uir_op_operand.argtypes = [c_void_p, sz, sz, sz, u32_p]
    lib.tpt_uir_op_operand.restype = c_int
    lib.tpt_uir_validate_ssa.argtypes = [c_void_p]
    lib.tpt_uir_validate_ssa.restype = c_int
    lib.tpt_uir_validate_dialect.argtypes = [c_void_p, ctypes.c_char_p]
    lib.tpt_uir_validate_dialect.restype = c_int
    _BINDINGS_DONE = True


def load_library() -> ctypes.CDLL:
    """Locate and load the native tpt_uir_ffi library."""
    override = os.environ.get("TPT_UIR_LIBRARY")
    if override:
        lib = ctypes.CDLL(override)
        _bind(lib)
        return lib

    if sys.platform == "win32":
        rel, fname = ("target", "release"), "tpt_uir_ffi.dll"
    elif sys.platform == "darwin":
        rel, fname = ("target", "release"), "libtpt_uir_ffi.dylib"
    else:
        rel, fname = ("target", "release"), "libtpt_uir_ffi.so"

    here = os.path.dirname(os.path.abspath(__file__))
    # bindings/python/tpt_uir -> repo root
    repo_root = os.path.abspath(os.path.join(here, "..", "..", ".."))
    candidates = [os.path.join(repo_root, *rel, fname)]
    stem = fname.replace("lib", "").split(".")[0]
    found = find_library(stem)
    if found:
        candidates.append(found)

    last_error = None
    for path in candidates:
        try:
            lib = ctypes.CDLL(path)
            _bind(lib)
            return lib
        except OSError as exc:
            last_error = exc
    raise OSError(
        "cannot load tpt_uir_ffi; build it with "
        "`cargo build --release -p tpt-uir-ffi` or set TPT_UIR_LIBRARY"
    ) from last_error
