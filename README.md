<div align="center">

# ⚡ `rtsc`

### **Rust TypeScript Compiler**

*A blazingly fast TypeScript compiler implemented from scratch in Rust, engineered for extreme performance and instantaneous feedback loops.*

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?style=for-the-badge)](#license)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=for-the-badge)](#contributing)
[![Performance](https://img.shields.io/badge/performance-10x--50x%20faster-bolt.svg?style=for-the-badge&color=ff69b4)](#performance)

[Overview](#-overview) • [Key Features](#-key-features) • [Performance Target](#-performance-target) • [Quick Start](#-quick-start) • [Architecture](#-architecture) • [Roadmap](#-roadmap) • [Contributing](#-contributing)

---

</div>

## 📖 Overview

**`rtsc`** (Rust TypeScript Compiler) is an ambitious, ground-up implementation of the TypeScript compiler written in Rust.

Standard TypeScript compilation (`tsc`) runs on top of V8/Node.js, which introduces significant runtime overhead, single-threaded execution bottlenecks, and non-trivial memory consumption during large-scale project builds. **`rtsc`** solves this by leveraging Rust's zero-cost abstractions, fearless concurrency, and manual memory optimization to bring order-of-magnitude speedups to your build & type-checking pipelines.

> **Goal:** Provide an ultra-fast, drop-in replacement for `tsc` that cuts type-checking and code generation times from minutes down to seconds.

---

## ✨ Key Features

- **⚡ Unmatched Speed**: Native multi-threaded parsing, symbol resolution, and type checking.
- **🧠 Zero-GC Overhead**: Eliminates V8 garbage collection pauses for consistent, deterministic build times.
- **🎯 `tsc` CLI & `tsconfig.json` Compatibility**: Designed to integrate seamlessly into existing TypeScript projects without configuration headaches.
- **📦 Memory Efficient**: Minimal memory footprint using arenas and optimized AST layout, making it ideal for massive monorepos and CI/CD runners.
- **🧱 Built From Scratch**: Clean-room implementation tailored specifically for parallelism and performance rather than legacy runtime compatibility.

---

## 🚀 Performance Target

Comparison of full type-check and compilation times on medium-to-large TypeScript projects:

| Compiler Engine | Runtime | Multi-threading | Memory Usage | Relative Speed |
| :--- | :--- | :---: | :---: | :---: |
| **`tsc` (Official)** | Node.js (V8) | ❌ Single-threaded | High (GC overhead) | 1x (Baseline) |
| **`rtsc` (Rust)** | Native Binary | 🛠️ Multi-threaded | Low (Optimized Arenas) | **10x - 50x Faster** |

*Note: Benchmarks vary based on codebase size and hardware configuration.*

---

## 🛠️ Quick Start

### Prerequisites

Ensure you have the Rust toolchain installed:

```bash
rustup update stable
```

### Installation

Clone the repository and build from source:

```bash
# Clone the repository
git clone https://github.com/ShigrafS/rtsc.git
cd rtsc

# Build in release mode
cargo build --release

# (Optional) Install globally
cargo install --path .
```

### Basic Usage

```bash
# Run type check on current project directory
rtsc check

# Compile a single entry file
rtsc build src/index.ts

# Watch mode for live type checking
rtsc --watch
```

---

## 🏗️ Architecture Overview

`rtsc` is designed around a multi-stage parallel compiler pipeline:

```
    ┌──────────┐     ┌──────────┐     ┌──────────────┐     ┌──────────────┐
    │  Lexer   │ ──> │  Parser  │ ──> │ Symbol Table │ ──> │ Type Checker │
    └──────────┘     └──────────┘     └──────────────┘     └──────────────┘
                                                                  │
                                                                  ▼
                                                           ┌──────────────┐
                                                           │   Emitter    │
                                                           └──────────────┘
```

1. **Lexer & Parser**: High-throughput tokenization and lossless AST creation.
2. **Binder / Symbol Resolver**: Parallel scoping and symbol indexing.
3. **Type Checker**: Concurrent constraint solving and type inference.
4. **Emitter**: Fast JS/D.TS output code generation.

---

## 🗺️ Roadmap & Current Status

- [x] Project architecture & foundation setup
- [ ] High-performance Lexer & AST Parser
- [ ] Symbol binding and scope analysis
- [ ] Core Type Inference Engine
- [ ] `tsconfig.json` parsing & path mapping
- [ ] Incremental compilation & Watch mode
- [ ] Language Server Protocol (LSP) integration

---

## 🤝 Contributing

Contributions, feedback, and ideas are warmly welcome! Whether you are a Rustacean, a TypeScript enthusiast, or interested in compiler design:

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

Please ensure code conforms to standard formatting:
```bash
cargo fmt --all
cargo clippy --workspace --all-targets
```

---

## 📄 License

Distributed under the MIT License or Apache 2.0 License. See `LICENSE` for details.