// Cursor-based parser operating over compact token slice.
// Pushes nodes into AstArena without heap allocation per AST node.

use crate::ast::{AstArena, ColdNode, HotNode, NodeId, NodeKind};
use crate::interner::StringInterner;
use crate::lexer::{Token, TokenKind};
use crate::span::Span;

pub struct Parser<'a> {
    tokens: &'a [Token],
    src: &'a str,
    pos: u32,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], src: &'a str) -> Self {
        Self { tokens, src, pos: 0 }
    }

    #[inline(always)]
    pub fn peek(&self) -> TokenKind {
        if (self.pos as usize) < self.tokens.len() {
            self.tokens[self.pos as usize].kind
        } else {
            TokenKind::Eof
        }
    }

    #[inline(always)]
    pub fn bump(&mut self) -> Token {
        if (self.pos as usize) < self.tokens.len() {
            let tok = self.tokens[self.pos as usize];
            self.pos += 1;
            tok
        } else {
            let end = self.src.len() as u32;
            Token::new(TokenKind::Eof, end, end)
        }
    }

    pub fn parse_program(&mut self, interner: &mut StringInterner, arena: &mut AstArena) -> NodeId {
        let start_pos = if let Some(first) = self.tokens.first() {
            first.start
        } else {
            0
        };

        let mut first_stmt = NodeId::DUMMY;
        let mut last_stmt = NodeId::DUMMY;

        while self.peek() != TokenKind::Eof {
            let stmt_id = self.parse_statement(interner, arena);
            if first_stmt == NodeId::DUMMY {
                first_stmt = stmt_id;
            }
            last_stmt = stmt_id;
        }

        let end_pos = if self.pos > 0 && (self.pos as usize - 1) < self.tokens.len() {
            self.tokens[self.pos as usize - 1].end
        } else {
            self.src.len() as u32
        };

        arena.alloc(
            HotNode::new(NodeKind::Program, first_stmt, last_stmt, crate::interner::NameId::EMPTY),
            ColdNode::new(Span::new(start_pos, end_pos)),
        )
    }

    fn parse_statement(&mut self, interner: &mut StringInterner, arena: &mut AstArena) -> NodeId {
        match self.peek() {
            TokenKind::Const | TokenKind::Let | TokenKind::Var => self.parse_variable_declaration(interner, arena),
            _ => self.parse_expression_statement(interner, arena),
        }
    }

    fn parse_variable_declaration(&mut self, interner: &mut StringInterner, arena: &mut AstArena) -> NodeId {
        let kw_tok = self.bump(); // const / let / var
        let ident_tok = self.bump();
        let ident_text = ident_tok.span_text(self.src);
        let name_id = interner.intern(ident_text);

        let ident_node = arena.alloc(
            HotNode::new(NodeKind::Identifier, NodeId::DUMMY, NodeId::DUMMY, name_id),
            ColdNode::new(Span::new(ident_tok.start, ident_tok.end)),
        );

        let mut init_node = NodeId::DUMMY;
        if self.peek() == TokenKind::Equals {
            self.bump();
            init_node = self.parse_expression(interner, arena);
        }

        if self.peek() == TokenKind::Semicolon {
            self.bump();
        }

        let end_pos = if init_node != NodeId::DUMMY {
            arena.get_cold(init_node).span.end
        } else {
            ident_tok.end
        };

        arena.alloc(
            HotNode::new(NodeKind::VariableDecl, ident_node, init_node, name_id),
            ColdNode::new(Span::new(kw_tok.start, end_pos)),
        )
    }

    fn parse_expression_statement(&mut self, interner: &mut StringInterner, arena: &mut AstArena) -> NodeId {
        let expr = self.parse_expression(interner, arena);
        if self.peek() == TokenKind::Semicolon {
            self.bump();
        }
        let cold_span = arena.get_cold(expr).span;
        arena.alloc(
            HotNode::new(NodeKind::ExprStmt, expr, NodeId::DUMMY, crate::interner::NameId::EMPTY),
            ColdNode::new(cold_span),
        )
    }

    fn parse_expression(&mut self, interner: &mut StringInterner, arena: &mut AstArena) -> NodeId {
        let mut left = self.parse_primary_expression(interner, arena);

        while self.peek() == TokenKind::Plus || self.peek() == TokenKind::Minus {
            let _op_tok = self.bump();
            let right = self.parse_primary_expression(interner, arena);
            let start = arena.get_cold(left).span.start;
            let end = arena.get_cold(right).span.end;

            left = arena.alloc(
                HotNode::new(NodeKind::BinaryExpr, left, right, crate::interner::NameId::EMPTY),
                ColdNode::new(Span::new(start, end)),
            );
        }

        left
    }

    fn parse_primary_expression(&mut self, interner: &mut StringInterner, arena: &mut AstArena) -> NodeId {
        let tok = self.bump();
        match tok.kind {
            TokenKind::NumericLiteral => {
                let name = interner.intern(tok.span_text(self.src));
                arena.alloc(
                    HotNode::new(NodeKind::NumericLiteral, NodeId::DUMMY, NodeId::DUMMY, name),
                    ColdNode::new(Span::new(tok.start, tok.end)),
                )
            }
            TokenKind::Identifier => {
                let name = interner.intern(tok.span_text(self.src));
                arena.alloc(
                    HotNode::new(NodeKind::Identifier, NodeId::DUMMY, NodeId::DUMMY, name),
                    ColdNode::new(Span::new(tok.start, tok.end)),
                )
            }
            _ => arena.alloc(
                HotNode::new(NodeKind::Identifier, NodeId::DUMMY, NodeId::DUMMY, crate::interner::NameId::EMPTY),
                ColdNode::new(Span::new(tok.start, tok.end)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_simple_const_decl() {
        let code = "const x = 42;";
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
        assert_eq!(arena.get_hot(root).kind, NodeKind::Program);
    }
}
