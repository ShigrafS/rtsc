// Type Checker Engine with canonical types, integer TypeId, and assignability relation caching.

use std::collections::HashMap;
use crate::ast::{AstArena, NodeId, NodeKind};
use crate::interner::NameId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

impl TypeId {
    pub const ANY: TypeId = TypeId(0);
    pub const UNKNOWN: TypeId = TypeId(1);
    pub const NUMBER: TypeId = TypeId(2);
    pub const STRING: TypeId = TypeId(3);
    pub const BOOLEAN: TypeId = TypeId(4);
    pub const VOID: TypeId = TypeId(5);
    pub const UNDEFINED: TypeId = TypeId(6);
    pub const NULL: TypeId = TypeId(7);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Any,
    Unknown,
    Number,
    String,
    Boolean,
    Void,
    Undefined,
    Null,
    Union(Vec<TypeId>),
    Object(Vec<(NameId, TypeId)>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RelationKey {
    pub source: TypeId,
    pub target: TypeId,
}

pub struct TypeChecker {
    types: Vec<TypeKind>,
    assignability_cache: HashMap<RelationKey, bool>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        let mut checker = Self {
            types: Vec::with_capacity(256),
            assignability_cache: HashMap::with_capacity(1024),
        };
        // Register canonical built-in primitives
        checker.types.push(TypeKind::Any);       // 0
        checker.types.push(TypeKind::Unknown);   // 1
        checker.types.push(TypeKind::Number);    // 2
        checker.types.push(TypeKind::String);    // 3
        checker.types.push(TypeKind::Boolean);   // 4
        checker.types.push(TypeKind::Void);      // 5
        checker.types.push(TypeKind::Undefined); // 6
        checker.types.push(TypeKind::Null);      // 7
        checker
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_type(&self, id: TypeId) -> Option<&TypeKind> {
        self.types.get(id.0 as usize)
    }

    pub fn is_assignable(&mut self, source: TypeId, target: TypeId) -> bool {
        // Fast path 1: Identity equality
        if source == target || target == TypeId::ANY || source == TypeId::ANY {
            return true;
        }

        let key = RelationKey { source, target };
        if let Some(&cached) = self.assignability_cache.get(&key) {
            return cached;
        }

        let result = match (self.get_type(source), self.get_type(target)) {
            (Some(TypeKind::Number), Some(TypeKind::Number)) => true,
            (Some(TypeKind::String), Some(TypeKind::String)) => true,
            (Some(TypeKind::Boolean), Some(TypeKind::Boolean)) => true,
            _ => false,
        };

        self.assignability_cache.insert(key, result);
        result
    }

    pub fn check_node(&mut self, node: NodeId, arena: &AstArena) -> TypeId {
        if node == NodeId::DUMMY {
            return TypeId::VOID;
        }

        let hot = arena.get_hot(node);
        match hot.kind {
            NodeKind::NumericLiteral => TypeId::NUMBER,
            NodeKind::StringLiteral => TypeId::STRING,
            NodeKind::BinaryExpr => TypeId::NUMBER,
            _ => TypeId::ANY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_assignability_fast_path() {
        let mut checker = TypeChecker::new();
        assert!(checker.is_assignable(TypeId::NUMBER, TypeId::NUMBER));
        assert!(checker.is_assignable(TypeId::NUMBER, TypeId::ANY));
        assert!(!checker.is_assignable(TypeId::NUMBER, TypeId::STRING));
    }

    #[test]
    fn test_assignability_caching() {
        let mut checker = TypeChecker::new();
        let res1 = checker.is_assignable(TypeId::NUMBER, TypeId::STRING);
        let res2 = checker.is_assignable(TypeId::NUMBER, TypeId::STRING);

        assert_eq!(res1, res2);
        assert_eq!(checker.assignability_cache.len(), 1);
    }
}
