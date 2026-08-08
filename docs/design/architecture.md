# Compiler Architecture & Pipeline Strategy

## 1. High-Level Design Goal
`rtsc` is designed from first principles as an ultra-fast, multi-threaded TypeScript compiler in Rust targeting **0.667x wall-clock time** (or faster) compared to Go-based compilers and standard Node-based `tsc`.

```
               ┌───────────────┐
               │ Source Bytes  │
               └───────┬───────┘
                       │ Zero-Copy Lexer (0 alloc, 12-byte Tokens)
                       ▼
                 Vec<Token>
                       │
                       ▼ Token Cursor Parser
               ┌───────────────┐
               │ Indexed Arena │
               │   Hot / Cold  │
               └───────┬───────┘
                       │ Compact NodeId / NameId / SymbolId / TypeId
            ┌──────────┼──────────┐
            ▼          ▼          ▼
         Symbols     Scopes    Types
            │          │          │
            └──────────┼──────────┘
                       ▼
               Parallel Checker
                       │
                       ▼
                 Emitter Engine
```

## 2. Core Pillars of Execution Speed
1. **Zero-Allocation Hot Paths**: Zero heap allocations in lexing and arena-indexed AST construction.
2. **Compact Integer Offsets & Struct Sizes**:
   - `Token`: 12 bytes (`#[repr(C)]` with `kind: u8`, `start: u32`, `end: u32`).
   - `Span`: 8 bytes (`start: u32`, `end: u32`).
   - `NodeId`, `SymbolId`, `TypeId`, `NameId`: 4-byte `u32` newtypes.
3. **Data-Oriented Memory Layout**: Hot AST fields (`kind`, child IDs, flags) separated from Cold AST fields (`Span`, trivia offsets).
4. **Canonical Type Caching**: Global and thread-local canonical `TypeId` structures with relation memoization (`RelationKey`).
5. **Differential Fuzzing**: Continuous lexical and syntactic differential testing against TSC reference streams.

## 3. Benchmarking Framework & Hardware Metrics
Performance is evaluated across MB/s throughput, tokens/sec, and hardware performance counters:
- **Throughput Metrics**: MB/sec, tokens/sec, total allocations, bytes allocated.
- **Hardware Counter Audits**: IPC, L1/L2/LLC cache misses, branch mispredictions (`perf` / Linux, Criterion on Windows/Linux).
- **Target Workloads**: React, TypeScript compiler, Angular, Next.js, VS Code, Node, Deno.
