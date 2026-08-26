"""High-level Region handle over the tpt-uir-ffi C ABI."""

import ctypes

from .ffi import TptUirError, load_library


class Region:
    """Opaque handle to a deserialized TPT-UIR region."""

    def __init__(self, handle: ctypes.c_void_p, lib: ctypes.CDLL) -> None:
        if not handle:
            raise ValueError("null handle")
        self._lib = lib
        self._handle = handle

    # -- construction -------------------------------------------------------

    @classmethod
    def from_library(cls, lib: ctypes.CDLL, data: bytes) -> "Region":
        out = ctypes.c_void_p()
        buf = (ctypes.c_ubyte * len(data)).from_buffer_copy(data)
        rc = lib.tpt_uir_load(buf, ctypes.c_size_t(len(data)), ctypes.byref(out))
        if rc != 0:
            raise TptUirError("tpt_uir_load", rc)
        return cls(out, lib)

    @classmethod
    def load_bytes(cls, data: bytes) -> "Region":
        return cls.from_library(load_library(), data)

    @classmethod
    def load(cls, path: str) -> "Region":
        with open(path, "rb") as fh:
            return cls.load_bytes(fh.read())

    # -- lifecycle ----------------------------------------------------------

    def close(self) -> None:
        handle = getattr(self, "_handle", None)
        if handle:
            self._lib.tpt_uir_free(handle)
            self._handle = None

    def __enter__(self) -> "Region":
        return self

    def __exit__(self, *_exc) -> None:
        self.close()

    def __del__(self):
        try:
            self.close()
        except Exception:  # pragma: no cover - interpreter shutdown
            pass

    # -- accessors ----------------------------------------------------------

    def block_count(self) -> int:
        out = ctypes.c_size_t()
        rc = self._lib.tpt_uir_block_count(self._handle, ctypes.byref(out))
        if rc != 0:
            raise TptUirError("tpt_uir_block_count", rc)
        return out.value

    def op_count(self, block_index: int = 0) -> int:
        out = ctypes.c_size_t()
        rc = self._lib.tpt_uir_op_count(
            self._handle, ctypes.c_size_t(block_index), ctypes.byref(out)
        )
        if rc != 0:
            raise TptUirError("tpt_uir_op_count", rc)
        return out.value

    def op_name(self, block_index: int = 0, op_index: int = 0) -> str:
        dialect = ctypes.c_char_p()
        op = ctypes.c_char_p()
        b, o = ctypes.c_size_t(block_index), ctypes.c_size_t(op_index)
        rc1 = self._lib.tpt_uir_op_dialect(self._handle, b, o, ctypes.byref(dialect))
        if rc1 != 0:
            raise TptUirError("tpt_uir_op_dialect", rc1)
        rc2 = self._lib.tpt_uir_op_op(self._handle, b, o, ctypes.byref(op))
        if rc2 != 0:
            raise TptUirError("tpt_uir_op_op", rc2)
        return f"{dialect.value.decode()}.{op.value.decode()}"

    def operand(self, op_index: int, operand_index: int, block_index: int = 0) -> int:
        out = ctypes.c_uint32()
        rc = self._lib.tpt_uir_op_operand(
            self._handle,
            ctypes.c_size_t(block_index),
            ctypes.c_size_t(op_index),
            ctypes.c_size_t(operand_index),
            ctypes.byref(out),
        )
        if rc != 0:
            raise TptUirError("tpt_uir_op_operand", rc)
        return out.value

    def operand_count(self, op_index: int, block_index: int = 0) -> int:
        out = ctypes.c_size_t()
        rc = self._lib.tpt_uir_op_operand_count(
            self._handle,
            ctypes.c_size_t(block_index),
            ctypes.c_size_t(op_index),
            ctypes.byref(out),
        )
        if rc != 0:
            raise TptUirError("tpt_uir_op_operand_count", rc)
        return out.value

    def validate_ssa(self) -> bool:
        """Return True when the region passes SSA validation."""
        return self._lib.tpt_uir_validate_ssa(self._handle) == 0

    def validate_dialect(self, prefix: str) -> bool:
        """Return True when every op's dialect starts with `prefix`."""
        encoded = prefix.encode() + b"\x00"
        buf = (ctypes.c_char * len(encoded)).from_buffer_copy(encoded)
        return self._lib.tpt_uir_validate_dialect(self._handle, buf) == 0
