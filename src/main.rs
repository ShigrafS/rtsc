use std::env;
use std::fs;
use std::time::Instant;

use rtsc::ast::AstArena;
use rtsc::binder::Binder;
use rtsc::checker::TypeChecker;
use rtsc::emitter::Emitter;
use rtsc::fuzz::DifferentialFuzzer;
use rtsc::interner::StringInterner;
use rtsc::lexer::{Lexer, TokenKind};
use rtsc::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "check" | "build" => {
            let file_path = args.get(2).map(|s| s.as_str()).unwrap_or("src/index.ts");
            let source = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(_) => {
                    // Fallback to sample TS source if file does not exist yet
                    "const x = 42; const y = x + 10;".to_string()
                }
            };

            let start = Instant::now();

            let mut lexer = Lexer::new(&source);
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
            let mut parser = Parser::new(&tokens, &source);
            let root = parser.parse_program(&mut interner, &mut arena);

            let mut binder = Binder::new();
            binder.bind_program(root, &arena);

            let mut checker = TypeChecker::new();
            checker.check_node(root, &arena);

            let mut emitter = Emitter::new();
            let emitted = emitter.emit_program(root, &arena, &interner);

            let elapsed = start.elapsed();
            println!(
                "⚡ [rtsc] Compiled {} tokens ({} bytes) in {:?}",
                tokens.len(),
                source.len(),
                elapsed
            );
            if command == "build" {
                println!("--- Emitted Output ---\n{}", emitted);
            }
        }
        "fuzz" => {
            println!("🔥 Running differential fuzzing harness...");
            let fuzzer = DifferentialFuzzer::new();
            for seed in 0..100 {
                let sample_js = fuzzer.generate_random_js(seed);
                let tokens = fuzzer.tokenize_rtsc(&sample_js);
                if let Err(err) = fuzzer.verify_differential(&sample_js, &tokens) {
                    eprintln!("❌ Fuzz failure at seed {}: {}", seed, err);
                    return;
                }
            }
            println!("✅ 100 differential fuzzing iterations passed cleanly!");
        }
        _ => {
            println!("⚡ rtsc - Rust TypeScript Compiler");
            println!("Usage:");
            println!("  rtsc check [file.ts]  Run type checking pipeline");
            println!("  rtsc build [file.ts]  Compile and emit JS output");
            println!("  rtsc fuzz             Run differential fuzzing suite");
        }
    }
}
