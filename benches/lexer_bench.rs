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

fn bench_character_classification(c: &mut Criterion) {
    let mut group = c.benchmark_group("char_classification_methods");

    let sample_bytes: Vec<u8> = (0..=255u8).cycle().take(4096).collect();
    group.throughput(Throughput::Bytes(sample_bytes.len() as u64));

    // 1. Range check pattern matching
    group.bench_function("range_check_match", |b| {
        b.iter(|| {
            let mut count = 0;
            for &byte in black_box(&sample_bytes) {
                if matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$') {
                    count += 1;
                }
            }
            count
        });
    });

    // 2. Table-driven bitmask classification
    group.bench_function("table_bitmask_lookup", |b| {
        b.iter(|| {
            let mut count = 0;
            for &byte in black_box(&sample_bytes) {
                if rtsc::lexer::char_class::is_ident_continue(byte) {
                    count += 1;
                }
            }
            count
        });
    });

    group.finish();
}

fn bench_keyword_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyword_recognition");

    let identifiers = [
        "const", "let", "return", "function", "if", "else", "class", "interface",
        "type", "import", "export", "async", "await", "this", "true", "false",
        "foo", "bar", "handleClick", "data", "renderComponent", "options",
        "calculateSum", "variableName", "response", "element", "isActive",
    ];

    let total_bytes: usize = identifiers.iter().map(|s| s.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("stratified_keyword_lookup", |b| {
        b.iter(|| {
            let mut keywords = 0;
            for &ident in black_box(&identifiers) {
                let kind = rtsc::lexer::keyword::lookup_keyword(ident.as_bytes());
                if kind.is_keyword() {
                    keywords += 1;
                }
            }
            keywords
        });
    });

    group.finish();
}

fn bench_token_throughput_rates(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer_throughput");

    // Generate ~1 MB realistic TypeScript corpus
    let mut large_src = String::with_capacity(1_000_000);
    let stubs = [
        "export interface UserProfile {\n  id: number;\n  name: string;\n  isActive?: boolean;\n}\n",
        "export async function fetchUserData(userId: number): Promise<UserProfile> {\n  const res = await api.get(`/users/${userId}`);\n  return res.data;\n}\n",
        "const DEFAULT_CONFIG = { retries: 3, timeout: 5000, debug: false };\n",
        "// Fast path evaluation\nif (flags & 0xFF) { const count = calc(42, 100n); }\n",
    ];

    while large_src.len() < 500_000 {
        for stub in stubs {
            large_src.push_str(stub);
        }
    }

    group.throughput(Throughput::Bytes(large_src.len() as u64));
    group.bench_function("synthetic_corpus_500kb", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&large_src));
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

    group.finish();
}

criterion_group!(
    benches,
    bench_lexer_workloads,
    bench_character_classification,
    bench_keyword_lookup,
    bench_token_throughput_rates
);
criterion_main!(benches);



