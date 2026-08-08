# Incremental Compilation & Editor Latency Strategy

## 1. Incremental Re-Compilation Pipeline
When a source file is edited, `rtsc` re-uses cached artifacts from unaffected files:

```
File A (Modified)   File B (Unchanged)   File C (Unchanged)
      │                    │                    │
   Reparse              Cached               Cached
  & Rebind             AST/Symbols          AST/Symbols
      │                    │                    │
      └────────────────────┼────────────────────┘
                           ▼
                    Incremental Check
```

## 2. Granular Dependency Invalidation
- **Token Cache**: Re-lex only changed lines or affected files.
- **AST Cache**: Fine-grained subtree invalidation based on byte spans.
- **Symbol & Scope Graph**: Re-bind modified files; propagate changes only if public export signatures differ.

## 3. LSP & Editor Latency Targets
- Target editor completion response time < 5ms for monorepo projects.
- Thread-local Arenas: Avoid lock contention between background type-checking threads and LSP diagnostic queries.
