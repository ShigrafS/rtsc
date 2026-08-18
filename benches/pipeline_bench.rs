use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rtsc::ast::AstArena;
use rtsc::binder::Binder;
use rtsc::checker::TypeChecker;
use rtsc::emitter::Emitter;
use rtsc::interner::StringInterner;
use rtsc::lexer::{Lexer, Token, TokenKind};
use rtsc::parser::Parser;

// Helper to pre-lex tokens for parser/binder/checker phase benchmarks
fn tokenize_all(src: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(src);
    lexer.skip_trivia = true;
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        if tok.kind == TokenKind::Eof {
            break;
        }
        tokens.push(tok);
    }
    tokens
}

fn bench_pipeline_phases(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiler_pipeline_phases");

    // Workload 1: Medium sized script
    let mut medium_src = String::with_capacity(4096);
    for i in 0..50 {
        medium_src.push_str(&format!("const a_{} = {}; const b_{} = a_{} + 10;\n", i, i * 2, i, i));
    }
    group.throughput(Throughput::Bytes(medium_src.len() as u64));

    // 1. Lexing phase benchmark
    group.bench_function("1_lexing", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&medium_src));
            lexer.skip_trivia = true;
            let mut count = 0;
            loop {
                let tok = lexer.next_token();
                if tok.kind == TokenKind::Eof {
                    break;
                }
                count += 1;
            }
            count
        });
    });

    // Pre-tokenize for isolated downstream phase benchmarking
    let tokens = tokenize_all(&medium_src);

    // 2. Parsing phase benchmark
    group.bench_function("2_parsing", |b| {
        b.iter(|| {
            let mut interner = StringInterner::new();
            let mut arena = AstArena::new();
            let mut parser = Parser::new(black_box(&tokens), black_box(&medium_src));
            parser.parse_program(&mut interner, &mut arena)
        });
    });

    // Pre-parse for isolated binding / checking / emitter benchmarks
    let mut interner_base = StringInterner::new();
    let mut arena_base = AstArena::new();
    let mut parser = Parser::new(&tokens, &medium_src);
    let root = parser.parse_program(&mut interner_base, &mut arena_base);

    // 3. Binding phase benchmark
    group.bench_function("3_binding", |b| {
        b.iter(|| {
            let mut binder = Binder::new();
            binder.bind_program(black_box(root), black_box(&arena_base));
            binder
        });
    });

    // 4. Type checking phase benchmark
    group.bench_function("4_typecheck", |b| {
        b.iter(|| {
            let mut checker = TypeChecker::new();
            checker.check_node(black_box(root), black_box(&arena_base))
        });
    });

    // 5. Code emission phase benchmark
    group.bench_function("5_emit", |b| {
        b.iter(|| {
            let mut emitter = Emitter::new();
            emitter.emit_program(black_box(root), black_box(&arena_base), black_box(&interner_base))
        });
    });

    // 6. Full clean compile (all phases combined)
    group.bench_function("6_full_clean_build", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&medium_src));
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
            let mut parser = Parser::new(&tokens, &medium_src);
            let root = parser.parse_program(&mut interner, &mut arena);

            let mut binder = Binder::new();
            binder.bind_program(root, &arena);

            let mut checker = TypeChecker::new();
            checker.check_node(root, &arena);

            let mut emitter = Emitter::new();
            emitter.emit_program(root, &arena, &interner)
        });
    });

    group.finish();
}

fn bench_pipeline_scales(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiler_workload_scales");

    let tiny_src = "const x = 1;".to_string();
    
    let mut medium_src = String::with_capacity(4096);
    for i in 0..50 {
        medium_src.push_str(&format!("const a_{} = {}; const b_{} = a_{} + 10;\n", i, i * 2, i, i));
    }

    let mut huge_src = String::with_capacity(65536);
    for i in 0..1000 {
        huge_src.push_str(&format!("const item_{} = {} + {};\n", i, i, i * 3));
    }

    let workloads = [
        ("tiny", tiny_src),
        ("medium", medium_src),
        ("huge_1k_stmts", huge_src),
    ];

    for (name, src) in workloads {
        group.throughput(Throughput::Bytes(src.len() as u64));
        group.bench_with_input(BenchmarkId::new("full_build", name), &src, |b, input| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(input));
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
                let mut parser = Parser::new(&tokens, input);
                let root = parser.parse_program(&mut interner, &mut arena);

                let mut binder = Binder::new();
                binder.bind_program(root, &arena);

                let mut checker = TypeChecker::new();
                checker.check_node(root, &arena);

                let mut emitter = Emitter::new();
                emitter.emit_program(root, &arena, &interner)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_pipeline_phases, bench_pipeline_scales);
criterion_main!(benches);
