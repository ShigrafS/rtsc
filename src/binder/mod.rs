// Binder & Scope Analysis engine.
// Uses compact integer SymbolId and ScopeId with contiguous arrays to prevent pointer-chasing and allocation overhead.

use crate::ast::{AstArena, NodeId, NodeKind};
use crate::interner::NameId;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

impl SymbolId {
    pub const DUMMY: SymbolId = SymbolId(u32::MAX);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);

impl ScopeId {
    pub const GLOBAL: ScopeId = ScopeId(0);
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: NameId,
    pub flags: u16,
    pub declaration: NodeId,
}

impl Symbol {
    pub fn new(name: NameId, flags: u16, declaration: NodeId) -> Self {
        Self {
            name,
            flags,
            declaration,
        }
    }
}

pub struct Scope {
    pub parent: Option<ScopeId>,
    pub bindings: HashMap<NameId, SymbolId>,
}

impl Scope {
    pub fn new(parent: Option<ScopeId>) -> Self {
        Self {
            parent,
            bindings: HashMap::with_capacity(16),
        }
    }
}

#[derive(Default)]
pub struct Binder {
    pub symbols: Vec<Symbol>,
    pub scopes: Vec<Scope>,
}

impl Binder {
    pub fn new() -> Self {
        let mut binder = Self {
            symbols: Vec::with_capacity(512),
            scopes: Vec::with_capacity(64),
        };
        // Initialize global scope (ScopeId(0))
        binder.scopes.push(Scope::new(None));
        binder
    }

    pub fn bind_program(&mut self, root: NodeId, arena: &AstArena) {
        if root == NodeId::DUMMY {
            return;
        }

        let hot = arena.get_hot(root);
        if hot.kind != NodeKind::Program {
            return;
        }

        // Traverse declarations at root
        let mut curr = hot.child_a;
        while curr != NodeId::DUMMY {
            let stmt_hot = arena.get_hot(curr);
            if stmt_hot.kind == NodeKind::VariableDecl {
                self.declare_symbol(stmt_hot.name, 1 /* Variable */, curr, ScopeId::GLOBAL);
            }
            curr = stmt_hot.child_b;
        }
    }

    pub fn declare_symbol(
        &mut self,
        name: NameId,
        flags: u16,
        decl: NodeId,
        scope_id: ScopeId,
    ) -> SymbolId {
        let sym_id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(Symbol::new(name, flags, decl));

        if let Some(scope) = self.scopes.get_mut(scope_id.0 as usize) {
            scope.bindings.insert(name, sym_id);
        }

        sym_id
    }

    pub fn resolve_symbol(&self, name: NameId, mut scope_id: ScopeId) -> Option<SymbolId> {
        loop {
            let scope = self.scopes.get(scope_id.0 as usize)?;
            if let Some(&sym_id) = scope.bindings.get(&name) {
                return Some(sym_id);
            }
            scope_id = scope.parent?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binder_declare_and_resolve() {
        let mut binder = Binder::new();
        let name = NameId(42);
        let decl = NodeId(0);

        let sym_id = binder.declare_symbol(name, 1, decl, ScopeId::GLOBAL);
        let resolved = binder.resolve_symbol(name, ScopeId::GLOBAL);

        assert_eq!(resolved, Some(sym_id));
        assert_eq!(binder.symbols[sym_id.0 as usize].name, name);
    }
}
