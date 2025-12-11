# Skills Required for Rspack Development

This document outlines the skills and knowledge areas needed to effectively contribute to the Rspack project. Whether you're a developer, contributor, or AI assistant, understanding these skills will help you navigate and contribute to the codebase.

## Core Programming Languages

### Rust (Essential)

Rust is the primary language for Rspack's core engine. Strong Rust skills are essential for most contributions.

**Required Skills:**

- **Rust Fundamentals**: Ownership, borrowing, lifetimes, traits, generics
- **Async Programming**: `async/await`, `Future`, `tokio` runtime
- **Concurrency**: Thread-safe data structures, `Arc`, `Mutex`, `RwLock`
- **Error Handling**: `Result<T, E>`, `Option<T>`, error propagation
- **Macros**: Declarative macros, procedural macros, derive macros
- **Performance**: Zero-cost abstractions, memory management, profiling
- **Testing**: Unit tests, integration tests, test organization

**Key Rust Crates Used:**

- `napi-rs`: Node.js bindings
- `swc`: JavaScript/TypeScript compiler
- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `rayon`: Parallel processing
- `rustc_hash`: Fast hash maps
- `regex`: Regular expressions

**Resources:**

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### JavaScript/TypeScript (Essential)

TypeScript is used for the JavaScript API layer, CLI tools, and test infrastructure.

**Required Skills:**

- **TypeScript**: Type system, generics, utility types, type inference
- **Modern JavaScript**: ES6+, async/await, Promises, modules (ESM/CJS)
- **Node.js**: APIs, streams, file system, process management
- **Package Management**: npm, pnpm, workspace management
- **Build Tools**: Understanding of bundlers, loaders, plugins

**Key Libraries Used:**

- `@rspack/core`: Core Rspack API
- `@rspack/test-tools`: Testing utilities
- Various webpack-compatible plugins and loaders

**Resources:**

- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [Node.js Documentation](https://nodejs.org/docs/)
- [Webpack Concepts](https://webpack.js.org/concepts/)

## Build System & Compiler Knowledge

### Bundler Concepts (Essential)

Understanding how bundlers work is crucial for contributing to Rspack.

**Key Concepts:**

- **Module Resolution**: How modules are found and resolved
- **Dependency Graph**: Module relationships and dependencies
- **Code Splitting**: Chunking strategies and lazy loading
- **Tree Shaking**: Dead code elimination
- **Source Maps**: Debugging support
- **Hot Module Replacement (HMR)**: Development experience
- **Asset Processing**: Handling various file types (JS, CSS, images, etc.)

**Resources:**

- [Webpack Concepts](https://webpack.js.org/concepts/)
- [Module Federation](https://webpack.js.org/concepts/module-federation/)

### Compiler Theory (Helpful)

Understanding compiler concepts helps with code transformation and optimization.

**Key Concepts:**

- **AST (Abstract Syntax Tree)**: Code representation
- **Parsing**: Converting source code to AST
- **Transformation**: Modifying AST
- **Code Generation**: Converting AST back to code
- **Static Analysis**: Type checking, dependency analysis

**Resources:**

- [Crafting Interpreters](https://craftinginterpreters.com/)
- [The Super Tiny Compiler](https://github.com/jamiebuilds/the-super-tiny-compiler)

## Testing & Quality Assurance

### Testing Skills (Essential)

Comprehensive testing is critical for maintaining Rspack's reliability.

**Rust Testing:**

- Unit tests with `#[test]`
- Integration tests
- Test organization and fixtures
- Mocking and test doubles

**JavaScript/TypeScript Testing:**

- Integration tests in `tests/rspack-test/`
- Snapshot testing
- Test case types: Normal, Config, Hot, Watch, StatsOutput, etc.
- E2E testing

**Test Tools:**

- `cargo test`: Rust test runner
- `@rspack/test-tools`: Rspack test utilities
- Jest-like testing patterns

**Resources:**

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Testing Guide](./website/docs/en/contribute/development/testing.mdx)

### Code Quality (Essential)

Maintaining high code quality is essential for a production tool.

**Linting:**

- Rust: `cargo clippy`, `cargo check`
- JavaScript/TypeScript: Biome, Rslint
- Type checking: TypeScript compiler

**Formatting:**

- Rust: `cargo fmt`
- JavaScript/TypeScript: Prettier
- TOML: taplo

**Resources:**

- [Rust Clippy](https://github.com/rust-lang/rust-clippy)
- [Biome](https://biomejs.dev/)

## Debugging & Performance

### Debugging Skills (Essential)

Effective debugging is crucial for development and troubleshooting.

**Rust Debugging:**

- VS Code debugging configuration
- `rust-lldb` for panic debugging
- Breakpoints and step-through debugging
- Deadlock detection

**JavaScript Debugging:**

- Node.js `--inspect` flag
- Chrome DevTools integration
- Source map debugging

**Resources:**

- [Debugging Guide](./website/docs/en/contribute/development/debugging.mdx)

### Performance Optimization (Important)

Rspack prioritizes performance, so understanding optimization is valuable.

**Key Areas:**

- **Profiling**: Identifying bottlenecks
- **Parallelization**: Multi-threaded processing
- **Caching**: Persistent and memory caching
- **Incremental Compilation**: Only rebuilding changed modules
- **Memory Management**: Efficient memory usage

**Tools:**

- `cargo flamegraph`: Rust profiling
- Perfetto tracing
- Benchmark suites

**Resources:**

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Performance Tracing](./crates/rspack_tracing/)

## System Design & Architecture

### Monorepo Management (Important)

Rspack is a monorepo with multiple Rust crates and JavaScript packages.

**Skills:**

- **Workspace Management**: pnpm workspaces, Cargo workspaces
- **Dependency Management**: Version consistency, dependency resolution
- **Build Orchestration**: Coordinating builds across packages
- **Code Organization**: Module boundaries, API design

**Resources:**

- [pnpm Workspaces](https://pnpm.io/workspaces)
- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)

### Plugin System Architecture (Important)

Understanding Rspack's plugin and hook system is essential for extending functionality.

**Key Concepts:**

- **Plugin Interface**: Implementing plugins
- **Hook System**: Tapable hooks, hook types (sync/async, series/parallel)
- **Compilation Lifecycle**: Make, seal, emit phases
- **Module Graph**: Managing module relationships

**Resources:**

- [Plugin System](./ARCHITECTURE.md)
- [Common Patterns](./COMMON_PATTERNS.md)

## Cross-Language Interoperability

### Rust-JavaScript Interop (Essential)

Rspack bridges Rust and JavaScript through NAPI (Node-API).

**Key Concepts:**

- **NAPI (Node-API)**: Stable C API for Node.js addons
- **napi-rs**: Rust bindings for NAPI
- **Type Marshalling**: Converting between Rust and JavaScript types
- **Async Interop**: Handling async operations across languages
- **Memory Management**: Managing memory across language boundaries

**Resources:**

- [NAPI Documentation](https://nodejs.org/api/n-api.html)
- [napi-rs Documentation](https://napi.rs/)

### WebAssembly (Optional)

Rspack supports WebAssembly builds for browser environments.

**Skills:**

- WASM compilation
- WASI (WebAssembly System Interface)
- Browser vs. Node.js environments

## Version Control & Collaboration

### Git & GitHub (Essential)

Effective collaboration requires Git proficiency.

**Skills:**

- **Branching**: Feature branches, main branch workflow
- **Commits**: Conventional commit messages
- **Pull Requests**: Creating and reviewing PRs
- **Code Review**: Reviewing and addressing feedback

**Resources:**

- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Flow](https://guides.github.com/introduction/flow/)

## Domain-Specific Knowledge

### Webpack Ecosystem (Important)

Rspack aims for webpack compatibility, so understanding webpack helps.

**Key Areas:**

- **Webpack API**: Compiler, Compilation, Module, Chunk APIs
- **Webpack Plugins**: Plugin development patterns
- **Webpack Loaders**: Loader development patterns
- **Webpack Configuration**: Configuration options and defaults

**Resources:**

- [Webpack Documentation](https://webpack.js.org/)
- [Webpack Plugin API](https://webpack.js.org/api/plugins/)

### Frontend Build Tools (Helpful)

Understanding the broader frontend ecosystem provides context.

**Related Tools:**

- Vite, esbuild, Rollup, Parcel
- Module Federation
- Code splitting strategies
- CSS processing (PostCSS, Lightning CSS)

## Learning Path Recommendations

### For Rust Developers New to Bundlers

1. Learn webpack concepts and ecosystem
2. Study Rspack's architecture and module system
3. Understand the compilation pipeline
4. Practice writing plugins and loaders

### For JavaScript Developers New to Rust

1. Complete Rust fundamentals (ownership, borrowing)
2. Learn async Rust and concurrency
3. Study NAPI and Rust-JavaScript interop
4. Practice reading and modifying Rust code

### For New Contributors

1. Set up development environment
2. Run existing tests and understand test structure
3. Fix small bugs or add tests
4. Gradually work on larger features

## Skill Assessment

### Beginner Level

- Can read and understand Rust code
- Can write basic TypeScript/JavaScript
- Understands basic bundler concepts
- Can run tests and debug issues

### Intermediate Level

- Can write Rust code following project patterns
- Can implement plugins or loaders
- Understands compilation pipeline
- Can debug complex issues

### Advanced Level

- Can design and implement new features
- Understands performance implications
- Can optimize hot paths
- Can mentor other contributors

## Resources for Skill Development

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

Contributing to Rspack requires a diverse skill set spanning Rust, JavaScript/TypeScript, build systems, and performance optimization. However, you don't need to master everything at once. Start with the areas most relevant to your contributions and gradually expand your knowledge.

The Rspack community is welcoming and supportive. Don't hesitate to ask questions, seek help, and learn from others. Every contribution, no matter how small, is valuable.
