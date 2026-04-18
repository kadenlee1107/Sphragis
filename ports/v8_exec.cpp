// Bat_OS — V8 Bytecode Execution Loop
// Software interpreter for V8 Ignition bytecodes on bare-metal ARM64.
//
// Architecture:
//   - Register-accumulator machine (matches V8 Ignition)
//   - 32 general-purpose registers + accumulator
//   - Bytecodes dispatched via switch/case loop
//   - Simplified Smi values (64-bit signed integers)
//
// Supported bytecodes:
//   Loading:    LdaZero, LdaSmi, LdaUndefined, LdaTrue, LdaFalse
//   Registers:  Ldar, Star, Star0-Star15, Mov
//   Arithmetic: Add, Sub, Mul, Div, Mod, AddSmi, SubSmi
//   Unary:      Negate, Inc, Dec
//   Comparison: TestEqual, TestLessThan, TestGreaterThan,
//               TestLessThanOrEqual, TestGreaterThanOrEqual
//   Jumps:      Jump, JumpIfTrue, JumpIfFalse, JumpLoop
//   Control:    Return

extern "C" {
    #include <stdio.h>
    #include <stdlib.h>
}

#include "src/interpreter/bytecodes.h"
#include "src/interpreter/bytecode-operands.h"

namespace bc = v8::internal::interpreter;
using Bytecode = bc::Bytecode;
using OperandScale = bc::OperandScale;

// Bytecode → raw byte for constructing bytecode arrays
static constexpr uint8_t B(Bytecode b) { return static_cast<uint8_t>(b); }

// ════════════════════════════════════════════════════════════════
// Value representation — simplified Smi (Small Integer)
// Real V8 uses tagged pointers; we use plain 64-bit integers.
// ════════════════════════════════════════════════════════════════
using Value = long long;
static constexpr Value kTrue  = 1;
static constexpr Value kFalse = 0;

// ════════════════════════════════════════════════════════════════
// Bytecode Interpreter — V8 Ignition on bare metal
// ════════════════════════════════════════════════════════════════
static constexpr int kMaxRegs = 32;

struct Interpreter {
    Value acc;                  // accumulator register
    Value regs[kMaxRegs];       // general-purpose register file
    const uint8_t* code;        // bytecode array
    int pc;                     // program counter (byte offset)
    int len;                    // bytecode array length
    bool halted;                // execution stopped
    int steps;                  // instructions executed (profiling)

    void init(const uint8_t* bytecode, int length) {
        acc = 0;
        for (int i = 0; i < kMaxRegs; i++) regs[i] = 0;
        code = bytecode;
        pc = 0;
        len = length;
        halted = false;
        steps = 0;
    }

    Value execute() {
        while (!halted && pc < len) {
            int ip = pc;                              // instruction start
            Bytecode op = static_cast<Bytecode>(code[pc++]);
            steps++;

            switch (op) {

            // ── Loading literals ──────────────────────────────
            case Bytecode::kLdaZero:
                acc = 0;
                break;
            case Bytecode::kLdaSmi:
                acc = static_cast<int8_t>(code[pc++]); // signed immediate
                break;
            case Bytecode::kLdaUndefined:
                acc = 0;
                break;
            case Bytecode::kLdaTrue:
                acc = kTrue;
                break;
            case Bytecode::kLdaFalse:
                acc = kFalse;
                break;

            // ── Register load / store ─────────────────────────
            case Bytecode::kLdar:
                acc = regs[code[pc++] & 31];
                break;
            case Bytecode::kStar:
                regs[code[pc++] & 31] = acc;
                break;
            case Bytecode::kMov: {
                uint8_t src = code[pc++] & 31;
                uint8_t dst = code[pc++] & 31;
                regs[dst] = regs[src];
                break;
            }

            // ── Short-star: single-byte store to r0–r15 ──────
            case Bytecode::kStar0:  regs[0]  = acc; break;
            case Bytecode::kStar1:  regs[1]  = acc; break;
            case Bytecode::kStar2:  regs[2]  = acc; break;
            case Bytecode::kStar3:  regs[3]  = acc; break;
            case Bytecode::kStar4:  regs[4]  = acc; break;
            case Bytecode::kStar5:  regs[5]  = acc; break;
            case Bytecode::kStar6:  regs[6]  = acc; break;
            case Bytecode::kStar7:  regs[7]  = acc; break;
            case Bytecode::kStar8:  regs[8]  = acc; break;
            case Bytecode::kStar9:  regs[9]  = acc; break;
            case Bytecode::kStar10: regs[10] = acc; break;
            case Bytecode::kStar11: regs[11] = acc; break;
            case Bytecode::kStar12: regs[12] = acc; break;
            case Bytecode::kStar13: regs[13] = acc; break;
            case Bytecode::kStar14: regs[14] = acc; break;
            case Bytecode::kStar15: regs[15] = acc; break;

            // ── Binary arithmetic: acc = reg[R] OP acc ────────
            // Each has: reg operand (1 byte) + feedback slot (1 byte, ignored)
            case Bytecode::kAdd: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = regs[r] + acc;
                break;
            }
            case Bytecode::kSub: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = regs[r] - acc;
                break;
            }
            case Bytecode::kMul: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = regs[r] * acc;
                break;
            }
            case Bytecode::kDiv: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = (acc != 0) ? regs[r] / acc : 0;
                break;
            }
            case Bytecode::kMod: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = (acc != 0) ? regs[r] % acc : 0;
                break;
            }

            // ── Smi immediate arithmetic: acc OP= imm ────────
            // Each has: signed immediate (1 byte) + feedback slot (1 byte)
            case Bytecode::kAddSmi: {
                int8_t v = static_cast<int8_t>(code[pc++]); pc++;
                acc += v;
                break;
            }
            case Bytecode::kSubSmi: {
                int8_t v = static_cast<int8_t>(code[pc++]); pc++;
                acc -= v;
                break;
            }

            // ── Unary operations ──────────────────────────────
            // Each has: feedback slot (1 byte)
            case Bytecode::kNegate: pc++; acc = -acc; break;
            case Bytecode::kInc:    pc++; acc++;      break;
            case Bytecode::kDec:    pc++; acc--;      break;

            // ── Comparisons: acc = (reg[R] CMP acc) → bool ───
            // Each has: reg operand (1 byte) + feedback slot (1 byte)
            case Bytecode::kTestEqual: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = (regs[r] == acc) ? kTrue : kFalse;
                break;
            }
            case Bytecode::kTestLessThan: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = (regs[r] < acc) ? kTrue : kFalse;
                break;
            }
            case Bytecode::kTestGreaterThan: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = (regs[r] > acc) ? kTrue : kFalse;
                break;
            }
            case Bytecode::kTestLessThanOrEqual: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = (regs[r] <= acc) ? kTrue : kFalse;
                break;
            }
            case Bytecode::kTestGreaterThanOrEqual: {
                uint8_t r = code[pc++] & 31; pc++;
                acc = (regs[r] >= acc) ? kTrue : kFalse;
                break;
            }

            // ── Jumps ─────────────────────────────────────────
            // Forward jumps: offset from instruction start
            case Bytecode::kJump: {
                uint8_t off = code[pc++];
                pc = ip + off;    // absolute target = instruction_start + offset
                break;
            }
            case Bytecode::kJumpIfTrue: {
                uint8_t off = code[pc++];
                if (acc == kTrue) pc = ip + off;
                break;
            }
            case Bytecode::kJumpIfFalse: {
                uint8_t off = code[pc++];
                if (acc == kFalse) pc = ip + off;
                break;
            }
            // Backward jump (loop): offset subtracted from instruction start
            // Operands: back_offset (UImm), interrupt_budget (Imm), osr (UImm)
            case Bytecode::kJumpLoop: {
                uint8_t back = code[pc++];
                // skip interrupt_budget + osr_urgency (not needed for execution)
                pc = ip - back;   // jump backward
                break;
            }

            // ── Control ───────────────────────────────────────
            case Bytecode::kReturn:
                halted = true;
                break;

            // ── Fallthrough for unhandled bytecodes ───────────
            default:
                printf("    HALT: unknown bytecode 0x%02x at offset %d\n",
                       static_cast<unsigned>(B(op)), ip);
                halted = true;
                break;
            }
        }
        return acc;
    }
};

// ════════════════════════════════════════════════════════════════
// Disassembler — uses V8's Bytecodes API for names + sizes
// ════════════════════════════════════════════════════════════════
static void disasm(const uint8_t* code, int len) {
    int pc = 0;
    while (pc < len) {
        Bytecode op = static_cast<Bytecode>(code[pc]);
        const char* name = bc::Bytecodes::ToString(op);
        int size = bc::Bytecodes::Size(op, OperandScale::kSingle);
        if (size <= 0) size = 1;

        printf("    %3d: %-24s", pc, name ? name : "???");

        // Print operand bytes
        for (int i = 1; i < size && pc + i < len; i++) {
            if (i > 1) printf(", ");
            printf("0x%02x", code[pc + i]);
        }
        printf("\n");

        pc += size;
    }
}

// ════════════════════════════════════════════════════════════════
// Test runner
// ════════════════════════════════════════════════════════════════
static int run_test(int num, const char* title, const char* js,
                    const uint8_t* code, int len, Value expected) {
    printf("[%d] %s\n", num, title);
    printf("    JS: %s\n", js);

    disasm(code, len);

    Interpreter vm;
    vm.init(code, len);
    Value result = vm.execute();

    bool pass = (result == expected);
    printf("    => %lld (expected %lld) %s [%d steps]\n\n",
           result, expected, pass ? "PASS" : "FAIL", vm.steps);
    return pass ? 1 : 0;
}

// ════════════════════════════════════════════════════════════════
// JavaScript Expression Compiler
// Recursive descent parser → V8 bytecodes
//
// Supported syntax:
//   - Integer literals: 0, 42, -5
//   - Arithmetic: +, -, *, /, %
//   - Comparison: ==, !=, <, >, <=, >=
//   - Parentheses: (expr)
//   - Unary minus: -expr
//
// Grammar:
//   expr       = comparison
//   comparison = addition (('=='|'!='|'<'|'>'|'<='|'>=') addition)?
//   addition   = multiply (('+' | '-') multiply)*
//   multiply   = unary (('*' | '/' | '%') unary)*
//   unary      = '-' unary | primary
//   primary    = NUMBER | '(' expr ')'
// ════════════════════════════════════════════════════════════════

struct Compiler {
    const char* src;
    int pos;
    int src_len;
    uint8_t out[256];   // output bytecode buffer
    int out_len;
    int next_reg;       // register allocator
    bool error;

    void init(const char* js) {
        src = js;
        pos = 0;
        src_len = 0;
        while (js[src_len]) src_len++;
        out_len = 0;
        next_reg = 0;
        error = false;
    }

    // ── Bytecode emitters ──
    void emit(uint8_t b) { if (out_len < 256) out[out_len++] = b; }
    void emit2(uint8_t b1, uint8_t b2) { emit(b1); emit(b2); }
    void emit3(uint8_t b1, uint8_t b2, uint8_t b3) { emit(b1); emit(b2); emit(b3); }

    int alloc_reg() { return next_reg++ & 31; }
    void free_reg() { if (next_reg > 0) next_reg--; }

    void emit_star(int r) {
        if (r <= 15) {
            // Short-star: enum goes Star15(lowest)...Star0(highest)
            // kStarN = kStar0 - N
            emit(static_cast<uint8_t>(static_cast<int>(Bytecode::kStar0) - r));
        } else {
            emit2(B(Bytecode::kStar), (uint8_t)r);
        }
    }

    // ── Lexer helpers ──
    void skip_ws() {
        while (pos < src_len && (src[pos] == ' ' || src[pos] == '\t')) pos++;
    }

    bool peek(char c) { skip_ws(); return pos < src_len && src[pos] == c; }
    bool match(char c) { if (peek(c)) { pos++; return true; } return false; }

    bool peek2(char a, char b) {
        skip_ws();
        return pos + 1 < src_len && src[pos] == a && src[pos+1] == b;
    }
    bool match2(char a, char b) {
        if (peek2(a, b)) { pos += 2; return true; }
        return false;
    }

    int parse_number() {
        skip_ws();
        int sign = 1;
        if (pos < src_len && src[pos] == '-') { sign = -1; pos++; }
        int val = 0;
        while (pos < src_len && src[pos] >= '0' && src[pos] <= '9') {
            val = val * 10 + (src[pos] - '0');
            pos++;
        }
        return sign * val;
    }

    // ── Recursive descent parser + code generator ──

    // primary = NUMBER | '(' expr ')'
    void compile_primary() {
        skip_ws();
        if (error || pos >= src_len) return;

        if (src[pos] >= '0' && src[pos] <= '9') {
            int val = parse_number();
            if (val == 0) {
                emit(B(Bytecode::kLdaZero));
            } else if (val >= -128 && val <= 127) {
                emit2(B(Bytecode::kLdaSmi), (uint8_t)(int8_t)val);
            } else {
                // Large number: decompose as multiply+add
                emit2(B(Bytecode::kLdaSmi), (uint8_t)(int8_t)(val / 128));
                int r = alloc_reg();
                emit_star(r);
                emit2(B(Bytecode::kLdaSmi), 127);
                emit3(B(Bytecode::kMul), (uint8_t)r, 0);
                emit_star(r);
                int rem = val - (val / 128) * 127;
                emit2(B(Bytecode::kLdaSmi), (uint8_t)(int8_t)rem);
                emit3(B(Bytecode::kAdd), (uint8_t)r, 0);
                free_reg();
            }
        } else if (match('(')) {
            compile_expr();
            if (!match(')')) error = true;
        } else {
            error = true;
        }
    }

    // unary = '-' unary | primary
    void compile_unary() {
        skip_ws();
        if (pos < src_len && src[pos] == '-' &&
            (pos + 1 >= src_len || src[pos+1] < '0' || src[pos+1] > '9')) {
            pos++;
            compile_unary();
            emit2(B(Bytecode::kNegate), 0);
        } else if (pos < src_len && src[pos] == '-' &&
                   pos + 1 < src_len && src[pos+1] >= '0') {
            // Negative number literal
            int val = parse_number();
            if (val == 0) emit(B(Bytecode::kLdaZero));
            else emit2(B(Bytecode::kLdaSmi), (uint8_t)(int8_t)val);
        } else {
            compile_primary();
        }
    }

    // multiply = unary (('*' | '/' | '%') unary)*
    void compile_multiply() {
        compile_unary();
        for (;;) {
            skip_ws();
            char op = 0;
            if (pos < src_len) {
                if (src[pos] == '*') op = '*';
                else if (src[pos] == '/') op = '/';
                else if (src[pos] == '%') op = '%';
            }
            if (!op) break;
            pos++;

            int r = alloc_reg();
            emit_star(r);           // save left operand
            compile_unary();        // right operand → acc

            switch (op) {
                case '*': emit3(B(Bytecode::kMul), (uint8_t)r, 0); break;
                case '/': emit3(B(Bytecode::kDiv), (uint8_t)r, 0); break;
                case '%': emit3(B(Bytecode::kMod), (uint8_t)r, 0); break;
            }
            free_reg();
        }
    }

    // addition = multiply (('+' | '-') multiply)*
    void compile_addition() {
        compile_multiply();
        for (;;) {
            skip_ws();
            char op = 0;
            if (pos < src_len) {
                // Don't match == or <= or >= as addition
                if (src[pos] == '+') op = '+';
                else if (src[pos] == '-' && (pos+1 >= src_len || src[pos+1] != '-')) op = '-';
            }
            if (!op) break;
            pos++;

            int r = alloc_reg();
            emit_star(r);
            compile_multiply();

            switch (op) {
                case '+': emit3(B(Bytecode::kAdd), (uint8_t)r, 0); break;
                case '-': emit3(B(Bytecode::kSub), (uint8_t)r, 0); break;
            }
            free_reg();
        }
    }

    // comparison = addition (('=='|'!='|'<'|'>'|'<='|'>=') addition)?
    void compile_comparison() {
        compile_addition();
        skip_ws();
        if (pos >= src_len) return;

        Bytecode cmp_op = Bytecode::kIllegal;
        bool negate = false;

        if (match2('=', '='))      cmp_op = Bytecode::kTestEqual;
        else if (match2('!', '=')) { cmp_op = Bytecode::kTestEqual; negate = true; }
        else if (match2('<', '=')) cmp_op = Bytecode::kTestLessThanOrEqual;
        else if (match2('>', '=')) cmp_op = Bytecode::kTestGreaterThanOrEqual;
        else if (match('<'))      cmp_op = Bytecode::kTestLessThan;
        else if (match('>'))      cmp_op = Bytecode::kTestGreaterThan;

        if (cmp_op == Bytecode::kIllegal) return;

        int r = alloc_reg();
        emit_star(r);       // save left operand
        compile_addition(); // right operand → acc
        emit3(B(cmp_op), (uint8_t)r, 0);

        if (negate) {
            // Flip true/false for !=
            // TestEqual gives 1 for equal; we want 1 for not-equal
            // acc = 1 - acc (XOR with 1)
            int r2 = alloc_reg();
            emit_star(r2);
            emit2(B(Bytecode::kLdaSmi), 1);
            emit3(B(Bytecode::kSub), (uint8_t)r2, 0);
            free_reg();
        }
        free_reg();
    }

    // Top-level: compile an expression + Return
    void compile_expr() { compile_comparison(); }

    bool compile(const char* js) {
        init(js);
        compile_expr();
        emit(B(Bytecode::kReturn));
        return !error && out_len > 0;
    }
};

// Run a JavaScript expression through the compiler + executor
static int eval_js(int num, const char* js, Value expected) {
    printf("[%d] Live JS Compilation\n", num);
    printf("    JS: %s\n", js);

    Compiler comp;
    if (!comp.compile(js)) {
        printf("    COMPILE ERROR\n\n");
        return 0;
    }

    printf("    Compiled %d bytes of V8 bytecodes:\n", comp.out_len);
    disasm(comp.out, comp.out_len);

    Interpreter vm;
    vm.init(comp.out, comp.out_len);
    Value result = vm.execute();

    bool pass = (result == expected);
    printf("    => %lld (expected %lld) %s [%d steps]\n\n",
           result, expected, pass ? "PASS" : "FAIL", vm.steps);
    return pass ? 1 : 0;
}

// ════════════════════════════════════════════════════════════════
// Entry point — 6 hand-coded tests + 4 live-compiled JS tests
// ════════════════════════════════════════════════════════════════
extern "C" void _start() {
    printf("\n=== Bat_OS V8 Bytecode Executor ===\n");
    printf("V8 Ignition interpreter on bare-metal ARM64\n\n");

    int passed = 0;

    // ── [1] Simple Addition: 1 + 2 = 3 ──────────────────────
    {
        const uint8_t code[] = {
            B(Bytecode::kLdaSmi), 1,            //  0: acc = 1
            B(Bytecode::kStar0),                //  2: r0 = 1
            B(Bytecode::kLdaSmi), 2,            //  3: acc = 2
            B(Bytecode::kAdd), 0, 0,            //  5: acc = r0 + acc = 3
            B(Bytecode::kReturn),               //  8: return 3
        };
        passed += run_test(1, "Simple Addition", "1 + 2",
                          code, sizeof(code), 3);
    }

    // ── [2] Complex Expression: (3 + 4) * 2 = 14 ────────────
    {
        const uint8_t code[] = {
            B(Bytecode::kLdaSmi), 3,            //  0: acc = 3
            B(Bytecode::kStar0),                //  2: r0 = 3
            B(Bytecode::kLdaSmi), 4,            //  3: acc = 4
            B(Bytecode::kAdd), 0, 0,            //  5: acc = 3 + 4 = 7
            B(Bytecode::kStar0),                //  8: r0 = 7
            B(Bytecode::kLdaSmi), 2,            //  9: acc = 2
            B(Bytecode::kMul), 0, 0,            // 11: acc = 7 * 2 = 14
            B(Bytecode::kReturn),               // 14: return 14
        };
        passed += run_test(2, "Complex Expression", "(3 + 4) * 2",
                          code, sizeof(code), 14);
    }

    // ── [3] Comparison: 5 > 3 = true ─────────────────────────
    {
        const uint8_t code[] = {
            B(Bytecode::kLdaSmi), 5,            //  0: acc = 5
            B(Bytecode::kStar0),                //  2: r0 = 5
            B(Bytecode::kLdaSmi), 3,            //  3: acc = 3
            B(Bytecode::kTestGreaterThan), 0, 0,//  5: acc = (r0 > acc) = (5>3)
            B(Bytecode::kReturn),               //  8: return true (1)
        };
        passed += run_test(3, "Comparison", "5 > 3",
                          code, sizeof(code), kTrue);
    }

    // ── [4] Conditional: if (10 > 5) 42 else -1 ─────────────
    {
        const uint8_t code[] = {
            B(Bytecode::kLdaSmi), 10,             //  0: acc = 10
            B(Bytecode::kStar0),                  //  2: r0 = 10
            B(Bytecode::kLdaSmi), 5,              //  3: acc = 5
            B(Bytecode::kTestGreaterThan), 0, 0,  //  5: acc = (10 > 5) = true
            B(Bytecode::kJumpIfFalse), 6,         //  8: if false -> 14
            B(Bytecode::kLdaSmi), 42,             // 10: acc = 42
            B(Bytecode::kJump), 4,                // 12: -> 16 (skip else)
            B(Bytecode::kLdaSmi), 0xFF,           // 14: acc = -1  (else branch)
            B(Bytecode::kReturn),                 // 16: return
        };
        passed += run_test(4, "Conditional", "if (10 > 5) 42 else -1",
                          code, sizeof(code), 42);
    }

    // ── [5] Factorial Loop: 10! = 3628800 ────────────────────
    //
    // JavaScript equivalent:
    //   let result = 1, i = 10;
    //   while (i > 0) { result = result * i; i = i - 1; }
    //   return result;
    //
    {
        const uint8_t code[] = {
            B(Bytecode::kLdaSmi), 1,                //  0: acc = 1
            B(Bytecode::kStar0),                    //  2: r0 = result = 1
            B(Bytecode::kLdaSmi), 10,               //  3: acc = 10
            B(Bytecode::kStar1),                    //  5: r1 = i = 10
            // loop start (offset 6):
            B(Bytecode::kLdaZero),                  //  6: acc = 0
            B(Bytecode::kTestGreaterThan), 1, 0,    //  7: acc = (r1 > 0)
            B(Bytecode::kJumpIfFalse), 18,          // 10: if false -> 28
            B(Bytecode::kLdar), 0,                  // 12: acc = result
            B(Bytecode::kMul), 1, 0,                // 14: acc = result * i
            B(Bytecode::kStar0),                    // 17: r0 = new result
            B(Bytecode::kLdar), 1,                  // 18: acc = i
            B(Bytecode::kSubSmi), 1, 0,             // 20: acc = i - 1
            B(Bytecode::kStar1),                    // 23: r1 = new i
            B(Bytecode::kJumpLoop), 18, 0, 0,       // 24: -> offset 6
            // after loop (offset 28):
            B(Bytecode::kLdar), 0,                  // 28: acc = result
            B(Bytecode::kReturn),                   // 30: return 3628800
        };
        passed += run_test(5, "Factorial Loop", "10!",
                          code, sizeof(code), 3628800LL);
    }

    // ── [6] Fibonacci Loop: fib(10) = 55 ─────────────────────
    //
    // JavaScript equivalent:
    //   let a = 0, b = 1, count = 10;
    //   while (count > 0) {
    //     let temp = a + b;
    //     a = b; b = temp; count--;
    //   }
    //   return a;
    //
    {
        const uint8_t code[] = {
            B(Bytecode::kLdaZero),                  //  0: acc = 0
            B(Bytecode::kStar0),                    //  1: r0 = a = 0
            B(Bytecode::kLdaSmi), 1,                //  2: acc = 1
            B(Bytecode::kStar1),                    //  4: r1 = b = 1
            B(Bytecode::kLdaSmi), 10,               //  5: acc = 10
            B(Bytecode::kStar2),                    //  7: r2 = count = 10
            // loop start (offset 8):
            B(Bytecode::kLdaZero),                  //  8: acc = 0
            B(Bytecode::kTestGreaterThan), 2, 0,    //  9: acc = (r2 > 0)
            B(Bytecode::kJumpIfFalse), 24,          // 12: if false -> 36
            B(Bytecode::kLdar), 0,                  // 14: acc = a
            B(Bytecode::kAdd), 1, 0,                // 16: acc = a + b
            B(Bytecode::kStar3),                    // 19: r3 = temp
            B(Bytecode::kMov), 1, 0,                // 20: r0 = r1 (a = b)
            B(Bytecode::kLdar), 3,                  // 23: acc = temp
            B(Bytecode::kStar1),                    // 25: r1 = temp (b = temp)
            B(Bytecode::kLdar), 2,                  // 26: acc = count
            B(Bytecode::kSubSmi), 1, 0,             // 28: acc = count - 1
            B(Bytecode::kStar2),                    // 31: r2 = new count
            B(Bytecode::kJumpLoop), 24, 0, 0,       // 32: -> offset 8
            // after loop (offset 36):
            B(Bytecode::kLdar), 0,                  // 36: acc = a
            B(Bytecode::kReturn),                   // 38: return 55
        };
        passed += run_test(6, "Fibonacci Loop", "fib(10)",
                          code, sizeof(code), 55LL);
    }

    // ════════════════════════════════════════════════════════════
    // Part 2: Live JavaScript Compilation
    // Parse JS expressions → compile to V8 bytecodes → execute
    // ════════════════════════════════════════════════════════════
    printf("--- Live JS Compiler ---\n\n");

    // [7] Simple expression compiled on the fly
    passed += eval_js(7, "2 + 3", 5);

    // [8] Operator precedence
    passed += eval_js(8, "2 + 3 * 4", 14);

    // [9] Parenthesized grouping
    passed += eval_js(9, "(2 + 3) * 4", 20);

    // [10] Comparison
    passed += eval_js(10, "100 > 50", kTrue);

    // ── Summary ──────────────────────────────────────────────
    printf("===================================\n");
    printf("V8 Executor: %d/10 passed\n", passed);
    if (passed >= 9) {
        printf("=== JAVASCRIPT RUNS ON BARE METAL ===\n");
    }
    printf("===================================\n");

    exit(0);
}
