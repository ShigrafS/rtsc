// Table-driven character classification array for fast-path dispatch.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CharClass {
    Whitespace,
    LineBreak,
    IdentifierStart,
    Digit,
    Quote,
    Operator,
    Slash,
    Backtick,
    Other,
}

pub static CHAR_CLASS: [CharClass; 256] = {
    let mut table = [CharClass::Other; 256];
    
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
    
    // Quotes
    table[b'"' as usize] = CharClass::Quote;
    table[b'\'' as usize] = CharClass::Quote;
    
    // Backtick
    table[b'`' as usize] = CharClass::Backtick;
    
    // Slash
    table[b'/' as usize] = CharClass::Slash;
    
    // Operators / Punctuators
    table[b'(' as usize] = CharClass::Operator;
    table[b')' as usize] = CharClass::Operator;
    table[b'{' as usize] = CharClass::Operator;
    table[b'}' as usize] = CharClass::Operator;
    table[b'[' as usize] = CharClass::Operator;
    table[b']' as usize] = CharClass::Operator;
    table[b';' as usize] = CharClass::Operator;
    table[b',' as usize] = CharClass::Operator;
    table[b'.' as usize] = CharClass::Operator;
    table[b':' as usize] = CharClass::Operator;
    table[b'?' as usize] = CharClass::Operator;
    table[b'=' as usize] = CharClass::Operator;
    table[b'!' as usize] = CharClass::Operator;
    table[b'+' as usize] = CharClass::Operator;
    table[b'-' as usize] = CharClass::Operator;
    table[b'*' as usize] = CharClass::Operator;
    table[b'%' as usize] = CharClass::Operator;
    table[b'&' as usize] = CharClass::Operator;
    table[b'|' as usize] = CharClass::Operator;
    table[b'^' as usize] = CharClass::Operator;
    table[b'~' as usize] = CharClass::Operator;
    table[b'<' as usize] = CharClass::Operator;
    table[b'>' as usize] = CharClass::Operator;
    
    table
};

#[inline(always)]
pub fn classify_ascii(byte: u8) -> CharClass {
    CHAR_CLASS[byte as usize]
}
