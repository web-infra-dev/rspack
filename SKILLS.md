# Skills Required for Rspack Development

Skills and knowledge areas needed to contribute to Rspack.

## Core Programming Languages

### Rust (Essential)

Primary language for Rspack's core engine.

**Required Skills:**

- **Fundamentals**: Ownership, borrowing, lifetimes, traits, generics
- **Async**: `async/await`, `Future`, `tokio` runtime
- **Concurrency**: Thread-safe structures, `Arc`, `Mutex`, `RwLock`
- **Error Handling**: `Result<T, E>`, `Option<T>`, error propagation
- **Macros**: Declarative, procedural, derive macros
- **Performance**: Zero-cost abstractions, memory management, profiling
- **Testing**: Unit tests, integration tests

**Key Crates:**

- `napi-rs`: Node.js bindings
- `swc`: JavaScript/TypeScript compiler
- `tokio`: Async runtime
- `serde`: Serialization
- `rayon`: Parallel processing

**Resources:**

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### JavaScript/TypeScript (Essential)

Used for JavaScript API layer, CLI tools, and test infrastructure.

**Required Skills:**

- **TypeScript**: Type system, generics, utility types
- **Modern JavaScript**: ES6+, async/await, modules (ESM/CJS)
- **Node.js**: APIs, streams, file system
- **Package Management**: npm, pnpm, workspaces
- **Build Tools**: Understanding bundlers, loaders, plugins

**Key Libraries:**

- `@rspack/core`: Core Rspack API
- `@rspack/test-tools`: Testing utilities

**Resources:**

- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [Node.js Documentation](https://nodejs.org/docs/)
- [Webpack Concepts](https://webpack.js.org/concepts/)

## Build System & Compiler Knowledge

### Bundler Concepts (Essential)

**Key Concepts:**

- Module resolution
- Dependency graph
- Code splitting
- Tree shaking
- Source maps
- Hot Module Replacement (HMR)
- Asset processing

**Resources:**

- [Webpack Concepts](https://webpack.js.org/concepts/)
- [Module Federation](https://webpack.js.org/concepts/module-federation/)

### Compiler Theory (Helpful)

**Key Concepts:**

- AST (Abstract Syntax Tree)
- Parsing, transformation, code generation
- Static analysis

**Resources:**

- [Crafting Interpreters](https://craftinginterpreters.com/)
- [The Super Tiny Compiler](https://github.com/jamiebuilds/the-super-tiny-compiler)

## Testing & Quality Assurance

### Testing Skills (Essential)

**Rust Testing:**

- Unit tests with `#[test]`
- Integration tests
- Test organization

**JavaScript/TypeScript Testing:**

- Integration tests in `tests/rspack-test/`
- Snapshot testing
- Test types: Normal, Config, Hot, Watch, StatsOutput, etc.

**Test Tools:**

- `cargo test`: Rust test runner
- `@rspack/test-tools`: Rspack test utilities

**Resources:**

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Testing Guide](./website/docs/en/contribute/development/testing.mdx)

### Code Quality (Essential)

**Linting:**

- Rust: `cargo clippy`, `cargo check`
- JavaScript/TypeScript: Biome, Rslint
- Type checking: TypeScript compiler

**Formatting:**

- Rust: `cargo fmt`
- JavaScript/TypeScript: Prettier
- TOML: taplo

## Debugging & Performance

### Debugging Skills (Essential)

**Rust Debugging:**

- VS Code debugging configuration
- `rust-lldb` for panic debugging
- Breakpoints and step-through debugging

**JavaScript Debugging:**

- Node.js `--inspect` flag
- Chrome DevTools integration

**Resources:**

- [Debugging Guide](./website/docs/en/contribute/development/debugging.mdx)

### Performance Optimization (Important)

**Key Areas:**

- Profiling bottlenecks
- Parallelization
- Caching (persistent and memory)
- Incremental compilation
- Memory management

**Tools:**

- `cargo flamegraph`: Rust profiling
- Perfetto tracing
- Benchmark suites

**Resources:**

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Performance Tracing](./crates/rspack_tracing/)

## System Design & Architecture

### Monorepo Management (Important)

**Skills:**

- Workspace management (pnpm, Cargo)
- Dependency management
- Build orchestration
- Code organization

**Resources:**

- [pnpm Workspaces](https://pnpm.io/workspaces)
- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)

### Plugin System Architecture (Important)

**Key Concepts:**

- Plugin interface
- Hook system (sync/async, series/parallel)
- Compilation lifecycle (make, seal, emit)
- Module graph management

**Resources:**

- [Plugin System](./ARCHITECTURE.md)
- [Common Patterns](./COMMON_PATTERNS.md)

## Cross-Language Interoperability

### Rust-JavaScript Interop (Essential)

**Key Concepts:**

- NAPI (Node-API)
- `napi-rs`: Rust bindings
- Type marshalling
- Async interop
- Memory management

**Resources:**

- [NAPI Documentation](https://nodejs.org/api/n-api.html)
- [napi-rs Documentation](https://napi.rs/)

### WebAssembly (Optional)

- WASM compilation
- WASI (WebAssembly System Interface)
- Browser vs. Node.js environments

## Version Control & Collaboration

### Git & GitHub (Essential)

**Skills:**

- Branching (feature branches)
- Conventional commits
- Pull requests
- Code review

**Resources:**

- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Flow](https://guides.github.com/introduction/flow/)

## Domain-Specific Knowledge

### Webpack Ecosystem (Important)

**Key Areas:**

- Webpack API (Compiler, Compilation, Module, Chunk)
- Plugin development patterns
- Loader development patterns
- Configuration options

**Resources:**

- [Webpack Documentation](https://webpack.js.org/)
- [Webpack Plugin API](https://webpack.js.org/api/plugins/)

### Frontend Build Tools (Helpful)

- Vite, esbuild, Rollup, Parcel
- Module Federation
- Code splitting strategies
- CSS processing (PostCSS, Lightning CSS)

## Learning Path Recommendations

### For Rust Developers New to Bundlers

1. Learn webpack concepts and ecosystem
2. Study Rspack's architecture
3. Understand compilation pipeline
4. Practice writing plugins and loaders

### For JavaScript Developers New to Rust

1. Complete Rust fundamentals (ownership, borrowing)
2. Learn async Rust and concurrency
3. Study NAPI and Rust-JavaScript interop
4. Practice reading and modifying Rust code

### For New Contributors

1. Set up development environment
2. Run existing tests and understand structure
3. Fix small bugs or add tests
4. Gradually work on larger features

## Skill Assessment

### Beginner Level

- Can read and understand Rust code
- Can write basic TypeScript/JavaScript
- Understands basic bundler concepts
- Can run tests and debug issues

### Intermediate Level

- Can write Rust code following patterns
- Can implement plugins or loaders
- Understands compilation pipeline
- Can debug complex issues

### Advanced Level

- Can design and implement new features
- Understands performance implications
- Can optimize hot paths
- Can mentor other contributors

## Resources

### Official Documentation

- [Rspack Documentation](https://rspack.rs/)
- [Project Architecture](./website/docs/en/contribute/development/project.md)
- [Architecture Guide](./ARCHITECTURE.md)
- [API Design](./API_DESIGN.md)
- [Code Style](./CODE_STYLE.md)
- [Common Patterns](./COMMON_PATTERNS.md)
- [Glossary](./GLOSSARY.md)

### External Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [Webpack Documentation](https://webpack.js.org/)
- [Node.js Documentation](https://nodejs.org/docs/)

### Community

- [Rspack Discord](https://discord.gg/79ZZ66GH9E)
- [GitHub Discussions](https://github.com/web-infra-dev/rspack/discussions)
- [GitHub Issues](https://github.com/web-infra-dev/rspack/issues)

## Conclusion

Contributing to Rspack requires skills in Rust, JavaScript/TypeScript, build systems, and performance optimization. Start with areas most relevant to your contributions and gradually expand knowledge. The community is welcoming and supportive.
