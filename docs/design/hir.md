# High-Level Intermediate Representation (HIR) Architecture

## 1. Role of HIR in Compiler Lifecycle
After parsing and binding, `rtsc` lowers the surface AST into a High-Level Intermediate Representation (HIR):

```
Surface AST (Lossless) ──> HIR Lowering ──> Type Checker & Control Flow
```

## 2. Key Differences from Surface AST
- **Desugared Constructs**: Class fields, decorators, async/await transforms lowered to standardized canonical HIR primitives.
- **Flattened Expressions**: Implicit conversions and default parameter initializers explicitly represented as explicit control nodes.
- **Symbol & Scope Bindings Baked In**: HIR nodes directly store `SymbolId` and `ScopeId` instead of identifier strings.

## 3. High Performance Lowering
- Lowering operates entirely within flat arena vectors (`HirArena`).
- Enables parallel control-flow analysis and dataflow checking per function body.
