# Architecture

High-level architecture of Rspack, including core components, data flow, and design decisions.

## Overview

Rspack is a high-performance JavaScript bundler written in Rust that maintains strong compatibility with the webpack ecosystem. The architecture leverages Rust's performance while providing a webpack-compatible API through Node.js bindings.

## High-Level Architecture

```text
JavaScript/TypeScript Layer (@rspack/core, plugins, loaders)
         ↓
Binding Layer (@rspack/binding, rspack_binding_api, rspack_napi)
         ↓
Rust Core Layer (rspack_core, compilation engine)
```

### Layer separation

1. **JavaScript/TypeScript Layer** (`packages/`): Webpack-compatible API, configuration, file system operations
2. **Rust Core Layer** (`crates/`): Core compilation engine, module system, plugin/loader execution
3. **Binding Layer** (`crates/node_binding`, `crates/rspack_binding_api`, `crates/rspack_napi`):
   Node-API packaging, conversion, native-backed objects, JavaScript callbacks, and runtime support

## Core components

### Compiler

Main entry point orchestrating the build process.

**Responsibilities:**

- Configuration management
- Plugin registration and execution
- Compilation lifecycle management
- File system abstraction
- Watch mode support

**Key Hooks:**

- `beforeRun`, `run`, `compile`, `make`, `emit`, `done`

### Compilation

Represents a single build instance managing module graph, chunks, and assets.

**Responsibilities:**

- Module graph construction
- Dependency resolution
- Chunk creation and optimization
- Asset generation
- Code generation

**Key Data Structures:**

- `ModuleGraph`: Tracks modules, dependencies, and connections
- `ChunkGraph`: Relationship between chunks and modules
- `Assets`: Output files and content

**Key Hooks:**

- `buildModule`, `succeedModule`, `processAssets`, `optimizeChunks`, `afterSeal`

### Module system

Modules are the basic unit of code organization.

**Module Types:**

- `NormalModule`: Regular JavaScript/TypeScript modules
- `ContextModule`: Dynamic require contexts
- `ExternalModule`: External dependencies
- `ConcatenatedModule`: Concatenated modules for optimization

**Module Lifecycle:**

1. **Parse**: Parse source code into AST
2. **Build**: Resolve dependencies and build module
3. **Code Generation**: Generate runtime code
4. **Seal**: Finalize module

### Module graph

Manages modules and their relationships through dependencies and connections.

**Key Concepts:**

- **Dependency**: Relationship between modules (import, require). Has `DependencyId` and types (ModuleDependency, ContextDependency, etc.)
- **Connection**: `ModuleGraphConnection` linking modules with `dependency_id`, `original_module_identifier`, `resolved_module`
- **Module Graph**: Central data structure tracking all modules, dependencies, connections, and export/import information

**Graph Construction:**

1. Start from entry points
2. Parse modules to discover dependencies
3. Create `Dependency` objects
4. Resolve dependencies to target modules
5. Create `ModuleGraphConnection` objects
6. Build complete module graph

### Chunk system

Chunks are groups of modules bundled together.

**Chunk Types:**

- **Entry Chunks**: Generated from entry points
- **Async Chunks**: Code-split chunks loaded on demand
- **Runtime Chunks**: Webpack runtime code
- **Vendor Chunks**: Third-party dependencies

**Chunk Splitting:**

- Controlled by `optimization.splitChunks`
- Groups modules based on criteria (size, cache groups, etc.)

## Compilation pipeline

```text
1. Initialize → Load config, create compiler, register plugins
2. Compile → Build module graph, resolve dependencies
3. Optimize → Tree shaking, code splitting, minification
4. Generate → Code generation, asset creation, output
```

### Detailed stages

#### Initialize phase

- Load and normalize configuration
- Create compiler instance
- Apply plugins
- Initialize file systems

#### Compile phase

- **Entry Processing**: Process entry points, create entry modules
- **Module Building**: Parse source (SWC), extract dependencies, transform (loaders)
- **Dependency Resolution**: Resolve paths, handle aliases/extensions, process externals

#### Optimization phase

- **Tree Shaking**: Analyze exports/imports, remove unused code
- **Code Splitting**: Split chunks based on configuration, create async chunks
- **Minification**: Minify JS (SWC), CSS (Lightning CSS)

#### Code generation phase

- **Runtime Code**: Generate webpack runtime, module loading code, HMR code
- **Asset Generation**: Generate output files, apply filename templates, generate source maps

## Plugin system

### Plugin architecture

Plugins extend functionality by hooking into compilation lifecycle.

**Plugin Types:**

- **Builtin Plugins**: Core functionality (JavaScript, CSS, HTML)
- **User Plugins**: Custom plugins via configuration
- **External Plugins**: webpack-compatible plugins

### Hook system

Hooks allow plugins to intercept and modify compilation.

**Hook Types:**

- **SyncSeries**: Synchronous, sequential
- **SyncSeriesBail**: Synchronous, can bail out
- **AsyncSeries**: Asynchronous, sequential
- **AsyncSeriesBail**: Asynchronous, can bail out
- **AsyncParallel**: Asynchronous, parallel

## Loader system

Loaders transform source code before adding to dependency graph.

**Loader Execution:**

1. Loader chain determined by module rules
2. Loaders execute in reverse order (last to first)
3. Each loader receives previous loader's output
4. Final output parsed and added to module graph

**Loader Types:**

- **Builtin Loaders**: SWC loader, Lightning CSS loader
- **JavaScript Loaders**: Custom loaders in JavaScript
- **Rust Loaders**: High-performance loaders in Rust

## Module resolution

Determines how module paths resolve to actual files.

**Resolution Process:**

1. Check if module is external
2. Apply aliases and extensions
3. Resolve using enhanced-resolve
4. Handle package.json exports/imports
5. Return resolved path

**Resolution Strategies:**

- Relative: `./module`, `../module`
- Absolute: `/path/to/module`
- Module: `module-name` (from node_modules)
- Alias: Custom alias mappings

## Cache and incremental compilation

Cache and Incremental are independent performance mechanisms:

- **Cache** stores fine-grained computation results. It supports process-local memory storage and
  filesystem-backed persistent storage.
- **Incremental** recovers prior pass artifacts during development rebuilds, watch mode, and HMR so
  unaffected portions can be reused and affected work can be updated from known mutations.

The two options form orthogonal axes. Disabling Cache must not disable Incremental, and disabling
Incremental must not disable Cache. This allows all four Cache on/off × Incremental on/off
combinations.

Incremental does not make separate one-shot `rspack build` invocations incremental. Reuse across
those invocations comes from filesystem Cache. Incremental is conceptually aligned with webpack's
`cacheUnaffected` behavior, but Rspack exposes it independently from Cache.

Rspack currently has two internal Cache backend implementations, `legacy_cache` and `new_cache`.
Both implement Cache storage responsibilities and must remain independent from the compiler-owned
Incremental artifacts.

See [Cache and Incremental Compilation](./CACHE_AND_INCREMENTAL.md) for the configuration matrix,
ownership model, backend boundary, and architectural invariants.

## File system abstraction

Cross-platform file system abstraction.

**File System Types:**

- **InputFileSystem**: Read source files
- **OutputFileSystem**: Write output files
- **IntermediateFileSystem**: Temporary files
- **WatchFileSystem**: File watching for watch mode

**Implementation:**

- Node.js: Uses Node.js fs module
- Browser: Uses in-memory file system (memfs)
- Custom: Can be overridden for testing

## Error handling

Unified error system.

**Error Types:**

- **Build Errors**: Module build failures
- **Resolution Errors**: Module resolution failures
- **Compilation Errors**: Compilation process errors
- **Plugin Errors**: Plugin execution errors

**Error Propagation:**

- Errors collected in `compilation.errors`
- Warnings collected in `compilation.warnings`
- Errors formatted with context and suggestions

## Performance optimizations

### Parallel processing

- Module building parallelized
- Asset processing parallelized
- Code generation uses parallel workers

### Incremental compilation

- Targets development rebuilds, watch mode, and HMR in the same compiler lifecycle
- Uses mutations and dependency relationships to determine affected work
- Recovers prior pass artifacts and updates affected work from mutations independently from Cache
- Does not provide incremental semantics across standalone one-shot builds

### Memory management

- Uses mimalloc for optimized allocation (Linux/macOS)
- Efficient data structures (custom HashMap, HashSet)
- Minimizes allocations in hot paths

## Data flow

### Build request flow

```text
User Code → Configuration → Compiler.apply() → Plugin Registration
→ Compiler.run() → Compilation → Module Graph → Optimization
→ Code Generation → Asset Emission → Output Files
```

### Module processing flow

```text
Source File → Loader Chain → Parsed AST → Dependency Extraction
→ Module Graph Node → Code Generation → Runtime Code
```

## Design decisions

### Why Rust?

- **Performance**: Near-C performance with memory safety
- **Concurrency**: Excellent async/await for parallel processing
- **Ecosystem**: Rich ecosystem for parsing and transformation

### Why webpack Compatibility?

- **Ecosystem**: Leverage existing webpack plugins and loaders
- **Migration**: Easy migration path for existing projects
- **Community**: Benefit from webpack's large community

### Why NAPI?

- **Performance**: Native bindings provide low overhead
- **Compatibility**: Works with Node.js ecosystem
- **Type Safety**: Type-safe bindings between Rust and TypeScript

## Extension points

### Adding a new plugin

1. Create plugin struct with `#[plugin]` attribute
2. Implement hooks with `#[plugin_hook]` attribute
3. Implement `Plugin` trait
4. Register hooks in `apply` method

### Adding a new loader

1. Create loader function
2. Register in module rules
3. Implement transformation logic
4. Return transformed code

### Adding a new hook

1. Define hook using `define_hook!` macro
2. Add to appropriate hooks struct
3. Call hook at appropriate point
4. Plugins can tap into the hook

## Resources

- [Project Structure](../website/docs/en/contribute/development/project.md)
- [JavaScript API Architecture](../website/docs/en/api/javascript-api/architecture.mdx)
- [JavaScript Binding Guide](./BINDING.md)
- [Cache and Incremental Compilation](./CACHE_AND_INCREMENTAL.md)
- [Incremental Artifacts](./ARTIFACTS.md)
- [Common Patterns](./COMMON_PATTERNS.md)
- [Code Style](./CODE_STYLE.md)
- [Plugin API Documentation](https://rspack.rs/api/plugin-api/)
