# Data-Oriented AST & Arena Strategy

## 1. Traditional AST Pitfalls
Traditional compiler ASTs use recursive pointer nodes:
```rust
// AVOID: Memory fragmentation and cache miss penalty
struct Node {
    kind: NodeKind,
    children: Vec<Box<Node>>,
}
```
This causes heavy allocation pressure, heap fragmentation, and CPU cache misses.

## 2. Indexed Arena Design
`rtsc` uses a flat, index-backed AST representation (`AstArena`):
- `NodeId(u32)`: Compact index reference into flat vector storage.
- Eliminates heap allocation per AST node.
- Cache-friendly array traversal during tree walking.

## 3. Hot vs Cold Separation
AST nodes are split into two parallel vectors in `AstArena`:
- **`HotNode`**: Inspected constantly during binding and type checking (`kind`, `child_a`, `child_b`, `name: NameId`, `flags`).
- **`ColdNode`**: Accessed only during diagnostic formatting or source emission (`span: Span`).

```rust
pub struct AstArena {
    hot_nodes: Vec<HotNode>,
    cold_nodes: Vec<ColdNode>,
}
```

## 4. Node Compaction & Integer References
All references (expressions, statements, identifiers) use compact 32-bit IDs:
- `ExprId(u32)`
- `StmtId(u32)`
- `NameId(u32)`
- `SymbolId(u32)`
- `TypeId(u32)`
