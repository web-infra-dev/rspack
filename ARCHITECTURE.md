# Architecture

This document describes the high-level architecture of Rspack, including the core components, data flow, and design decisions.

## Overview

Rspack is a high-performance JavaScript bundler written in Rust that maintains strong compatibility with the webpack ecosystem. The architecture is designed to leverage Rust's performance while providing a webpack-compatible API through Node.js bindings.

## High-Level Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    JavaScript/TypeScript Layer               │
│  (@rspack/core, @rspack/cli, plugins, loaders)               │
└──────────────────────┬──────────────────────────────────────┘
                       │ NAPI (Node-API)
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                    Rust Core Layer                          │
│  (rspack_core, plugins, loaders, compilation engine)       │
└─────────────────────────────────────────────────────────────┘
```

### Layer Separation

1. **JavaScript/TypeScript Layer** (`packages/`)
   - Provides webpack-compatible API
   - Handles configuration and plugin registration
   - Manages file system operations
   - Bridges to Rust core via NAPI

2. **Rust Core Layer** (`crates/`)
   - Core compilation engine
   - Module system and dependency graph
   - Plugin and loader execution
   - Code transformation and optimization

3. **Binding Layer** (`crates/rspack_binding_api`, `crates/rspack_napi`)
   - NAPI bindings for Rust-JavaScript interop
   - Type-safe API translation
   - Memory management

## Core Components

### Compiler

The `Compiler` is the main entry point that orchestrates the build process.

**Responsibilities:**

- Configuration management
- Plugin registration and execution
- Compilation lifecycle management
- File system abstraction
- Watch mode support

**Key Hooks:**

- `beforeRun`: Before starting compilation
- `run`: Start compilation
- `compile`: Create compilation
- `make`: Build modules
- `emit`: Write assets to output
- `done`: Compilation complete

### Compilation

The `Compilation` represents a single build instance and manages the module graph, chunks, and assets.

**Responsibilities:**

- Module graph construction
- Dependency resolution
- Chunk creation and optimization
- Asset generation
- Code generation

**Key Data Structures:**

- `ModuleGraph`: Module graph tracking all modules, dependencies, and connections
- `ChunkGraph`: Relationship between chunks and modules
- `Assets`: Output files and their content
- `ChunkByUkey`: Chunk storage and lookup

**Key Hooks:**

- `buildModule`: Build individual modules
- `succeedModule`: Module built successfully
- `processAssets`: Process and transform assets
- `optimizeChunks`: Optimize chunk structure
- `afterSeal`: After sealing compilation

### Module System

Modules are the basic unit of code organization in Rspack.

**Module Types:**

- `NormalModule`: Regular JavaScript/TypeScript modules
- `ContextModule`: Dynamic require contexts
- `ExternalModule`: External dependencies
- `ConcatenatedModule`: Concatenated modules for optimization

**Module Lifecycle:**

1. **Parse**: Parse source code into AST
2. **Build**: Resolve dependencies and build module
3. **Code Generation**: Generate runtime code
4. **Seal**: Finalize module (no more changes)

### Module Graph

The Module Graph manages modules and their relationships through dependencies and connections.

**Key Concepts:**

- **Dependency**: Represents a relationship between modules (e.g., import, require). Each dependency has a `DependencyId` and can be of different types (ModuleDependency, ContextDependency, etc.)
- **Connection**: A `ModuleGraphConnection` represents the actual link between modules, containing:
  - `dependency_id`: The dependency that created this connection
  - `original_module_identifier`: The module that references another module
  - `resolved_module`: The module being referenced
- **Module Graph**: The central data structure (`ModuleGraph`) that tracks:
  - All modules in the compilation
  - All dependencies between modules
  - All connections (links) between modules
  - Export/import information

**Graph Construction:**

1. Start from entry points
2. Parse modules to discover dependencies
3. Create `Dependency` objects for each import/require
4. Resolve dependencies to target modules
5. Create `ModuleGraphConnection` objects linking modules
6. Build complete module graph with all relationships

### Chunk System

Chunks are groups of modules that are bundled together.

**Chunk Types:**

- **Entry Chunks**: Generated from entry points
- **Async Chunks**: Code-split chunks loaded on demand
- **Runtime Chunks**: Webpack runtime code
- **Vendor Chunks**: Third-party dependencies

**Chunk Splitting:**

- Controlled by `optimization.splitChunks`
- Groups modules based on criteria (size, cache groups, etc.)
- Creates separate chunks for better caching

## Compilation Pipeline

The compilation process follows these stages:

```text
1. Initialize
   ├── Load configuration
   ├── Create compiler instance
   └── Register plugins

2. Compile
   ├── Create compilation
   ├── Build module graph
   │   ├── Parse entry modules
   │   ├── Resolve dependencies
   │   └── Build all modules
   ├── Optimize
   │   ├── Tree shaking
   │   ├── Code splitting
   │   └── Minification
   └── Generate
       ├── Code generation
       ├── Asset creation
       └── Output to filesystem
```

### Detailed Stages

#### 1. Initialize Phase

- Load and normalize configuration
- Create compiler instance
- Apply plugins
- Initialize file systems
- Set up hooks

#### 2. Compile Phase

**Entry Processing:**

- Process entry points from configuration
- Create entry modules
- Add to module graph

**Module Building:**

- Parse source code (using SWC)
- Extract dependencies
- Transform code (using loaders)
- Build module metadata

**Dependency Resolution:**

- Resolve module paths
- Handle aliases and extensions
- Process externals
- Create dependency edges in graph

#### 3. Optimization Phase

**Tree Shaking:**

- Analyze module exports/imports
- Remove unused code
- Mark side effects

**Code Splitting:**

- Analyze chunk dependencies
- Split chunks based on configuration
- Create async chunks for dynamic imports

**Minification:**

- Minify JavaScript (using SWC)
- Minify CSS (using Lightning CSS)
- Optimize asset sizes

#### 4. Code Generation Phase

**Runtime Code:**

- Generate webpack runtime
- Create module loading code
- Generate HMR code (if enabled)

**Asset Generation:**

- Generate output files
- Apply filename templates
- Generate source maps
- Process assets through plugins

## Plugin System

### Plugin Architecture

Plugins extend Rspack's functionality by hooking into the compilation lifecycle.

**Plugin Types:**

- **Builtin Plugins**: Core functionality (JavaScript, CSS, HTML, etc.)
- **User Plugins**: Custom plugins via configuration
- **External Plugins**: webpack-compatible plugins

**Plugin Registration:**

```rust
impl Plugin for MyPlugin {
  fn apply(&self, ctx: &mut ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.process_assets.tap(process_assets::new(self));
    Ok(())
  }
}
```

### Hook System

Hooks allow plugins to intercept and modify the compilation process.

**Hook Types:**

- **SyncSeries**: Synchronous, sequential execution
- **SyncSeriesBail**: Synchronous, can bail out early
- **AsyncSeries**: Asynchronous, sequential execution
- **AsyncSeriesBail**: Asynchronous, can bail out early
- **AsyncParallel**: Asynchronous, parallel execution

**Hook Usage:**

```rust
#[plugin_hook(CompilationProcessAssets for MyPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  // Process assets
  Ok(())
}
```

## Loader System

Loaders transform source code before it's added to the dependency graph.

**Loader Execution:**

1. Loader chain is determined by module rules
2. Loaders execute in reverse order (last to first)
3. Each loader receives previous loader's output
4. Final output is parsed and added to module graph

**Loader Types:**

- **Builtin Loaders**: SWC loader, Lightning CSS loader, etc.
- **JavaScript Loaders**: Custom loaders written in JavaScript
- **Rust Loaders**: High-performance loaders written in Rust

## Module Resolution

Module resolution determines how module paths are resolved to actual files.

**Resolution Process:**

1. Check if module is external
2. Apply aliases and extensions
3. Resolve using enhanced-resolve
4. Handle package.json exports/imports
5. Return resolved path

**Resolution Strategies:**

- **Relative**: `./module`, `../module`
- **Absolute**: `/path/to/module`
- **Module**: `module-name` (from node_modules)
- **Alias**: Custom alias mappings

## Caching System

Rspack uses a multi-level caching system for performance.

**Cache Levels:**

1. **Memory Cache**: In-memory cache for current build
2. **Persistent Cache**: Disk-based cache across builds
3. **Module Cache**: Cached module build results
4. **Compilation Cache**: Cached compilation results

**Cache Invalidation:**

- File content changes
- Configuration changes
- Dependency changes
- Manual cache clearing

## File System Abstraction

Rspack uses a file system abstraction layer for cross-platform support.

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

Errors are handled through a unified error system.

**Error Types:**

- **Build Errors**: Module build failures
- **Resolution Errors**: Module resolution failures
- **Compilation Errors**: Compilation process errors
- **Plugin Errors**: Plugin execution errors

**Error Propagation:**

- Errors are collected in `compilation.errors`
- Warnings are collected in `compilation.warnings`
- Errors can be formatted with context and suggestions

## Performance Optimizations

### Parallel Processing

- Module building is parallelized
- Asset processing can be parallelized
- Code generation uses parallel workers

### Incremental Compilation

- Only rebuilds changed modules
- Uses dependency graph to determine affected modules
- Caches unchanged modules

### Memory Management

- Uses mimalloc for optimized memory allocation (Linux/macOS)
- Efficient data structures (custom HashMap, HashSet)
- Minimizes allocations in hot paths

## Data Flow

### Build Request Flow

```text
User Code
  ↓
Configuration
  ↓
Compiler.apply()
  ↓
Plugin Registration
  ↓
Compiler.run()
  ↓
Compilation Creation
  ↓
Module Graph Building
  ↓
Optimization
  ↓
Code Generation
  ↓
Asset Emission
  ↓
Output Files
```

### Module Processing Flow

```text
Source File
  ↓
Loader Chain
  ↓
Parsed AST
  ↓
Dependency Extraction
  ↓
Module Graph Node
  ↓
Code Generation
  ↓
Runtime Code
```

## Design Decisions

### Why Rust?

- **Performance**: Rust provides near-C performance with memory safety
- **Concurrency**: Excellent async/await support for parallel processing
- **Ecosystem**: Rich ecosystem for parsing, transformation, and optimization

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
3. Call hook at appropriate point in compilation
4. Plugins can now tap into the hook

## Resources

- [Project Structure](./website/docs/en/contribute/development/project.md)
- [Common Patterns](./COMMON_PATTERNS.md)
- [Code Style](./CODE_STYLE.md)
- [Plugin API Documentation](https://rspack.rs/api/plugin-api/)
