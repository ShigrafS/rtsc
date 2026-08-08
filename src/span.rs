// Compact 8-byte source span for AST nodes and tokens.
// Avoid storing full file paths or line/col numbers directly on hot AST nodes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

// Line start offsets table for binary search location lookups on diagnostics path.
#[derive(Debug, Clone, Default)]
pub struct LineStarts {
    starts: Vec<u32>,
}

impl LineStarts {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0];
        for (i, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push((i + 1) as u32);
            }
        }
        Self { starts }
    }

    // Binary search to convert source byte offset to 1-based (line, col)
    pub fn location(&self, pos: u32) -> (u32, u32) {
        match self.starts.binary_search(&pos) {
            Ok(idx) => ((idx + 1) as u32, 1),
            Err(idx) => {
                let line_idx = idx.saturating_sub(1);
                let line_start = self.starts.get(line_idx).copied().unwrap_or(0);
                let col = pos.saturating_sub(line_start) + 1;
                ((line_idx + 1) as u32, col)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_size() {
        assert_eq!(std::mem::size_of::<Span>(), 8);
    }

    #[test]
    fn test_line_starts_lookup() {
        let src = "const x = 1;\nconst y = 2;\nconsole.log(x);";
        let table = LineStarts::new(src);
        // Line 1: 'c' at 0
        assert_eq!(table.location(0), (1, 1));
        // Line 2: 'c' at 13
        assert_eq!(table.location(13), (2, 1));
        // Line 3: 'c' at 26
        assert_eq!(table.location(26), (3, 1));
    }
}
