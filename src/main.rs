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
            let iters: usize = args
                .get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(100);

            println!("🔥 Running differential fuzzing & invariant testbed ({} iterations)...", iters);
            let fuzzer = DifferentialFuzzer::new();
            match fuzzer.run_fuzz_suite(iters, 1234567) {
                Ok(summary) => {
                    println!("✅ All {} fuzzing iterations passed cleanly in {:?}!", summary.iterations, summary.duration);
                    println!(
                        "   Verified {} tokens across {} bytes ({:.1} MB/s throughput)",
                        summary.total_tokens,
                        summary.total_bytes,
                        (summary.total_bytes as f64 / (1024.0 * 1024.0)) / summary.duration.as_secs_f64().max(0.000001)
                    );
                }
                Err(err) => {
                    eprintln!("❌ Fuzz failure detected: {}", err);
                    std::process::exit(1);
                }
            }
        }
        "profile" => {
            let file_path = args.get(2).map(|s| s.as_str()).unwrap_or("src/index.ts");
            let source = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(_) => {
                    // Fallback to synthetic multi-statement TS source for profiling demo
                    let mut s = String::new();
                    for i in 0..500 {
                        s.push_str(&format!("const item_{} = {} + {};\n", i, i, i * 2));
                    }
                    s
                }
            };
            println!("📊 Profiling compiler pipeline across {} bytes of TypeScript...", source.len());
            let (metrics, _) = rtsc::telemetry::profile_compiler_pipeline(&source);
            metrics.print_dashboard();
        }
        _ => {
            println!("⚡ rtsc - Rust TypeScript Compiler");
            println!("Usage:");
            println!("  rtsc check [file.ts]    Run type checking pipeline");
            println!("  rtsc build [file.ts]    Compile and emit JS output");
            println!("  rtsc profile [file.ts]  Profile all pipeline stages and memory usage");
            println!("  rtsc fuzz               Run differential fuzzing suite");
        }
    }
}
