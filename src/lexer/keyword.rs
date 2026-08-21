// High-performance keyword recognition for TypeScript and JavaScript.
// Uses length-stratified dispatch with frequency-ordered branches to minimize branch mispredictions.

use crate::lexer::kind::TokenKind;

#[inline(always)]
pub fn lookup_keyword(bytes: &[u8]) -> TokenKind {
    match bytes.len() {
        // Length 2
        2 => match bytes {
            b"if" => TokenKind::If,
            b"as" => TokenKind::As,
            b"in" => TokenKind::In,
            b"do" => TokenKind::Do,
            _ => TokenKind::Identifier,
        },

        // Length 3
        3 => match bytes {
            b"let" => TokenKind::Let,
            b"var" => TokenKind::Var,
            b"for" => TokenKind::For,
            b"new" => TokenKind::New,
            b"try" => TokenKind::Try,
            _ => TokenKind::Identifier,
        },

        // Length 4
        4 => match bytes {
            b"this" => TokenKind::This,
            b"true" => TokenKind::True,
            b"null" => TokenKind::Null,
            b"type" => TokenKind::Type,
            b"else" => TokenKind::Else,
            b"from" => TokenKind::From,
            b"case" => TokenKind::Case,
            b"enum" => TokenKind::Enum,
            b"void" => TokenKind::Void,
            _ => TokenKind::Identifier,
        },

        // Length 5
        5 => match bytes {
            b"const" => TokenKind::Const,
            b"false" => TokenKind::False,
            b"async" => TokenKind::Async,
            b"await" => TokenKind::Await,
            b"class" => TokenKind::Class,
            b"while" => TokenKind::While,
            b"break" => TokenKind::Break,
            b"catch" => TokenKind::Catch,
            b"super" => TokenKind::Super,
            b"throw" => TokenKind::Throw,
            b"yield" => TokenKind::Yield,
            _ => TokenKind::Identifier,
        },

        // Length 6
        6 => match bytes {
            b"return" => TokenKind::Return,
            b"import" => TokenKind::Import,
            b"export" => TokenKind::Export,
            b"typeof" => TokenKind::TypeOf,
            b"switch" => TokenKind::Switch,
            b"delete" => TokenKind::Delete,
            _ => TokenKind::Identifier,
        },

        // Length 7
        7 => match bytes {
            b"default" => TokenKind::Default,
            b"extends" => TokenKind::Extends,
            b"finally" => TokenKind::Finally,
            _ => TokenKind::Identifier,
        },

        // Length 8
        8 => match bytes {
            b"function" => TokenKind::Function,
            b"continue" => TokenKind::Continue,
            _ => TokenKind::Identifier,
        },

        // Length 9
        9 => match bytes {
            b"interface" => TokenKind::Interface,
            b"undefined" => TokenKind::Undefined,
            _ => TokenKind::Identifier,
        },

        // Length 10
        10 => match bytes {
            b"implements" => TokenKind::Implements,
            b"instanceof" => TokenKind::InstanceOf,
            _ => TokenKind::Identifier,
        },

        _ => TokenKind::Identifier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_all_keywords() {
        assert_eq!(lookup_keyword(b"const"), TokenKind::Const);
        assert_eq!(lookup_keyword(b"let"), TokenKind::Let);
        assert_eq!(lookup_keyword(b"var"), TokenKind::Var);
        assert_eq!(lookup_keyword(b"function"), TokenKind::Function);
        assert_eq!(lookup_keyword(b"return"), TokenKind::Return);
        assert_eq!(lookup_keyword(b"if"), TokenKind::If);
        assert_eq!(lookup_keyword(b"else"), TokenKind::Else);
        assert_eq!(lookup_keyword(b"import"), TokenKind::Import);
        assert_eq!(lookup_keyword(b"export"), TokenKind::Export);
        assert_eq!(lookup_keyword(b"class"), TokenKind::Class);
        assert_eq!(lookup_keyword(b"interface"), TokenKind::Interface);
        assert_eq!(lookup_keyword(b"type"), TokenKind::Type);
        assert_eq!(lookup_keyword(b"implements"), TokenKind::Implements);
        assert_eq!(lookup_keyword(b"undefined"), TokenKind::Undefined);
    }

    #[test]
    fn test_lookup_non_keywords() {
        assert_eq!(lookup_keyword(b"foo"), TokenKind::Identifier);
        assert_eq!(lookup_keyword(b"constant"), TokenKind::Identifier);
        assert_eq!(lookup_keyword(b"letting"), TokenKind::Identifier);
        assert_eq!(lookup_keyword(b"functions"), TokenKind::Identifier);
        assert_eq!(lookup_keyword(b"i"), TokenKind::Identifier);
        assert_eq!(lookup_keyword(b"aVeryLongIdentifierName"), TokenKind::Identifier);
    }
}
