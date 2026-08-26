/*
 * basic.c — minimal C consumer of the tpt-uir-ffi C ABI.
 *
 * Loads a postcard-encoded region, inspects it through the API declared in
 * `../include/tpt_uir_ffi.h`, and frees the handle. The encoded bytes below are
 * the exact `tpt_uir_serde::serialize_region` output for a region containing a
 * single `tpt_gpu.launch` operation over two Index block arguments (the same
 * fixture used by the Rust FFI tests).
 *
 * Build (after `cargo build -p tpt-uir-ffi --release`):
 *
 *   cc -I../include basic.c -o basic \
 *       -L../../../target/release -ltpt_uir_ffi -Wl,-rpath,../../../target/release
 */
#include <stdio.h>
#include <stdint.h>
#include "tpt_uir_ffi.h"

int main(void) {
    static const uint8_t bytes[] = {
        0x01, 0x02, 0x00, 0x02, 0x01, 0x02, 0x01, 0x01, 0x07,
        't',  'p',  't',  '_',  'g',  'p',  'u',  0x06,
        'l',  'a',  'u',  'n',  'c',  'h',  0x02, 0x00,
        0x01, 0x00, 0x00, 0x00,
    };

    void *handle = NULL;
    int rc = tpt_uir_load(bytes, sizeof(bytes), &handle);
    if (rc != 0) {
        fprintf(stderr, "tpt_uir_load failed with code %d\n", rc);
        return 1;
    }

    size_t blocks = 0;
    if (tpt_uir_block_count(handle, &blocks) != 0 || blocks != 1) {
        fprintf(stderr, "unexpected block count\n");
        tpt_uir_free(handle);
        return 1;
    }

    size_t ops = 0;
    if (tpt_uir_op_count(handle, 0, &ops) != 0 || ops != 1) {
        fprintf(stderr, "unexpected op count\n");
        tpt_uir_free(handle);
        return 1;
    }

    const char *dialect = NULL;
    const char *op = NULL;
    if (tpt_uir_op_dialect(handle, 0, 0, &dialect) != 0 ||
        tpt_uir_op_op(handle, 0, 0, &op) != 0) {
        fprintf(stderr, "failed to read op name\n");
        tpt_uir_free(handle);
        return 1;
    }
    printf("op: %s.%s\n", dialect, op);

    uint32_t operand = 99;
    if (tpt_uir_op_operand(handle, 0, 0, 1, &operand) != 0 || operand != 1) {
        fprintf(stderr, "failed to read operand 1\n");
        tpt_uir_free(handle);
        return 1;
    }
    printf("operand[1] = %u\n", operand);

    if (tpt_uir_validate_ssa(handle) != 0) {
        fprintf(stderr, "SSA validation failed\n");
        tpt_uir_free(handle);
        return 1;
    }
    printf("SSA valid\n");

    tpt_uir_free(handle);
    return 0;
}

