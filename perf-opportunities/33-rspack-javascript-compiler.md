# rspack_javascript_compiler — Performance Opportunities

**Size**: 1,605 lines across 8 files  
**Role**: Thin wrapper around SWC — provides parse, transform, minify, stringify for JavaScript/TypeScript  
**Impact**: Medium — called for every JS/TS module during build (parse) and potentially during minification

---

## Architecture

```rust
pub struct JavaScriptCompiler {
    globals: Globals,       // SWC's global state (thread-local marks)
    cm: Arc<SwcSourceMap>,  // SWC source map (interned file positions)
}
```

All operations go through `self.run(|| ...)` which sets the `GLOBALS` thread-local:

```rust
fn run<R>(&self, op: impl FnOnce() -> R) -> R {
    GLOBALS.set(&self.globals, op)
}
```

### Parse Path

```rust
pub fn parse_with_lexer(self, source, lexer, is_module, comments, with_tokens) -> Result<(Ast, Option<Vec<TokenAndSpan>>)> {
    parse_with_lexer(lexer, is_module, with_tokens)
        .map(|(program, tokens)| {
            (Ast::new(program, self.cm.clone(), comments).with_context(ast::Context::new(self.cm, Some(self.globals))), tokens)
        })
}
```

**Key observation**: `JavaScriptCompiler::new()` creates new `Globals` and `SwcSourceMap` each time. The `Globals` includes SWC's `Mark` counter which is process-global, so creating multiple compilers doesn't share marks correctly.

### Transform Path

The transform method accepts callbacks for custom SWC visitors:
```rust
pub fn transform(source, filename, comments, options, source_map_kind, before_pass, after_pass) -> Result<TransformOutput>
```

This creates a full SWC compilation pipeline per call including program/module configuration.

---

## Performance Concerns

### 1. New Compiler Per Module

Each module build creates `JavaScriptCompiler::new()`:
```rust
// In parser_and_generator/mod.rs:
let javascript_compiler = JavaScriptCompiler::new();
```

And in SwcLoader:
```rust
// In rspack_loader_swc/src/lib.rs:
let javascript_compiler = JavaScriptCompiler::new();
```

Each `new()` creates:
- `Globals::new()` — allocates TLS state
- `Arc<SwcSourceMap>` — new source map with empty files

At 10K modules, that's 20K compiler instantiations (one for loader, one for parser).

**Opportunity**: Use thread-local pooling:
```rust
thread_local! {
    static COMPILER: RefCell<JavaScriptCompiler> = RefCell::new(JavaScriptCompiler::new());
}
```

**Estimated savings**: 1-2% of parse/transform time (instantiation is fast but not free)

### 2. SourceMap Accumulation

Each `JavaScriptCompiler` creates a new `SwcSourceMap` which accumulates source files. Since compilers are created per-module, each only holds one file, avoiding memory bloat. But if pooled, the source map would accumulate all files.

**Consideration**: If pooling compilers, periodically create fresh SourceMaps to avoid memory growth.

### 3. Token Collection Overhead

When `with_tokens = true`, the parser collects all tokens into a `Vec<TokenAndSpan>`:
```rust
fn parse_with_lexer(lexer: Lexer, is_module: IsModule, with_tokens: bool) -> Result<(SwcProgram, Option<Vec<TokenAndSpan>>)>
```

For a 1000-line file, this can be 5,000-10,000 tokens. Each `TokenAndSpan` is ~32 bytes. That's ~160-320KB per module just for tokens that are only used for semicolon detection.

**Opportunity**: Detect semicolons during lexing instead of collecting all tokens. This would eliminate the token allocation entirely.

**Estimated savings**: 5-10% of parse memory, 1-2% of parse time

---

## Summary

| # | Opportunity | Estimated Impact | Effort |
|---|-----------|-----------------|--------|
| 1 | Pool JavaScriptCompiler per thread | 1-2% of parse/transform | Low |
| 2 | Detect semicolons during lexing | 1-2% parse time, 5-10% parse memory | High (SWC change) |
