// Compact u8 TokenKind enum (< 256 variants) ensuring Token stays 12 bytes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TokenKind {
    // Control & Special
    Eof = 0,
    Unknown,
    Invalid,

    // Trivia
    Whitespace,
    LineBreak,
    SingleLineComment,
    MultiLineComment,

    // Identifiers & Literals
    Identifier,
    NumericLiteral,
    StringLiteral,
    NoSubstitutionTemplateLiteral,
    TemplateHead,
    TemplateMiddle,
    TemplateTail,
    RegularExpressionLiteral,

    // Keywords
    Const,
    Let,
    Var,
    Function,
    Return,
    If,
    Else,
    For,
    While,
    Do,
    Import,
    Export,
    From,
    As,
    Class,
    Interface,
    Type,
    Enum,
    Async,
    Await,
    Yield,
    Break,
    Continue,
    Switch,
    Case,
    Default,
    Try,
    Catch,
    Finally,
    Throw,
    New,
    This,
    Super,
    Extends,
    Implements,
    TypeOf,
    InstanceOf,
    In,
    Void,
    Delete,
    True,
    False,
    Null,
    Undefined,

    // Punctuators / Operators
    OpenParen,               // (
    CloseParen,              // )
    OpenBrace,               // {
    CloseBrace,              // }
    OpenBracket,             // [
    CloseBracket,            // ]
    Semicolon,               // ;
    Comma,                   // ,
    Dot,                     // .
    DotDotDot,               // ...
    Colon,                   // :
    Question,                // ?
    QuestionDot,             // ?.
    Equals,                  // =
    EqualsEquals,            // ==
    EqualsEqualsEquals,      // ===
    Exclamation,             // !
    ExclamationEquals,       // !=
    ExclamationEqualsEquals, // !==
    Plus,                    // +
    PlusPlus,                // ++
    PlusEquals,              // +=
    Minus,                   // -
    MinusMinus,              // --
    MinusEquals,             // -=
    Asterisk,                // *
    AsteriskAsterisk,        // **
    AsteriskEquals,          // *=
    Slash,                   // /
    SlashEquals,             // /=
    Percent,                 // %
    PercentEquals,           // %=
    Ampersand,               // &
    AmpersandAmpersand,      // &&
    AmpersandEquals,         // &=
    Bar,                     // |
    BarBar,                  // ||
    BarEquals,               // |=
    Caret,                   // ^
    CaretEquals,             // ^=
    Tilde,                   // ~
    LessThan,                // <
    LessThanEquals,          // <=
    GreaterThan,             // >
    GreaterThanEquals,       // >=
    Arrow,                   // =>
    At,                      // @
}

impl TokenKind {
    #[inline]
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            TokenKind::Whitespace
                | TokenKind::LineBreak
                | TokenKind::SingleLineComment
                | TokenKind::MultiLineComment
        )
    }

    #[inline]
    pub fn is_keyword(&self) -> bool {
        (*self as u8) >= (TokenKind::Const as u8) && (*self as u8) <= (TokenKind::Undefined as u8)
    }
}
