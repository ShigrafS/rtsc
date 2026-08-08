# Diagnostic System & Error Recovery Architecture

## 1. Zero Overhead on Hot Path
- Diagnostics generation is kept strictly off the hot parsing/checking path.
- Errors store compact `(Span, ErrorCode)` tuples.
- String formatting, line/column calculations, and snippet formatting occur only on the cold path when emitting errors.

## 2. Fast Binary Search Line/Column Table (`LineStarts`)
- Source file line start offsets are pre-computed into `Vec<u32>` (`LineStarts`).
- Byte offset -> (line, col) mapping uses fast $O(\log L)$ binary search.

```rust
pub struct LineStarts {
    starts: Vec<u32>,
}
```

## 3. Resilient AST Parsing & Binding
- Syntax errors produce error nodes (`NodeKind::Unknown`) in the arena.
- Compiler continues binder and type-checking phases on valid subtrees to provide comprehensive IDE diagnostics in a single pass.
