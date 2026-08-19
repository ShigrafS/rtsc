// Differential Fuzzing and Invariant Testbed module for scanner & parser correctness.
// Validates span monotonicity, lossless string reconstruction, AST arena integrity,
// and differential reference equivalence across random JS/TS streams and arbitrary bytes.

use std::time::{Duration, Instant};
use crate::ast::AstArena;
use crate::interner::StringInterner;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::parser::Parser;

// Fast, deterministic pseudo-random number generator for reproducible fuzz test runs
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        // SplitMix64 PRNG
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    #[inline]
    pub fn next_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        let range = max - min;
        min + (self.next_u64() as usize % range)
    }

    #[inline]
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        &slice[self.next_range(0, slice.len())]
    }
}

#[derive(Debug, Clone, Default)]
pub struct FuzzSummary {
    pub iterations: usize,
    pub total_bytes: usize,
    pub total_tokens: usize,
    pub duration: Duration,
}

#[derive(Default)]
pub struct DifferentialFuzzer;

impl DifferentialFuzzer {
    pub fn new() -> Self {
        Self
    }

    // Tokenize source into compact Token vector (with option to preserve trivia)
    pub fn tokenize_all(&self, src: &str, skip_trivia: bool) -> Vec<Token> {
        let mut lexer = Lexer::new(src);
        lexer.skip_trivia = skip_trivia;
        let mut tokens = Vec::with_capacity(128);

        loop {
            let tok = lexer.next_token();
            if tok.kind == TokenKind::Eof {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    // Tokenize source with rtsc lexer into normalized (kind, text) pairs
    pub fn tokenize_rtsc(&self, src: &str) -> Vec<(TokenKind, String)> {
        let tokens = self.tokenize_all(src, true);
        tokens
            .into_iter()
            .map(|tok| (tok.kind, tok.span_text(src).to_string()))
            .collect()
    }

    // Generates a rich, pseudo-random TypeScript/JavaScript source text
    pub fn generate_random_js(&self, seed: u64) -> String {
        let mut rng = DeterministicRng::new(seed);
        let keywords = [
            "const", "let", "var", "function", "return", "if", "else",
            "class", "interface", "type", "enum", "async", "await",
            "import", "export", "true", "false", "null", "undefined",
        ];
        let idents = [
            "x", "y", "foo", "bar", "_temp", "$val", "result2",
            "MyClass", "calculate_total", "foo_µ", "_ಠ_ಠ",
        ];
        let numbers = [
            "0", "1", "42", "3.14159", "1000", "0.5", "1e5", "0xFF", "0b101",
        ];
        let strings = [
            "\"hello\"", "'world'", "\"escaped \\\" quote\"", "'nested \\' quote'",
            "`template string`", "\"multiline\\nline\"",
        ];
        let operators = [
            "+", "-", "*", "/", "%", "=", "==", "===", "!=", "!==",
            "+=", "-=", "++", "--", "&&", "||", "=>", "?.", "...", ";",
            "(", ")", "{", "}", "[", "]", ",", ":",
        ];
        let comments = [
            "// single line comment\n",
            "/* inline multi-line comment */ ",
            "// trailing note\n",
            "/** jsdoc styled comment */ ",
        ];
        let whitespace = [" ", "\t", "\n", "\r\n", "  "];

        let token_count = rng.next_range(15, 60);
        let mut src = String::with_capacity(token_count * 8);

        for _ in 0..token_count {
            let kind_choice = rng.next_range(0, 10);
            match kind_choice {
                0..=2 => {
                    src.push_str(rng.choose(&keywords));
                }
                3..=4 => {
                    src.push_str(rng.choose(&idents));
                }
                5 => {
                    src.push_str(rng.choose(&numbers));
                }
                6 => {
                    src.push_str(rng.choose(&strings));
                }
                7 => {
                    src.push_str(rng.choose(&comments));
                }
                _ => {
                    src.push_str(rng.choose(&operators));
                }
            }
            src.push_str(rng.choose(&whitespace));
        }

        src
    }

    // Generates arbitrary, potentially invalid UTF-8 byte streams to verify lexer robustness
    pub fn generate_arbitrary_bytes(&self, seed: u64, length: usize) -> Vec<u8> {
        let mut rng = DeterministicRng::new(seed);
        let mut bytes = Vec::with_capacity(length);
        for _ in 0..length {
            bytes.push(rng.next_u64() as u8);
        }
        bytes
    }

    // Invariant 1: Token span monotonicity and boundary containment
    pub fn verify_span_invariants(&self, src: &str, tokens: &[Token]) -> Result<(), String> {
        let mut last_end = 0u32;
        let src_len = src.len() as u32;

        for (idx, tok) in tokens.iter().enumerate() {
            if tok.start > tok.end {
                return Err(format!(
                    "Token #{} span inverted: start={} > end={}",
                    idx, tok.start, tok.end
                ));
            }
            if tok.end > src_len {
                return Err(format!(
                    "Token #{} span exceeds source bounds: end={} > src_len={}",
                    idx, tok.end, src_len
                ));
            }
            if tok.start < last_end {
                return Err(format!(
                    "Token #{} overlaps with preceding token: start={} < last_end={}",
                    idx, tok.start, last_end
                ));
            }
            last_end = tok.end;
        }

        Ok(())
    }

    // Invariant 2: Lossless source reconstruction from all non-trivia and trivia tokens
    pub fn verify_reconstruction(&self, src: &str) -> Result<(), String> {
        let tokens = self.tokenize_all(src, false);
        self.verify_span_invariants(src, &tokens)?;

        let mut reconstructed = String::with_capacity(src.len());
        for tok in tokens {
            reconstructed.push_str(tok.span_text(src));
        }

        if reconstructed != src {
            return Err(format!(
                "Reconstruction failed! Original length: {}, Reconstructed length: {}",
                src.len(),
                reconstructed.len()
            ));
        }

        Ok(())
    }

    // Invariant 3: Parser arena safety on arbitrary fuzzed token stream
    pub fn verify_parser_safety(&self, src: &str) -> Result<(), String> {
        let tokens = self.tokenize_all(src, true);
        let mut interner = StringInterner::new();
        let mut arena = AstArena::new();
        let mut parser = Parser::new(&tokens, src);

        let root = parser.parse_program(&mut interner, &mut arena);
        if arena.get_hot(root).kind != crate::ast::NodeKind::Program {
            return Err("Parser root node kind is not Program".to_string());
        }

        Ok(())
    }

    // Invariant 4: Differential assertion against reference tokens
    pub fn verify_differential(&self, src: &str, reference: &[(TokenKind, String)]) -> Result<(), String> {
        let actual = self.tokenize_rtsc(src);
        if actual.len() != reference.len() {
            return Err(format!(
                "Token count mismatch: actual={}, reference={}",
                actual.len(),
                reference.len()
            ));
        }

        for (i, (act, ref_tok)) in actual.iter().zip(reference.iter()).enumerate() {
            if act.0 != ref_tok.0 || act.1 != ref_tok.1 {
                return Err(format!(
                    "Mismatch at index {}: actual=({:?}, {:?}), reference=({:?}, {:?})",
                    i, act.0, act.1, ref_tok.0, ref_tok.1
                ));
            }
        }
        Ok(())
    }

    // Runs a batch fuzzing suite verifying all invariants across iterations
    pub fn run_fuzz_suite(&self, iterations: usize, seed_base: u64) -> Result<FuzzSummary, String> {
        let start = Instant::now();
        let mut total_bytes = 0;
        let mut total_tokens = 0;

        for i in 0..iterations {
            let seed = seed_base.wrapping_add(i as u64 * 7919);
            let sample_js = self.generate_random_js(seed);
            total_bytes += sample_js.len();

            // Invariant: Lossless full reconstruction (trivia included)
            self.verify_reconstruction(&sample_js)
                .map_err(|e| format!("Reconstruction invariant failed (seed: {}): {}", seed, e))?;

            // Invariant: Span monotonicity and bounds (skip trivia)
            let tokens = self.tokenize_all(&sample_js, true);
            total_tokens += tokens.len();
            self.verify_span_invariants(&sample_js, &tokens)
                .map_err(|e| format!("Span invariant failed (seed: {}): {}", seed, e))?;

            // Invariant: Parser safety
            self.verify_parser_safety(&sample_js)
                .map_err(|e| format!("Parser invariant failed (seed: {}): {}", seed, e))?;

            // Stress test: Arbitrary lossy string parsing
            let raw_bytes = self.generate_arbitrary_bytes(seed, 256);
            let lossy_str = String::from_utf8_lossy(&raw_bytes);
            let _ = self.tokenize_all(&lossy_str, false);
        }

        Ok(FuzzSummary {
            iterations,
            total_bytes,
            total_tokens,
            duration: start.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzz_deterministic_reproducibility() {
        let fuzzer = DifferentialFuzzer::new();
        let js1 = fuzzer.generate_random_js(42);
        let js2 = fuzzer.generate_random_js(42);
        assert_eq!(js1, js2);
    }

    #[test]
    fn test_fuzz_reconstruction_invariant() {
        let fuzzer = DifferentialFuzzer::new();
        for seed in 1..=25 {
            let sample = fuzzer.generate_random_js(seed);
            assert_eq!(fuzzer.verify_reconstruction(&sample), Ok(()));
        }
    }

    #[test]
    fn test_fuzz_arbitrary_bytes_no_panic() {
        let fuzzer = DifferentialFuzzer::new();
        for seed in 100..120 {
            let raw_bytes = fuzzer.generate_arbitrary_bytes(seed, 512);
            let lossy_str = String::from_utf8_lossy(&raw_bytes);
            let tokens = fuzzer.tokenize_all(&lossy_str, false);
            assert_eq!(fuzzer.verify_span_invariants(&lossy_str, &tokens), Ok(()));
        }
    }

    #[test]
    fn test_fuzz_suite_execution() {
        let fuzzer = DifferentialFuzzer::new();
        let summary = fuzzer.run_fuzz_suite(50, 9999).expect("Fuzz suite passed");
        assert_eq!(summary.iterations, 50);
        assert!(summary.total_bytes > 0);
        assert!(summary.total_tokens > 0);
    }
}

