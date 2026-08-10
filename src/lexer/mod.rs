pub mod char_class;
pub mod kind;

pub use kind::TokenKind;

// Compact 12-byte Token structure layout. Zero heap allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Token {
    pub kind: TokenKind,
    pub start: u32,
    pub end: u32,
}

impl Token {
    #[inline]
    pub const fn new(kind: TokenKind, start: u32, end: u32) -> Self {
        Self { kind, start, end }
    }

    #[inline]
    pub fn span_text<'a>(&self, src: &'a str) -> &'a str {
        &src[self.start as usize..self.end as usize]
    }
}

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    end: usize,
    pub skip_trivia: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
            end: src.len(),
            skip_trivia: false,
        }
    }

    #[inline(always)]
    pub fn is_eof(&self) -> bool {
        self.pos >= self.end
    }

    #[inline(always)]
    fn peek_byte(&self) -> u8 {
        if self.pos < self.end {
            self.src[self.pos]
        } else {
            0
        }
    }

    #[inline(always)]
    fn bump(&mut self) -> u8 {
        let b = self.src[self.pos];
        self.pos += 1;
        b
    }

    // Zero-allocation main lexing loop
    pub fn next_token(&mut self) -> Token {
        loop {
            if self.is_eof() {
                return Token::new(TokenKind::Eof, self.pos as u32, self.pos as u32);
            }

            let start = self.pos as u32;
            let b = self.bump();

            // Unicode slow path invoked only when byte >= 128
            if b >= 128 {
                return self.scan_unicode(start);
            }

            let token = match b {
                b' ' | b'\t' => self.scan_whitespace(start),
                b'\n' | b'\r' => Token::new(TokenKind::LineBreak, start, self.pos as u32),

                b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => self.scan_identifier(start),
                b'0'..=b'9' => self.scan_number(start),

                b'"' | b'\'' => self.scan_string(start, b),
                b'`' => self.scan_template(start),
                b'/' => self.scan_slash(start),

                b'(' => Token::new(TokenKind::OpenParen, start, self.pos as u32),
                b')' => Token::new(TokenKind::CloseParen, start, self.pos as u32),
                b'{' => Token::new(TokenKind::OpenBrace, start, self.pos as u32),
                b'}' => Token::new(TokenKind::CloseBrace, start, self.pos as u32),
                b'[' => Token::new(TokenKind::OpenBracket, start, self.pos as u32),
                b']' => Token::new(TokenKind::CloseBracket, start, self.pos as u32),
                b';' => Token::new(TokenKind::Semicolon, start, self.pos as u32),
                b',' => Token::new(TokenKind::Comma, start, self.pos as u32),
                b':' => Token::new(TokenKind::Colon, start, self.pos as u32),
                b'~' => Token::new(TokenKind::Tilde, start, self.pos as u32),
                b'@' => Token::new(TokenKind::At, start, self.pos as u32),

                b'.' => {
                    if self.peek_byte() == b'.'
                        && self.pos + 1 < self.end
                        && self.src[self.pos + 1] == b'.'
                    {
                        self.pos += 2;
                        Token::new(TokenKind::DotDotDot, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::Dot, start, self.pos as u32)
                    }
                }

                b'=' => {
                    if self.peek_byte() == b'=' {
                        self.bump();
                        if self.peek_byte() == b'=' {
                            self.bump();
                            Token::new(TokenKind::EqualsEqualsEquals, start, self.pos as u32)
                        } else {
                            Token::new(TokenKind::EqualsEquals, start, self.pos as u32)
                        }
                    } else if self.peek_byte() == b'>' {
                        self.bump();
                        Token::new(TokenKind::Arrow, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::Equals, start, self.pos as u32)
                    }
                }

                b'!' => {
                    if self.peek_byte() == b'=' {
                        self.bump();
                        if self.peek_byte() == b'=' {
                            self.bump();
                            Token::new(TokenKind::ExclamationEqualsEquals, start, self.pos as u32)
                        } else {
                            Token::new(TokenKind::ExclamationEquals, start, self.pos as u32)
                        }
                    } else {
                        Token::new(TokenKind::Exclamation, start, self.pos as u32)
                    }
                }

                b'+' => {
                    if self.peek_byte() == b'+' {
                        self.bump();
                        Token::new(TokenKind::PlusPlus, start, self.pos as u32)
                    } else if self.peek_byte() == b'=' {
                        self.bump();
                        Token::new(TokenKind::PlusEquals, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::Plus, start, self.pos as u32)
                    }
                }

                b'-' => {
                    if self.peek_byte() == b'-' {
                        self.bump();
                        Token::new(TokenKind::MinusMinus, start, self.pos as u32)
                    } else if self.peek_byte() == b'=' {
                        self.bump();
                        Token::new(TokenKind::MinusEquals, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::Minus, start, self.pos as u32)
                    }
                }

                b'*' => {
                    if self.peek_byte() == b'*' {
                        self.bump();
                        Token::new(TokenKind::AsteriskAsterisk, start, self.pos as u32)
                    } else if self.peek_byte() == b'=' {
                        self.bump();
                        Token::new(TokenKind::AsteriskEquals, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::Asterisk, start, self.pos as u32)
                    }
                }

                b'<' => {
                    if self.peek_byte() == b'=' {
                        self.bump();
                        Token::new(TokenKind::LessThanEquals, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::LessThan, start, self.pos as u32)
                    }
                }

                b'>' => {
                    if self.peek_byte() == b'=' {
                        self.bump();
                        Token::new(TokenKind::GreaterThanEquals, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::GreaterThan, start, self.pos as u32)
                    }
                }

                b'&' => {
                    if self.peek_byte() == b'&' {
                        self.bump();
                        Token::new(TokenKind::AmpersandAmpersand, start, self.pos as u32)
                    } else if self.peek_byte() == b'=' {
                        self.bump();
                        Token::new(TokenKind::AmpersandEquals, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::Ampersand, start, self.pos as u32)
                    }
                }

                b'|' => {
                    if self.peek_byte() == b'|' {
                        self.bump();
                        Token::new(TokenKind::BarBar, start, self.pos as u32)
                    } else if self.peek_byte() == b'=' {
                        self.bump();
                        Token::new(TokenKind::BarEquals, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::Bar, start, self.pos as u32)
                    }
                }

                b'?' => {
                    if self.peek_byte() == b'.' {
                        self.bump();
                        Token::new(TokenKind::QuestionDot, start, self.pos as u32)
                    } else {
                        Token::new(TokenKind::Question, start, self.pos as u32)
                    }
                }

                _ => Token::new(TokenKind::Unknown, start, self.pos as u32),
            };

            if self.skip_trivia && token.kind.is_trivia() {
                continue;
            }

            return token;
        }
    }

    #[inline(always)]
    fn scan_whitespace(&mut self, start: u32) -> Token {
        while self.pos < self.end && (self.src[self.pos] == b' ' || self.src[self.pos] == b'\t') {
            self.pos += 1;
        }
        Token::new(TokenKind::Whitespace, start, self.pos as u32)
    }

    #[inline(always)]
    fn scan_identifier(&mut self, start: u32) -> Token {
        while self.pos < self.end {
            let b = self.src[self.pos];
            if matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$') {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text = &self.src[start as usize..self.pos];
        let kind = match text {
            b"const" => TokenKind::Const,
            b"let" => TokenKind::Let,
            b"var" => TokenKind::Var,
            b"function" => TokenKind::Function,
            b"return" => TokenKind::Return,
            b"if" => TokenKind::If,
            b"else" => TokenKind::Else,
            b"for" => TokenKind::For,
            b"while" => TokenKind::While,
            b"import" => TokenKind::Import,
            b"export" => TokenKind::Export,
            b"class" => TokenKind::Class,
            b"interface" => TokenKind::Interface,
            b"type" => TokenKind::Type,
            b"async" => TokenKind::Async,
            b"await" => TokenKind::Await,
            b"true" => TokenKind::True,
            b"false" => TokenKind::False,
            b"null" => TokenKind::Null,
            b"undefined" => TokenKind::Undefined,
            _ => TokenKind::Identifier,
        };
        Token::new(kind, start, self.pos as u32)
    }

    #[inline(always)]
    fn scan_number(&mut self, start: u32) -> Token {
        while self.pos < self.end && self.src[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.end
            && self.src[self.pos] == b'.'
            && self.pos + 1 < self.end
            && self.src[self.pos + 1].is_ascii_digit()
        {
            self.pos += 1;
            while self.pos < self.end && self.src[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        Token::new(TokenKind::NumericLiteral, start, self.pos as u32)
    }

    #[inline(always)]
    fn scan_string(&mut self, start: u32, quote: u8) -> Token {
        while self.pos < self.end {
            let b = self.bump();
            if b == quote {
                break;
            }
            if b == b'\\' && self.pos < self.end {
                self.bump();
            }
        }
        Token::new(TokenKind::StringLiteral, start, self.pos as u32)
    }

    #[inline(always)]
    fn scan_template(&mut self, start: u32) -> Token {
        while self.pos < self.end {
            let b = self.bump();
            if b == b'`' {
                break;
            }
            if b == b'\\' && self.pos < self.end {
                self.bump();
            }
        }
        Token::new(
            TokenKind::NoSubstitutionTemplateLiteral,
            start,
            self.pos as u32,
        )
    }

    #[inline(always)]
    fn scan_slash(&mut self, start: u32) -> Token {
        if self.peek_byte() == b'/' {
            self.bump();
            while self.pos < self.end && self.src[self.pos] != b'\n' && self.src[self.pos] != b'\r'
            {
                self.pos += 1;
            }
            Token::new(TokenKind::SingleLineComment, start, self.pos as u32)
        } else if self.peek_byte() == b'*' {
            self.bump();
            while self.pos < self.end {
                if self.bump() == b'*' && self.peek_byte() == b'/' {
                    self.bump();
                    break;
                }
            }
            Token::new(TokenKind::MultiLineComment, start, self.pos as u32)
        } else if self.peek_byte() == b'=' {
            self.bump();
            Token::new(TokenKind::SlashEquals, start, self.pos as u32)
        } else {
            Token::new(TokenKind::Slash, start, self.pos as u32)
        }
    }

    fn scan_unicode(&mut self, start: u32) -> Token {
        // Simple UTF-8 boundary scan for non-ASCII identifier / text
        while self.pos < self.end && self.src[self.pos] >= 128 {
            self.pos += 1;
        }
        Token::new(TokenKind::Identifier, start, self.pos as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_token_size_is_12_bytes() {
        assert_eq!(size_of::<Token>(), 12);
    }

    #[test]
    fn test_lexer_ascii_fast_path() {
        let code = "const foo = 42 + bar;";
        let mut lexer = Lexer::new(code);
        lexer.skip_trivia = true;

        let tokens: Vec<TokenKind> = std::iter::from_fn(|| {
            let tok = lexer.next_token();
            if tok.kind == TokenKind::Eof {
                None
            } else {
                Some(tok.kind)
            }
        })
        .collect();

        assert_eq!(
            tokens,
            vec![
                TokenKind::Const,
                TokenKind::Identifier,
                TokenKind::Equals,
                TokenKind::NumericLiteral,
                TokenKind::Plus,
                TokenKind::Identifier,
                TokenKind::Semicolon,
            ]
        );
    }
}
