// Table-driven character classification and bitmask lookup for ASCII fast path.
// Eliminates branch misses and multi-way range comparisons in hot scanning loops.

pub const FLAG_IDENT_START: u8 = 1 << 0;    // [a-zA-Z_$]
pub const FLAG_IDENT_CONTINUE: u8 = 1 << 1; // [a-zA-Z0-9_$]
pub const FLAG_DIGIT: u8 = 1 << 2;          // [0-9]
pub const FLAG_WHITESPACE: u8 = 1 << 3;     // [' ', '\t']
pub const FLAG_LINE_BREAK: u8 = 1 << 4;     // ['\n', '\r']
pub const FLAG_QUOTE: u8 = 1 << 5;          // ['"', '\'']
pub const FLAG_OPERATOR: u8 = 1 << 6;       // Punctuators and operators
pub const FLAG_NON_ASCII: u8 = 1 << 7;      // Byte >= 128 (Unicode slow path)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CharClass {
    IdentifierStart = 0,
    Whitespace,
    LineBreak,
    Digit,
    Quote,
    Operator,
    Slash,
    Backtick,
    NonAscii,
    Other,
}

// 256-byte static bitmask lookup table for single-cycle character classification
pub static CHAR_FLAGS: [u8; 256] = {
    let mut flags = [0u8; 256];

    // Digits [0-9]
    let mut b = b'0';
    while b <= b'9' {
        flags[b as usize] = FLAG_DIGIT | FLAG_IDENT_CONTINUE;
        b += 1;
    }

    // Lowercase [a-z]
    let mut b = b'a';
    while b <= b'z' {
        flags[b as usize] = FLAG_IDENT_START | FLAG_IDENT_CONTINUE;
        b += 1;
    }

    // Uppercase [A-Z]
    let mut b = b'A';
    while b <= b'Z' {
        flags[b as usize] = FLAG_IDENT_START | FLAG_IDENT_CONTINUE;
        b += 1;
    }

    // Ident symbols '_' and '$'
    flags[b'_' as usize] = FLAG_IDENT_START | FLAG_IDENT_CONTINUE;
    flags[b'$' as usize] = FLAG_IDENT_START | FLAG_IDENT_CONTINUE;

    // Whitespace
    flags[b' ' as usize] = FLAG_WHITESPACE;
    flags[b'\t' as usize] = FLAG_WHITESPACE;

    // Line breaks
    flags[b'\n' as usize] = FLAG_LINE_BREAK;
    flags[b'\r' as usize] = FLAG_LINE_BREAK;

    // Quotes
    flags[b'"' as usize] = FLAG_QUOTE;
    flags[b'\'' as usize] = FLAG_QUOTE;

    // Operators
    let ops = [
        b'(', b')', b'{', b'}', b'[', b']', b';', b',', b'.', b':',
        b'?', b'=', b'!', b'+', b'-', b'*', b'%', b'&', b'|', b'^',
        b'~', b'<', b'>',
    ];
    let mut i = 0;
    while i < ops.len() {
        flags[ops[i] as usize] |= FLAG_OPERATOR;
        i += 1;
    }

    // Non-ASCII bytes (128..=255)
    let mut b = 128usize;
    while b < 256 {
        flags[b] = FLAG_NON_ASCII;
        b += 1;
    }

    flags
};

// Primary static character classification array
pub static CHAR_CLASS: [CharClass; 256] = {
    let mut table = [CharClass::Other; 256];

    // Identifier start (a-z, A-Z, _, $)
    let mut b = b'a';
    while b <= b'z' {
        table[b as usize] = CharClass::IdentifierStart;
        b += 1;
    }
    let mut b = b'A';
    while b <= b'Z' {
        table[b as usize] = CharClass::IdentifierStart;
        b += 1;
    }
    table[b'_' as usize] = CharClass::IdentifierStart;
    table[b'$' as usize] = CharClass::IdentifierStart;

    // Whitespace
    table[b' ' as usize] = CharClass::Whitespace;
    table[b'\t' as usize] = CharClass::Whitespace;

    // Line breaks
    table[b'\n' as usize] = CharClass::LineBreak;
    table[b'\r' as usize] = CharClass::LineBreak;

    // Digits
    let mut b = b'0';
    while b <= b'9' {
        table[b as usize] = CharClass::Digit;
        b += 1;
    }

    // Quotes
    table[b'"' as usize] = CharClass::Quote;
    table[b'\'' as usize] = CharClass::Quote;

    // Backtick & Slash
    table[b'`' as usize] = CharClass::Backtick;
    table[b'/' as usize] = CharClass::Slash;

    // Operators / Punctuators
    let ops = [
        b'(', b')', b'{', b'}', b'[', b']', b';', b',', b'.', b':',
        b'?', b'=', b'!', b'+', b'-', b'*', b'%', b'&', b'|', b'^',
        b'~', b'<', b'>',
    ];
    let mut i = 0;
    while i < ops.len() {
        table[ops[i] as usize] = CharClass::Operator;
        i += 1;
    }

    // Non-ASCII bytes
    let mut b = 128usize;
    while b < 256 {
        table[b] = CharClass::NonAscii;
        b += 1;
    }

    table
};

#[inline(always)]
pub fn is_ident_start(byte: u8) -> bool {
    (CHAR_FLAGS[byte as usize] & FLAG_IDENT_START) != 0
}

#[inline(always)]
pub fn is_ident_continue(byte: u8) -> bool {
    (CHAR_FLAGS[byte as usize] & FLAG_IDENT_CONTINUE) != 0
}

#[inline(always)]
pub fn is_whitespace(byte: u8) -> bool {
    (CHAR_FLAGS[byte as usize] & FLAG_WHITESPACE) != 0
}

#[inline(always)]
pub fn is_line_break(byte: u8) -> bool {
    (CHAR_FLAGS[byte as usize] & FLAG_LINE_BREAK) != 0
}

#[inline(always)]
pub fn is_digit(byte: u8) -> bool {
    (CHAR_FLAGS[byte as usize] & FLAG_DIGIT) != 0
}

#[inline(always)]
pub fn classify_ascii(byte: u8) -> CharClass {
    CHAR_CLASS[byte as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ident_classification() {
        assert!(is_ident_start(b'a'));
        assert!(is_ident_start(b'Z'));
        assert!(is_ident_start(b'_'));
        assert!(is_ident_start(b'$'));
        assert!(!is_ident_start(b'0'));
        assert!(!is_ident_start(b' '));

        assert!(is_ident_continue(b'a'));
        assert!(is_ident_continue(b'9'));
        assert!(is_ident_continue(b'_'));
        assert!(!is_ident_continue(b'+'));
    }

    #[test]
    fn test_whitespace_and_line_break() {
        assert!(is_whitespace(b' '));
        assert!(is_whitespace(b'\t'));
        assert!(!is_whitespace(b'\n'));

        assert!(is_line_break(b'\n'));
        assert!(is_line_break(b'\r'));
    }
}

