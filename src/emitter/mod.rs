// JavaScript and D.TS Emitter / Code Generator module.

use crate::ast::{AstArena, NodeId, NodeKind};
use crate::interner::StringInterner;

#[derive(Default)]
pub struct Emitter {
    output: String,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            output: String::with_capacity(1024),
        }
    }

    pub fn emit_program(&mut self, root: NodeId, arena: &AstArena, interner: &StringInterner) -> String {
        self.output.clear();
        if root == NodeId::DUMMY {
            return self.output.clone();
        }

        let hot = arena.get_hot(root);
        if hot.kind == NodeKind::Program {
            let mut curr = hot.child_a;
            while curr != NodeId::DUMMY {
                self.emit_node(curr, arena, interner);
                self.output.push('\n');
                curr = arena.get_hot(curr).child_b;
            }
        }
        self.output.clone()
    }

    fn emit_node(&mut self, node: NodeId, arena: &AstArena, interner: &StringInterner) {
        if node == NodeId::DUMMY {
            return;
        }

        let hot = arena.get_hot(node);
        match hot.kind {
            NodeKind::VariableDecl => {
                self.output.push_str("const ");
                if let Some(name) = interner.resolve(hot.name) {
                    self.output.push_str(name);
                }
                if hot.child_b != NodeId::DUMMY {
                    self.output.push_str(" = ");
                    self.emit_node(hot.child_b, arena, interner);
                }
                self.output.push(';');
            }
            NodeKind::NumericLiteral | NodeKind::Identifier => {
                if let Some(val) = interner.resolve(hot.name) {
                    self.output.push_str(val);
                }
            }
            NodeKind::BinaryExpr => {
                self.emit_node(hot.child_a, arena, interner);
                self.output.push_str(" + ");
                self.emit_node(hot.child_b, arena, interner);
            }
            NodeKind::ExprStmt => {
                self.emit_node(hot.child_a, arena, interner);
                self.output.push(';');
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexer, TokenKind};
    use crate::parser::Parser;

    #[test]
    fn test_emitter_simple_const() {
        let code = "const foo = 42;";
        let mut lexer = Lexer::new(code);
        lexer.skip_trivia = true;

        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok.kind == TokenKind::Eof {
                break;
            }
            tokens.push(tok);
        }

        let mut interner = StringInterner::new();
        let mut arena = AstArena::new();
        let mut parser = Parser::new(&tokens, code);

        let root = parser.parse_program(&mut interner, &mut arena);
        let mut emitter = Emitter::new();
        let emitted = emitter.emit_program(root, &mut arena, &interner);

        assert!(emitted.contains("const foo = 42;"));
    }
}
