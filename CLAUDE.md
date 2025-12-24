# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Arkaan is a programming language with Afrikaans keywords. This monorepo contains:

1. **Arkaan Language** (Rust) - Interpreter with lexer, parser, bytecode compiler, and stack-based VM
2. **Arkaan LSP Server** (Rust) - Language Server Protocol implementation for IDE support
3. **VS Code Extension** (TypeScript) - Syntax highlighting, snippets, and LSP client

## Build Commands

### Rust (Language & LSP)
```bash
cargo build                    # Debug build (both arkaan and arkaan-lsp)
cargo build --release          # Release build
cargo run -- examples/file.ark # Run an Arkaan program
cargo run                      # Start REPL
```

### VS Code Extension
```bash
cd vscode-arkaan
npm install
npm run compile                # Build TypeScript
npm run watch                  # Watch mode
```

## Architecture

### Language Pipeline (`src/`)
```
Source → Lexer → Tokens → Parser → AST → Compiler → Bytecode → VM → Output
```

- `token.rs` - Token types for Afrikaans keywords (`stel`, `as`, `terwyl`, `druk`, etc.)
- `lexer.rs` - Tokenizes source code
- `parser.rs` - Builds AST from tokens
- `ast.rs` - AST node definitions
- `compiler.rs` - Compiles AST to bytecode
- `bytecode.rs` - Bytecode instruction definitions
- `vm.rs` - Stack-based virtual machine
- `value.rs` - Runtime value types

### LSP Server (`src/lsp/`)
- `main.rs` - Tower-LSP server implementation with completion, hover, diagnostics
- `analysis.rs` - Document analysis for diagnostics and completions

### VS Code Extension (`vscode-arkaan/`)
- `src/extension.ts` - LSP client that connects to `arkaan-lsp` executable
- `syntaxes/arkaan.tmLanguage.json` - TextMate grammar for syntax highlighting
- `snippets/arkaan.json` - Code snippets for common patterns

## Arkaan Language Keywords

| Afrikaans | English | Purpose |
|-----------|---------|---------|
| `stel`    | set     | Variable declaration |
| `as`      | if      | Conditional |
| `anders`  | else    | Else branch |
| `terwyl`  | while   | While loop |
| `druk`    | print   | Output |
| `waar`    | true    | Boolean true |
| `vals`    | false   | Boolean false |

## Development Workflow

The LSP server (`arkaan-lsp`) must be built before the VS Code extension can use it. The extension looks for the binary in `target/release/` or `target/debug/`.

Files use `.ark` extension.
