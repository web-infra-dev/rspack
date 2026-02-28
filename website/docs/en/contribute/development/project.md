# Project architecture

This is a **monorepo** containing both Rust crates and JavaScript packages:

## Rust crates (`crates/`)

### Core crates

- **`rspack`**: Main crate that integrates all core functionality and plugins, providing the complete build tool entry point
- **`rspack_core`**: Core engine containing module system, dependency graph, compilation pipeline, and other core functionality
- **`rspack_binding_api`**: Node.js binding API that bridges Rust core functionality to JavaScript/TypeScript interfaces
- **`node_binding`**: Node.js binding implementation that generates Node.js native modules
- **`rspack_napi`**: NAPI (Node-API) support layer for interoperability between Rust and Node.js
- **`rspack_allocator`**: Memory allocator using mimalloc to optimize memory allocation performance (Linux/macOS)

### Build & binding crates

- **`rspack_binding_build`**: Binding build script for building Node.js native bindings
- **`rspack_binding_builder`**: Binding builder for generating custom Rspack bindings
- **`rspack_binding_builder_macros`**: Binding builder macros providing procedural macro support
- **`rspack_binding_builder_testing`**: Binding builder testing utilities

### Utility crates

- **`rspack_error`**: Error handling and formatting, providing user-friendly error messages
- **`rspack_fs`**: File system abstraction layer providing cross-platform file operation interfaces
- **`rspack_paths`**: Path utilities for path normalization, resolution, and manipulation
- **`rspack_hash`**: Hash algorithm implementations supporting MD4, SHA2, xxhash, etc.
- **`rspack_regex`**: Regular expression utilities providing high-performance regex matching
- **`rspack_location`**: Location information handling for source code position tracking
- **`rspack_ids`**: ID generation and management for creating unique identifiers for modules, chunks, etc.
- **`rspack_collections`**: Collection data structures providing optimized HashMap, HashSet, etc.
- **`rspack_util`**: Utility function collection containing various helper functions
- **`rspack_futures`**: Async utilities providing asynchronous programming support
- **`rspack_workspace`**: Workspace support for handling monorepo scenarios

### Caching & storage

- **`rspack_cacheable`**: Caching system providing serialization and deserialization for cacheable data
- **`rspack_cacheable_macros`**: Caching system macros for auto-generating cache-related code
- **`rspack_cacheable_test`**: Caching system tests
- **`rspack_storage`**: Storage abstraction layer providing persistent storage interfaces

### Hook system

- **`rspack_hook`**: Hook system implementing plugin hook registration and invocation mechanisms
- **`rspack_macros`**: Procedural macros providing various compile-time code generation features
- **`rspack_macros_test`**: Macro system tests

### Compilation & transformation

- **`rspack_javascript_compiler`**: JavaScript compiler for processing JS/TS code compilation
- **`rspack_loader_runner`**: Loader runner for executing various loaders to process resources
- **`rspack_loader_swc`**: SWC loader using SWC for code transformation
- **`rspack_loader_lightningcss`**: Lightning CSS loader for processing CSS files
- **`rspack_loader_react_refresh`**: React Fast Refresh loader supporting React hot module replacement
- **`rspack_loader_preact_refresh`**: Preact Refresh loader supporting Preact hot module replacement
- **`rspack_loader_testing`**: Loader testing utilities

### Plugin system

#### Core plugins

- **`rspack_plugin_javascript`**: JavaScript plugin for parsing, transforming, and code generation of JS/TS modules
- **`rspack_plugin_runtime`**: Runtime plugin generating webpack-compatible runtime code
- **`rspack_plugin_entry`**: Entry plugin for handling entry point configuration
- **`rspack_plugin_dynamic_entry`**: Dynamic entry plugin supporting dynamic entry points

#### Asset & resource plugins

- **`rspack_plugin_asset`**: Asset plugin for processing static assets
- **`rspack_plugin_copy`**: Copy plugin for copying files to output directory
- **`rspack_plugin_json`**: JSON plugin for processing JSON files
- **`rspack_plugin_wasm`**: WebAssembly plugin for processing WASM modules
- **`rspack_plugin_html`**: HTML plugin for generating HTML files

#### CSS plugins

- **`rspack_plugin_css`**: CSS plugin for processing CSS modules and styles
- **`rspack_plugin_extract_css`**: CSS extraction plugin for extracting CSS to separate files
- **`rspack_plugin_css_chunking`**: CSS code splitting plugin
- **`rspack_plugin_lightning_css_minimizer`**: Lightning CSS minifier plugin

#### Optimization plugins

- **`rspack_plugin_swc_js_minimizer`**: SWC JavaScript minifier plugin
- **`rspack_plugin_split_chunks`**: Code splitting plugin implementing chunk splitting strategies
- **`rspack_plugin_merge_duplicate_chunks`**: Merge duplicate chunks plugin
- **`rspack_plugin_remove_empty_chunks`**: Remove empty chunks plugin
- **`rspack_plugin_remove_duplicate_modules`**: Remove duplicate modules plugin
- **`rspack_plugin_limit_chunk_count`**: Limit chunk count plugin
- **`rspack_plugin_real_content_hash`**: Real content hash plugin generating hashes based on content

#### Development plugins

- **`rspack_plugin_hmr`**: Hot Module Replacement (HMR) plugin
- **`rspack_plugin_devtool`**: Source Map plugin for generating source maps
- **`rspack_plugin_progress`**: Progress display plugin
- **`rspack_plugin_lazy_compilation`**: Lazy compilation plugin

#### Library & module plugins

- **`rspack_plugin_library`**: Library plugin for generating library files
- **`rspack_plugin_esm_library`**: ESM library plugin for generating ES module format libraries
- **`rspack_plugin_externals`**: Externals plugin for excluding external dependencies
- **`rspack_plugin_module_replacement`**: Module replacement plugin supporting module aliases
- **`rspack_plugin_ignore`**: Ignore plugin for ignoring specific modules

#### Advanced features

- **`rspack_plugin_mf`**: Module Federation plugin implementing micro-frontend module federation
- **`rspack_plugin_dll`**: DLL plugin implementing dynamic link library functionality
- **`rspack_plugin_worker`**: Web Worker plugin for processing Worker files
- **`rspack_plugin_web_worker_template`**: Web Worker template plugin
- **`rspack_plugin_schemes`**: Custom scheme plugin supporting custom resource protocols
- **`rspack_plugin_runtime_chunk`**: Runtime chunk plugin for separating runtime code

#### Utility plugins

- **`rspack_plugin_ensure_chunk_conditions`**: Ensure chunk conditions plugin
- **`rspack_plugin_no_emit_on_errors`**: No emit on errors plugin
- **`rspack_plugin_circular_dependencies`**: Circular dependency detection plugin
- **`rspack_plugin_banner`**: Banner plugin for adding file header comments
- **`rspack_plugin_size_limits`**: Size limits plugin for checking bundle sizes
- **`rspack_plugin_sri`**: Subresource Integrity (SRI) plugin
- **`rspack_plugin_module_info_header`**: Module info header plugin
- **`rspack_plugin_warn_sensitive_module`**: Warn sensitive module plugin

#### Debug & testing plugins

- **`rspack_plugin_rsdoctor`**: RsDoctor plugin providing debugging and diagnostic functionality
- **`rspack_plugin_rslib`**: RsLib plugin for library builds
- **`rspack_plugin_rstest`**: RsTest plugin for testing

### Browser & environment support

- **`rspack_browser`**: Browser environment support providing browser-side implementations
- **`rspack_browserslist`**: Browserslist support for handling browser compatibility queries

### Monitoring & tracing

- **`rspack_tracing`**: Tracing system providing performance tracing functionality
- **`rspack_tracing_perfetto`**: Perfetto tracing support integrating Perfetto performance analysis tools
- **`rspack_watcher`**: File watcher monitoring file changes to trigger rebuilds
- **`rspack_tasks`**: Task system for managing build tasks

### SWC plugins

- **`swc_plugin_import`**: SWC import plugin for processing module import transformations
- **`swc_plugin_ts_collector`**: SWC TypeScript collector plugin for collecting TS type information

## NPM packages (`packages/`)

### Core packages

- **`@rspack/core`**: Main JavaScript/TypeScript package that provides webpack-compatible API, wrapping the Rust core functionality and exposing the complete build tool interface for Node.js applications

### CLI tools

- **`@rspack/cli`**: Command-line interface providing build, serve, and preview commands for running Rspack from the terminal
- **`create-rspack`**: Project scaffolding tool for creating new Rspack projects with various templates (vanilla, React, Vue) supporting both JavaScript and TypeScript

### Browser support

- **`@rspack/browser`**: Browser-compatible version of Rspack that can run in browser environments using WebAssembly, currently in early development stage

### Test tools

- **`@rspack/test-tools`**: Testing utilities and helper functions for writing and running Rspack tests, including support for WebAssembly test execution

## Test cases (`tests/`)

### Core test suite (`rspack-test/`)

The main test suite for Rspack core functionality, containing various test types that simulate the build process:

#### Test types

- **`normalCases/`** (`Normal.test.js`, `Normal-dev.test.js`, `Normal-hot.test.js`, `Normal-prod.test.js`): Test cases for core build processes without configuration changes, used when testing does not require `rspack.config.js`
- **`configCases/`** (`Config.part1.test.js`, `Config.part2.test.js`, `Config.part3.test.js`): Test cases for build configuration options, allowing specification of build configuration through `rspack.config.js` and test behavior through `test.config.js`
- **`hotCases/`** (`HotNode.test.js`, `HotWeb.test.js`, `HotWorker.test.js`, `HotSnapshot.hottest.js`): Test cases for Hot Module Replacement (HMR) functionality, including HotNode (`target=async-node`), HotWeb (`target=web`), and HotWorker (`target=webworker`)
- **`watchCases/`** (`Watch.part1.test.js`, `Watch.part2.test.js`, `Watch.part3.test.js`): Test cases for incremental compilation in Watch mode, using numbered directories (0, 1, 2...) to represent change batches
- **`statsOutputCases/`** (`StatsOutput.test.js`): Test cases for console output logs after build completion, with snapshots stored in `__snapshots__/StatsOutput.test.js.snap`
- **`statsAPICases/`** (`StatsAPI.test.js`): Test cases for the Stats object generated after build completion, using `tests/rspack-test/fixtures` as source code
- **`diagnosticsCases/`** (`Diagnostics.test.js`): Test cases for formatted warning/error output during the build process, with snapshots stored in `stats.err` files
- **`hashCases/`** (`Hash.test.js`): Test cases for hash generation functionality, validating hash information in the Stats object
- **`compilerCases/`** (`Compiler.test.js`): Test cases for Compiler/Compilation object APIs, using `tests/rspack-test/fixtures` as source code
- **`defaultsCases/`** (`Defaults.test.js`): Test cases for configuration option interactions, generating build configurations and observing differences from defaults
- **`errorCases/`** (`Error.test.js`): Test cases for `compilation.errors` and `compilation.warnings` interactions
- **`hookCases/`** (`Hook.test.js`): Test cases for various hook functionalities, recording hook input/output in snapshots
- **`treeShakingCases/`** (`TreeShaking.test.js`): Test cases for Tree Shaking-related features, with product snapshots stored in `__snapshots__/treeshaking.snap.txt`
- **`builtinCases/`** (`Builtin.test.js`): Test cases for plugins with built-in native implementations, generating different snapshots based on plugin type (CSS, CSS modules, HTML, JavaScript)
- **`cacheCases/`** (`Cache.test.js`): Test cases for caching functionality, including common cache, invalidation, snapshot, storage, and portable cache scenarios
- **`serialCases/`** (`Serial.test.js`): Test cases for serial execution scenarios
- **`exampleCases/`** (`Example.test.js`): Example test cases demonstrating various Rspack features
- **`esmOutputCases/`** (`EsmOutput.test.js`): Test cases for ES module output functionality, including basic output, deconflict, dynamic import, externals, interop, namespace, preserve-modules, re-exports, and split-chunks scenarios
- **`multiCompilerCases/`** (`MultiCompiler.test.js`): Test cases for multi-compiler scenarios
- **Incremental tests** (`Incremental-node.test.js`, `Incremental-async-node.test.js`, `Incremental-web.test.js`, `Incremental-webworker.test.js`, `Incremental-watch.test.js`): Test cases for incremental compilation targeting different environments
- **Native watcher tests** (`NativeWatcher.part1.test.js`, `NativeWatcher.part2.test.js`, `NativeWatcher.part3.test.js`): Test cases for native file watcher functionality

#### Supporting directories

- **`fixtures/`**: General test files and shared fixtures used across multiple test types
- **`js/`**: Build artifacts and temporary files generated during test execution, organized by test type (e.g., `js/normal`, `js/config`, `js/hot-{target}`)
- **`__snapshots__/`**: Test snapshots for various test types, including StatsOutput, HotSnapshot, and other snapshot-based tests

### E2E Tests (`e2e/`)

End-to-end tests for Rspack, covering real-world scenarios and integration testing:

- **`cases/`**: E2E test cases organized by feature area (chunk, css, file, hooks, html, incremental, lazy-compilation, make, module-federation, persistent-cache, react, vue3, worker)
- **`fixtures/`**: Shared fixtures and utilities for E2E tests
- **`utils/`**: Utility functions for E2E test execution

### Benchmarks (`bench/`)

Performance benchmarks for tracking Rspack JavaScript API performance and preventing performance degradation:

- **`fixtures/`**: Benchmark test fixtures (e.g., `ts-react` project for benchmarking)
- Benchmark files for measuring build performance and API execution time
