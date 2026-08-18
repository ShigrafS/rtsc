// Telemetry and profiling harness for hardware metrics and per-phase compiler instrumentation.
// Tracks execution time, allocations, throughput (MB/s), and peak process RSS.

use std::time::{Duration, Instant};
use crate::ast::AstArena;
use crate::binder::Binder;
use crate::checker::TypeChecker;
use crate::emitter::Emitter;
use crate::interner::StringInterner;
use crate::lexer::{Lexer, TokenKind};
use crate::parser::Parser;

#[derive(Debug, Clone, Default)]
pub struct ProfileMetrics {
    pub source_bytes: usize,
    pub lex_duration: Duration,
    pub token_count: usize,
    pub parse_duration: Duration,
    pub node_count: usize,
    pub bind_duration: Duration,
    pub symbol_count: usize,
    pub scope_count: usize,
    pub check_duration: Duration,
    pub emit_duration: Duration,
    pub emit_bytes: usize,
    pub total_duration: Duration,
    pub peak_rss_bytes: usize,
}

impl ProfileMetrics {
    // Calculates phase percentage of total build time
    fn pct(&self, d: Duration) -> f64 {
        if self.total_duration.as_nanos() == 0 {
            0.0
        } else {
            (d.as_nanos() as f64 / self.total_duration.as_nanos() as f64) * 100.0
        }
    }

    // Calculates MB/s throughput for a given duration
    fn mb_per_sec(&self, d: Duration) -> f64 {
        let secs = d.as_secs_f64();
        if secs <= 0.0 {
            0.0
        } else {
            (self.source_bytes as f64 / (1024.0 * 1024.0)) / secs
        }
    }

    pub fn print_dashboard(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════════════╗");
        println!("║                      ⚡ RTSC PROFILER DASHBOARD                     ║");
        println!("╠══════════════════════╦══════════════╦═══════════╦════════════════════╣");
        println!("║ Phase                ║ Time         ║ % Total   ║ Throughput / Stats ║");
        println!("╠══════════════════════╬══════════════╬═══════════╬════════════════════╣");
        
        println!(
            "║ 1. Lexing            ║ {:>10.2?} ║ {:>8.1}% ║ {:>8.1} MB/s    ║",
            self.lex_duration,
            self.pct(self.lex_duration),
            self.mb_per_sec(self.lex_duration)
        );
        println!(
            "║ 2. Parsing (AST)     ║ {:>10.2?} ║ {:>8.1}% ║ {:>8} nodes    ║",
            self.parse_duration,
            self.pct(self.parse_duration),
            self.node_count
        );
        println!(
            "║ 3. Binding & Scopes  ║ {:>10.2?} ║ {:>8.1}% ║ {:>8} symbols  ║",
            self.bind_duration,
            self.pct(self.bind_duration),
            self.symbol_count
        );
        println!(
            "║ 4. Type Checking     ║ {:>10.2?} ║ {:>8.1}% ║ cached relations   ║",
            self.check_duration,
            self.pct(self.check_duration)
        );
        println!(
            "║ 5. JS Code Emission  ║ {:>10.2?} ║ {:>8.1}% ║ {:>8} bytes out║",
            self.emit_duration,
            self.pct(self.emit_duration),
            self.emit_bytes
        );
        println!("╠══════════════════════╬══════════════╬═══════════╬════════════════════╣");
        println!(
            "║ TOTAL PIPELINE       ║ {:>10.2?} ║    100.0% ║ {:>8.1} MB/s    ║",
            self.total_duration,
            self.mb_per_sec(self.total_duration)
        );
        println!("╠══════════════════════╩══════════════╩═══════════╩════════════════════╣");
        println!(
            "║ Peak RSS Memory: {:>8.2} MB | Tokens: {:>6} | Source: {:>6} bytes ║",
            self.peak_rss_bytes as f64 / (1024.0 * 1024.0),
            self.token_count,
            self.source_bytes
        );
        println!("╚══════════════════════════════════════════════════════════════════════╝\n");
    }
}

// OS specific memory query for peak RSS
#[cfg(windows)]
pub fn get_peak_rss_bytes() -> usize {
    #[repr(C)]
    struct PROCESS_MEMORY_COUNTERS {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn K32GetProcessMemoryInfo(
            process: *mut std::ffi::c_void,
            pmc: *mut PROCESS_MEMORY_COUNTERS,
            cb: u32,
        ) -> i32;
    }

    unsafe {
        let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        let proc = GetCurrentProcess();
        if K32GetProcessMemoryInfo(proc, &mut counters, counters.cb) != 0 {
            counters.peak_working_set_size
        } else {
            0
        }
    }
}

#[cfg(unix)]
pub fn get_peak_rss_bytes() -> usize {
    unsafe extern "C" {
        fn getrusage(who: i32, usage: *mut libc_rusage) -> i32;
    }

    #[repr(C)]
    struct libc_rusage {
        ru_utime: [usize; 2],
        ru_stime: [usize; 2],
        ru_maxrss: usize,
        ru_ixrss: usize,
        ru_idrss: usize,
        ru_isrss: usize,
        ru_minflt: usize,
        ru_majflt: usize,
        ru_nswap: usize,
        ru_inblock: usize,
        ru_oublock: usize,
        ru_msgsnd: usize,
        ru_msgrcv: usize,
        ru_nsignals: usize,
        ru_nvcsw: usize,
        ru_nivcsw: usize,
    }

    unsafe {
        let mut usage: libc_rusage = std::mem::zeroed();
        if getrusage(0, &mut usage) == 0 {
            #[cfg(target_os = "macos")]
            {
                usage.ru_maxrss
            }
            #[cfg(not(target_os = "macos"))]
            {
                usage.ru_maxrss * 1024
            }
        } else {
            0
        }
    }
}

#[cfg(not(any(windows, unix)))]
pub fn get_peak_rss_bytes() -> usize {
    0
}

// Executes full compiler pipeline while recording telemetry for each stage
pub fn profile_compiler_pipeline(source: &str) -> (ProfileMetrics, String) {
    let total_start = Instant::now();

    // 1. Lexing
    let lex_start = Instant::now();
    let mut lexer = Lexer::new(source);
    lexer.skip_trivia = true;
    let mut tokens = Vec::with_capacity(1024);
    loop {
        let tok = lexer.next_token();
        if tok.kind == TokenKind::Eof {
            break;
        }
        tokens.push(tok);
    }
    let lex_duration = lex_start.elapsed();

    // 2. Parsing
    let parse_start = Instant::now();
    let mut interner = StringInterner::new();
    let mut arena = AstArena::new();
    let mut parser = Parser::new(&tokens, source);
    let root = parser.parse_program(&mut interner, &mut arena);
    let parse_duration = parse_start.elapsed();

    // 3. Binding
    let bind_start = Instant::now();
    let mut binder = Binder::new();
    binder.bind_program(root, &arena);
    let bind_duration = bind_start.elapsed();

    // 4. Type Checking
    let check_start = Instant::now();
    let mut checker = TypeChecker::new();
    checker.check_node(root, &arena);
    let check_duration = check_start.elapsed();

    // 5. Code Emission
    let emit_start = Instant::now();
    let mut emitter = Emitter::new();
    let emitted = emitter.emit_program(root, &arena, &interner);
    let emit_duration = emit_start.elapsed();

    let total_duration = total_start.elapsed();
    let peak_rss_bytes = get_peak_rss_bytes();

    let metrics = ProfileMetrics {
        source_bytes: source.len(),
        lex_duration,
        token_count: tokens.len(),
        parse_duration,
        node_count: arena.len(),
        bind_duration,
        symbol_count: binder.symbols.len(),
        scope_count: binder.scopes.len(),
        check_duration,
        emit_duration,
        emit_bytes: emitted.len(),
        total_duration,
        peak_rss_bytes,
    };

    (metrics, emitted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_pipeline_measures_metrics() {
        let src = "const x = 10; const y = 20; const z = x + y;";
        let (metrics, emitted) = profile_compiler_pipeline(src);

        assert_eq!(metrics.source_bytes, src.len());
        assert!(metrics.token_count > 0);
        assert!(metrics.node_count > 0);
        assert!(metrics.symbol_count > 0);
        assert!(!emitted.is_empty());
        assert!(metrics.peak_rss_bytes > 0);
    }

    #[test]
    fn test_dashboard_does_not_panic() {
        let src = "const foo = 42;";
        let (metrics, _) = profile_compiler_pipeline(src);
        // Ensure printing dashboard functions without arithmetic errors / div-by-zero
        metrics.print_dashboard();
    }
}
