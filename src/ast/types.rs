// Compact integer IDs for AST arena nodes.

use crate::interner::NameId;
use crate::span::Span;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

impl NodeId {
    pub const DUMMY: NodeId = NodeId(u32::MAX);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct StmtId(pub u32);

// Hot AST node containing kind, flags, and direct child IDs.
// Stored contiguously in arena for cache locality during traversal.
#[derive(Debug, Clone)]
pub struct HotNode {
    pub kind: NodeKind,
    pub child_a: NodeId,
    pub child_b: NodeId,
    pub name: NameId,
    pub flags: u16,
}

impl HotNode {
    pub fn new(kind: NodeKind, child_a: NodeId, child_b: NodeId, name: NameId) -> Self {
        Self {
            kind,
            child_a,
            child_b,
            name,
            flags: 0,
        }
    }
}

// Cold AST node containing source span and trivia references.
// Accessed only when rendering diagnostics, formatters, or maps.
#[derive(Debug, Clone)]
pub struct ColdNode {
    pub span: Span,
}

impl ColdNode {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeKind {
    Program,
    VariableDecl,
    Identifier,
    NumericLiteral,
    StringLiteral,
    BinaryExpr,
    CallExpr,
    BlockStmt,
    FunctionDecl,
    ReturnStmt,
    ExprStmt,
}
