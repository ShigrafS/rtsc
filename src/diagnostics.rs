// Zero-cost cold diagnostic reporting and error emission infrastructure.
// Keeps hot paths free of formatting and allocations; renders rich compiler diagnostics on demand.

use crate::checker::TypeId;
use crate::interner::{NameId, StringInterner};
use crate::span::{LineStarts, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    Error = 0,
    Warning = 1,
    Info = 2,
    Hint = 3,
}

impl DiagnosticSeverity {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Hint => "hint",
        }
    }
}

// Structured diagnostic payload kept compact during compilation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    UnexpectedToken { expected: &'static str, found: &'static str },
    UnterminatedString,
    UnterminatedComment,
    UnterminatedTemplate,
    CannotFindName(NameId),
    TypeMismatch { expected: TypeId, found: TypeId },
    SyntaxError(&'static str),
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: u16,
    pub severity: DiagnosticSeverity,
    pub span: Span,
    pub kind: DiagnosticKind,
}

impl Diagnostic {
    pub fn new(code: u16, severity: DiagnosticSeverity, span: Span, kind: DiagnosticKind) -> Self {
        Self {
            code,
            severity,
            span,
            kind,
        }
    }

    pub fn error(code: u16, span: Span, kind: DiagnosticKind) -> Self {
        Self::new(code, DiagnosticSeverity::Error, span, kind)
    }

    pub fn warning(code: u16, span: Span, kind: DiagnosticKind) -> Self {
        Self::new(code, DiagnosticSeverity::Warning, span, kind)
    }

    // Resolves diagnostic message string using optional interner for interned names
    pub fn message(&self, interner: Option<&StringInterner>) -> String {
        match &self.kind {
            DiagnosticKind::UnexpectedToken { expected, found } => {
                format!("expected '{}', found '{}'", expected, found)
            }
            DiagnosticKind::UnterminatedString => "unterminated string literal".to_string(),
            DiagnosticKind::UnterminatedComment => "unterminated multi-line comment".to_string(),
            DiagnosticKind::UnterminatedTemplate => "unterminated template literal".to_string(),
            DiagnosticKind::CannotFindName(name_id) => {
                if let Some(interner) = interner {
                    let name = interner.resolve(*name_id).unwrap_or("<unknown>");
                    format!("cannot find name '{}'", name)
                } else {
                    format!("cannot find name (id: {:?})", name_id)
                }
            }
            DiagnosticKind::TypeMismatch { expected, found } => {
                format!(
                    "type mismatch: expected TypeId({}), found TypeId({})",
                    expected.0, found.0
                )
            }
            DiagnosticKind::SyntaxError(msg) => msg.to_string(),
            DiagnosticKind::Custom(msg) => msg.clone(),
        }
    }
}

// Collector for diagnostics produced across scanner, parser, binder, and checker
#[derive(Debug, Default, Clone)]
pub struct DiagnosticsBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticsBag {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::with_capacity(16),
        }
    }

    #[inline]
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    #[inline]
    pub fn error(&mut self, code: u16, span: Span, kind: DiagnosticKind) {
        self.add(Diagnostic::error(code, span, kind));
    }

    #[inline]
    pub fn warning(&mut self, code: u16, span: Span, kind: DiagnosticKind) {
        self.add(Diagnostic::warning(code, span, kind));
    }

    #[inline]
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.diagnostics.iter()
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

// Pretty-printer for diagnostics with source line snippet and carets
pub struct DiagnosticFormatter;

impl DiagnosticFormatter {
    pub fn format(
        diag: &Diagnostic,
        src: &str,
        file_name: &str,
        interner: Option<&StringInterner>,
    ) -> String {
        let line_starts = LineStarts::new(src);
        let (line, col) = line_starts.location(diag.span.start);
        let message = diag.message(interner);

        // Find source line bounds
        let lines: Vec<&str> = src.lines().collect();
        let line_text = if line > 0 && (line as usize) <= lines.len() {
            lines[line as usize - 1]
        } else {
            ""
        };

        let col_idx = col.saturating_sub(1) as usize;
        let underline_len = (diag.span.end.saturating_sub(diag.span.start)).max(1) as usize;
        let caret_count = underline_len.min(line_text.len().saturating_sub(col_idx).max(1));

        let indent = " ".repeat(col_idx);
        let carets = "^".repeat(caret_count);
        let line_num_str = format!("{}", line);
        let padding = " ".repeat(line_num_str.len());

        format!(
            "{}[TS{}]: {}\n  --> {}:{}:{}\n   {} |\n{} | {}\n   {} | {}{}\n",
            diag.severity.name(),
            diag.code,
            message,
            file_name,
            line,
            col,
            padding,
            line_num_str,
            line_text,
            padding,
            indent,
            carets
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_bag_collection() {
        let mut bag = DiagnosticsBag::new();
        assert!(!bag.has_errors());

        bag.warning(1100, Span::new(0, 5), DiagnosticKind::UnterminatedComment);
        assert!(!bag.has_errors());
        assert_eq!(bag.len(), 1);

        bag.error(1005, Span::new(6, 10), DiagnosticKind::UnterminatedString);
        assert!(bag.has_errors());
        assert_eq!(bag.len(), 2);
    }

    #[test]
    fn test_diagnostic_formatting_output() {
        let src = "const x: number = \"hello\";\nconst y = 20;";
        let diag = Diagnostic::error(
            2322,
            Span::new(18, 25), // "hello"
            DiagnosticKind::TypeMismatch {
                expected: TypeId::NUMBER,
                found: TypeId::STRING,
            },
        );

        let formatted = DiagnosticFormatter::format(&diag, src, "src/index.ts", None);
        assert!(formatted.contains("error[TS2322]"));
        assert!(formatted.contains("--> src/index.ts:1:19"));
        assert!(formatted.contains("const x: number = \"hello\";"));
        assert!(formatted.contains("^^^^^^^"));
    }
}
