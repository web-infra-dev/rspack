# Architecture

High-level architecture of Rspack, including core components, data flow, and design decisions.

## Overview

Rspack is a high-performance JavaScript bundler written in Rust that maintains strong compatibility with the webpack ecosystem. The architecture leverages Rust's performance while providing a webpack-compatible API through Node.js bindings.

## High-Level Architecture

```text
JavaScript/TypeScript Layer (@rspack/core, plugins, loaders)
         ↓ NAPI (Node-API)
Rust Core Layer (rspack_core, compilation engine)
```

### Layer Separation

1. **JavaScript/TypeScript Layer** (`packages/`): Webpack-compatible API, configuration, file system operations
2. **Rust Core Layer** (`crates/`): Core compilation engine, module system, plugin/loader execution
3. **Binding Layer**: NAPI bindings for Rust-JavaScript interop

## Core Components

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

### Module System

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

### Module Graph

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

### Chunk System

Chunks are groups of modules bundled together.

**Chunk Types:**

- **Entry Chunks**: Generated from entry points
- **Async Chunks**: Code-split chunks loaded on demand
- **Runtime Chunks**: Webpack runtime code
- **Vendor Chunks**: Third-party dependencies

**Chunk Splitting:**

- Controlled by `optimization.splitChunks`
- Groups modules based on criteria (size, cache groups, etc.)

## Compilation Pipeline

```text
1. Initialize → Load config, create compiler, register plugins
2. Compile → Build module graph, resolve dependencies
3. Optimize → Tree shaking, code splitting, minification
4. Generate → Code generation, asset creation, output
```

### Detailed Stages

#### Initialize Phase

- Load and normalize configuration
- Create compiler instance
- Apply plugins
- Initialize file systems

#### Compile Phase

- **Entry Processing**: Process entry points, create entry modules
- **Module Building**: Parse source (SWC), extract dependencies, transform (loaders)
- **Dependency Resolution**: Resolve paths, handle aliases/extensions, process externals

#### Optimization Phase

- **Tree Shaking**: Analyze exports/imports, remove unused code
- **Code Splitting**: Split chunks based on configuration, create async chunks
- **Minification**: Minify JS (SWC), CSS (Lightning CSS)

#### Code Generation Phase

- **Runtime Code**: Generate webpack runtime, module loading code, HMR code
- **Asset Generation**: Generate output files, apply filename templates, generate source maps

## Plugin System

### Plugin Architecture

Plugins extend functionality by hooking into compilation lifecycle.

**Plugin Types:**

- **Builtin Plugins**: Core functionality (JavaScript, CSS, HTML)
- **User Plugins**: Custom plugins via configuration
- **External Plugins**: webpack-compatible plugins

### Hook System

Hooks allow plugins to intercept and modify compilation.

**Hook Types:**

- **SyncSeries**: Synchronous, sequential
- **SyncSeriesBail**: Synchronous, can bail out
- **AsyncSeries**: Asynchronous, sequential
- **AsyncSeriesBail**: Asynchronous, can bail out
- **AsyncParallel**: Asynchronous, parallel

## Loader System

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

## Module Resolution

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

## Caching System

Multi-level caching for performance.

**Cache Levels:**

1. **Memory Cache**: In-memory for current build
2. **Persistent Cache**: Disk-based across builds
3. **Module Cache**: Cached module build results
4. **Compilation Cache**: Cached compilation results

**Cache Invalidation:**

- File content changes
- Configuration changes
- Dependency changes
- Manual cache clearing

## File System Abstraction

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

## Error Handling

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

## Performance Optimizations

### Parallel Processing

- Module building parallelized
- Asset processing parallelized
- Code generation uses parallel workers

### Incremental Compilation

- Only rebuilds changed modules
- Uses dependency graph to determine affected modules
- Caches unchanged modules

### Memory Management

- Uses mimalloc for optimized allocation (Linux/macOS)
- Efficient data structures (custom HashMap, HashSet)
- Minimizes allocations in hot paths

## Data Flow

### Build Request Flow

```text
User Code → Configuration → Compiler.apply() → Plugin Registration
→ Compiler.run() → Compilation → Module Graph → Optimization
→ Code Generation → Asset Emission → Output Files
```

### Module Processing Flow

```text
Source File → Loader Chain → Parsed AST → Dependency Extraction
→ Module Graph Node → Code Generation → Runtime Code
```

## Design Decisions

### Why Rust?

- **Performance**: Near-C performance with memory safety
- **Concurrency**: Excellent async/await for parallel processing
- **Ecosystem**: Rich ecosystem for parsing and transformation

### Why Webpack Compatibility?

- **Ecosystem**: Leverage existing webpack plugins and loaders
- **Migration**: Easy migration path for existing projects
- **Community**: Benefit from webpack's large community

### Why NAPI?

- **Performance**: Native bindings provide low overhead
- **Compatibility**: Works with Node.js ecosystem
- **Type Safety**: Type-safe bindings between Rust and TypeScript

## Extension Points

### Adding a New Plugin

1. Create plugin struct with `#[plugin]` attribute
2. Implement hooks with `#[plugin_hook]` attribute
3. Implement `Plugin` trait
4. Register hooks in `apply` method

### Adding a New Loader

1. Create loader function
2. Register in module rules
3. Implement transformation logic
4. Return transformed code

### Adding a New Hook

1. Define hook using `define_hook!` macro
2. Add to appropriate hooks struct
3. Call hook at appropriate point
4. Plugins can tap into the hook

## Resources

- [Project Structure](../website/docs/en/contribute/development/project.md)
- [Common Patterns](./COMMON_PATTERNS.md)
- [Code Style](./CODE_STYLE.md)
- [Plugin API Documentation](https://rspack.rs/api/plugin-api/)
