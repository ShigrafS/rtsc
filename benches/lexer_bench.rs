use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rtsc::lexer::{Lexer, TokenKind};

fn bench_lexer_workloads(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");

    // 1. Tiny file (startup overhead)
    let tiny_src = "const x = 1;";
    group.throughput(Throughput::Bytes(tiny_src.len() as u64));
    group.bench_function("tiny_file", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(tiny_src));
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

    // 2. Huge generated JavaScript file
    let mut huge_src = String::with_capacity(100_000);
    for i in 0..2000 {
        huge_src.push_str(&format!("const var_{} = {} + {};\n", i, i, i * 2));
    }
    group.throughput(Throughput::Bytes(huge_src.len() as u64));
    group.bench_function("huge_generated", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&huge_src));
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

    // 3. Real-world project corpus stubs (React, TS, VSCode, Next.js, Angular, Node, Deno)
    let real_world_cases = [
        ("React_Component", "import React from 'react'; export const Button = ({ label }) => <button>{label}</button>;"),
        ("TypeScript_Compiler", "interface CompilerOptions { target?: string; module?: string; strict?: boolean; }"),
        ("VSCode_Editor", "export class TextEditor { private doc: string; constructor(doc: string) { self.doc = doc; } }"),
        ("NextJS_App", "export default function Page({ data }) { return <div>{data.title}</div>; }"),
        ("Angular_Module", "@Component({ selector: 'app-root', template: '<h1>App</h1>' }) export class AppComponent {}"),
        ("Node_Server", "const http = require('http'); http.createServer((req, res) => res.end('OK')).listen(8080);"),
        ("Deno_Runtime", "import { serve } from 'https://deno.land/std/http/server.ts'; serve((_req) => new Response('OK'));"),
    ];

    for (name, sample) in real_world_cases {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_with_input(BenchmarkId::new("real_world", name), sample, |b, src| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(src));
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
    }

    // 4. Pathological lexical cases
    let pathological_ident = "a".repeat(10_000);
    group.throughput(Throughput::Bytes(pathological_ident.len() as u64));
    group.bench_function("pathological_identifier", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&pathological_ident));
            lexer.next_token()
        });
    });

    group.finish();
}

criterion_group!(benches, bench_lexer_workloads);
criterion_main!(benches);
