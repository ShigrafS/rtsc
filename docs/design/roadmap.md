# Optimization & Feature Roadmap (Stages 0–15)

## Stage 0: Baseline & Micro-Benchmarking
- Establish Criterion benchmark suite measuring MB/s, tokens/sec, allocations, and wall-clock times on real-world projects (React, TypeScript, Angular, Next.js, VS Code, Node, Deno).

## Stage 1: Zero-Allocation Lexer & ASCII Fast Path
- Fast-path byte loop (`src[pos]`) for ASCII chars.
- 12-byte `#[repr(C)] Token` (`size_of::<Token>() == 12`).
- Table-driven `CharClass` classification array.
- Specialized scanners (`scan_identifier`, `scan_number`, `scan_string`, `scan_template`, `scan_slash`, `scan_comment`).

## Stage 2: Data-Oriented AST & Compact Arenas
- Replace pointer-heavy AST with flat `AstArena` using 32-bit `NodeId`, `ExprId`, `StmtId`.
- Separate `HotNode` (kind, child IDs, flags) and `ColdNode` (`Span`).

## Stage 3: String Interning & Symbol Binder
- `StringInterner` mapping `&str` -> `NameId(u32)` for $O(1)$ symbol lookups.
- Scope hierarchy tree and centralized `Vec<Symbol>` array.

## Stage 4: Type Checker & Canonical Type Caching
- `TypeId(u32)` canonical type table.
- Fast-path primitive assignability checks (`NUMBER`, `STRING`, `BOOLEAN`, `ANY`).
- Assignability memoization via `RelationKey` table.

## Stage 5: Continuous Differential Fuzzing
- Automated random JS generator harness.
- Differential testing vs TSC reference scanner to guarantee 100% lexical correctness.

## Stage 6: Code Generation & Emitter
- Streamlined JS/D.TS code generator operating directly over `AstArena`.

## Stage 7: Incremental Builds & SIMD Optimization
- Subtree invalidation, thread-local arenas, and SIMD (AVX2/NEON/memchr) vectorization after hardware counter audit.
