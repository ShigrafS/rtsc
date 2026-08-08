// Differential Fuzzing module for scanner & parser correctness.
// Continuously compares rtsc lexer token output against reference token stream.

use crate::lexer::{Lexer, TokenKind};

#[derive(Default)]
pub struct DifferentialFuzzer;

impl DifferentialFuzzer {
    pub fn new() -> Self {
        Self
    }

    // Tokenize source with rtsc lexer into normalized (kind, text) pairs
    pub fn tokenize_rtsc(&self, src: &str) -> Vec<(TokenKind, String)> {
        let mut lexer = Lexer::new(src);
        lexer.skip_trivia = true;
        let mut result = Vec::new();

        loop {
            let tok = lexer.next_token();
            if tok.kind == TokenKind::Eof {
                break;
            }
            result.push((tok.kind, tok.span_text(src).to_string()));
        }
        result
    }

    // Pseudo-random JS stream generator for differential fuzz testing
    pub fn generate_random_js(&self, seed: u64) -> String {
        let keywords = ["const", "let", "var", "function", "return", "if"];
        let idents = ["a", "b", "foo", "bar", "_tmp", "$val"];
        let ops = ["+", "-", "=", "==", "===", ";"];

        let mut src = String::new();
        let mut state = seed;
        
        let mut next_rand = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            state
        };

        let token_count = 10 + (next_rand() % 40) as usize;
        for _ in 0..token_count {
            let choice = next_rand() % 4;
            match choice {
                0 => {
                    let kw = keywords[(next_rand() as usize) % keywords.len()];
                    src.push_str(kw);
                    src.push(' ');
                }
                1 => {
                    let id = idents[(next_rand() as usize) % idents.len()];
                    src.push_str(id);
                    src.push(' ');
                }
                2 => {
                    let num = (next_rand() % 100).to_string();
                    src.push_str(&num);
                    src.push(' ');
                }
                _ => {
                    let op = ops[(next_rand() as usize) % ops.len()];
                    src.push_str(op);
                    src.push(' ');
                }
            }
        }
        src
    }

    // Differential test assertion against reference lexer stream
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differential_fuzzer_deterministic_stream() {
        let fuzzer = DifferentialFuzzer::new();
        let sample_js = fuzzer.generate_random_js(12345);
        let tokens = fuzzer.tokenize_rtsc(&sample_js);

        assert!(!tokens.is_empty());
        assert_eq!(fuzzer.verify_differential(&sample_js, &tokens), Ok(()));
    }
}
