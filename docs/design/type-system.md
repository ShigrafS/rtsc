# Type System & Assignability Caching Architecture

## 1. Canonical Type Storage
Types are represented as compact `TypeId(u32)` indices referencing central canonical tables:
- **Primitives**: `ANY (0)`, `UNKNOWN (1)`, `NUMBER (2)`, `STRING (3)`, `BOOLEAN (4)`, `VOID (5)`, `UNDEFINED (6)`, `NULL (7)`.
- **Deduplication**: Structural types (unions, objects, tuples) resolve to unique, canonical `TypeId` instances.

## 2. Fast Path Primitive Assignability
Subtyping checks invoke fast-path checks before deep structural recursion:
```rust
if source == target || target == TypeId::ANY || source == TypeId::ANY {
    return true;
}
```

## 3. Assignability Relation Memoization
Assignability queries `is_assignable(source, target)` are cached in a dedicated `RelationKey` table:
```rust
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct RelationKey {
    pub source: TypeId,
    pub target: TypeId,
}
```
- Eliminates repeated structural checking of identical type pairs.
- Drastically reduces cache miss penalties and stack pressure.

## 4. Stack Safety & Work Lists
- Iterative work-stacks (`Vec<TypeId>`) replace deep recursive calls during complex union/intersection resolution to eliminate stack overflow risk on deep type hierarchies.
