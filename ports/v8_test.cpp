// Bat_OS — V8 Interpreter Infrastructure Test
extern "C" {
    #include <stdio.h>
    #include <stdlib.h>
}

#include "src/interpreter/bytecodes.h"
#include "src/interpreter/bytecode-operands.h"

namespace interp = v8::internal::interpreter;

extern "C" void _start() {
    printf("=== V8 Interpreter Test ===\n");
    printf("Chromium's JS engine on Bat_OS\n\n");

    int passed = 0;

    // [1] Bytecode definitions exist
    int total = static_cast<int>(interp::Bytecode::kLast) + 1;
    printf("[1] Total bytecodes: %d\n", total);
    if (total > 100) { printf("    PASS\n"); passed++; }

    // [2] Bytecode names
    const char* name = interp::Bytecodes::ToString(interp::Bytecode::kAdd);
    printf("[2] Add bytecode name: %s\n", name ? name : "(null)");
    if (name) { printf("    PASS\n"); passed++; }

    // [3] Bytecode sizes
    int add_size = interp::Bytecodes::Size(interp::Bytecode::kAdd, interp::OperandScale::kSingle);
    int ret_size = interp::Bytecodes::Size(interp::Bytecode::kReturn, interp::OperandScale::kSingle);
    printf("[3] Add size=%d, Return size=%d\n", add_size, ret_size);
    if (add_size > 0 && ret_size > 0) { printf("    PASS\n"); passed++; }

    // [4] Operand count
    int nops = interp::Bytecodes::NumberOfOperands(interp::Bytecode::kLdaSmi);
    printf("[4] LdaSmi operands: %d\n", nops);
    if (nops > 0) { printf("    PASS\n"); passed++; }

    // [5] Accumulator usage
    bool reads = interp::Bytecodes::ReadsAccumulator(interp::Bytecode::kAdd);
    bool writes = interp::Bytecodes::WritesAccumulator(interp::Bytecode::kAdd);
    printf("[5] Add: reads_acc=%d writes_acc=%d\n", reads, writes);
    if (reads || writes) { printf("    PASS\n"); passed++; }

    printf("\n=== V8 Test: %d/5 passed ===\n", passed);
    if (passed >= 4) printf("=== V8 PASSED ===\n");
    exit(0);
}
