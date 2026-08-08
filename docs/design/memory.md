# Memory Optimization & Alignment Architecture

## 1. Zero-Allocation Scanner Rules
- Lexing a source buffer must perform **0 heap allocations**.
- Tokens are returned as a flat 12-byte `#[repr(C)]` struct:
```rust
#[repr(C)]
pub struct Token {
    pub kind: TokenKind, // 1 byte (u8)
                         // 3 bytes alignment padding
    pub start: u32,      // 4 bytes
    pub end: u32,        // 4 bytes
}
```
- `size_of::<Token>()` assertion enforced by test suite.

## 2. String Interning (`StringInterner`)
- Avoids repeated allocations of identical string identifiers (`foo`, `bar`, `props`, `default`).
- Maps `&str` -> `NameId(u32)`.
- Comparisons convert from $O(N)$ string byte comparisons to $O(1)$ `u32` equality checks.

## 3. AoS vs SoA Benchmarking Decision
- **Array of Structures (AoS)**: `Vec<Token>` provides optimal cache line density when parser operations consume `kind`, `start`, and `end` together.
- **Structure of Arrays (SoA)**: Tested for pure kind-filtering passes; AoS selected as default based on profile evidence.
