pub mod types;

pub use types::{ColdNode, ExprId, HotNode, NodeId, NodeKind, StmtId};

// Contiguous arena storage for hot and cold AST node data.
#[derive(Debug, Default)]
pub struct AstArena {
    hot_nodes: Vec<HotNode>,
    cold_nodes: Vec<ColdNode>,
}

impl AstArena {
    pub fn new() -> Self {
        Self {
            hot_nodes: Vec::with_capacity(2048),
            cold_nodes: Vec::with_capacity(2048),
        }
    }

    pub fn alloc(&mut self, hot: HotNode, cold: ColdNode) -> NodeId {
        let id = NodeId(self.hot_nodes.len() as u32);
        self.hot_nodes.push(hot);
        self.cold_nodes.push(cold);
        id
    }

    #[inline]
    pub fn get_hot(&self, id: NodeId) -> &HotNode {
        &self.hot_nodes[id.0 as usize]
    }

    #[inline]
    pub fn get_cold(&self, id: NodeId) -> &ColdNode {
        &self.cold_nodes[id.0 as usize]
    }

    pub fn len(&self) -> usize {
        self.hot_nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hot_nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interner::NameId;
    use crate::span::Span;

    #[test]
    fn test_ast_arena_alloc() {
        let mut arena = AstArena::new();
        let hot = HotNode::new(
            NodeKind::Identifier,
            NodeId::DUMMY,
            NodeId::DUMMY,
            NameId(1),
        );
        let cold = ColdNode::new(Span::new(0, 3));
        let id = arena.alloc(hot, cold);

        assert_eq!(id, NodeId(0));
        assert_eq!(arena.get_hot(id).kind, NodeKind::Identifier);
        assert_eq!(arena.get_cold(id).span, Span::new(0, 3));
    }
}
