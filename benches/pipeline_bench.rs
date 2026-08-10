use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use rtsc::ast::AstArena;
use rtsc::binder::Binder;
use rtsc::checker::TypeChecker;
use rtsc::emitter::Emitter;
use rtsc::interner::StringInterner;
use rtsc::lexer::{Lexer, TokenKind};
use rtsc::parser::Parser;

fn bench_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("compiler_pipeline");

    let source = "const x = 10; const y = 20; const z = x + y;";
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("full_clean_build", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
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
            let mut parser = Parser::new(&tokens, source);
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

criterion_group!(benches, bench_pipeline);
criterion_main!(benches);
