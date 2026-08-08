# Fast Token-Cursor Parser Strategy

## 1. Zero-Allocation Token Cursor Architecture
The parser accepts a flat slice of 12-byte tokens `&[Token]` produced by the lexer:

```rust
pub struct Parser<'a> {
    tokens: &'a [Token],
    src: &'a str,
    pos: u32,
}
```

- **Cursor Navigation**: `peek()` reads `self.tokens[self.pos]`, `bump()` increments `self.pos`.
- **Zero Copy**: No string cloning or token allocations during parsing.
- **Copy Semantics**: `Token` implements `Copy` (12 bytes), allowing register passing.

## 2. Parsing Strategy & Error Recovery
- Direct recursive descent operating over token indices.
- Immediate allocation into contiguous `AstArena` vectors.
- Resilient recovery: Inserts dummy `NodeId::DUMMY` nodes on parse errors to allow binder and checker to continue analyzing valid sections.

## 3. Fast Path Dispatches
- Specialized parser routines for high-frequency constructs (`parse_variable_declaration`, `parse_binary_expression`, `parse_primary_expression`).
- Trivia skipping toggled via `skip_trivia: true` during production compilation.
