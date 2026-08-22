# Rspack

Rspack is the JavaScript bundling context for this repository. Use this glossary when discussing bundling behavior, webpack compatibility, extension points, and generated output.

## Language

### Product and compatibility

**Rspack**:
The JavaScript bundler maintained by this repository. Its product promise is high-performance bundling with strong webpack ecosystem compatibility.
_Avoid_: rspack, Rspack bundler

**Rstack**:
The broader JavaScript toolchain ecosystem centered on Rspack, including related tools such as Rsbuild and Rslib.
_Avoid_: Rspack ecosystem, RsPack stack

**Webpack compatibility**:
The property that an API, option, plugin, loader, runtime behavior, or output behavior matches webpack closely enough for existing webpack users and integrations to rely on it.
_Avoid_: Webpack parity, compatible enough

### Compilation model

**Compiler**:
The long-lived orchestrator for configuration, plugin registration, file systems, watch mode, and creation of compilations.
_Avoid_: Builder, runner

**Compilation**:
A single build instance that owns module processing, graph construction, optimization, chunk creation, and asset generation.
_Avoid_: Build, compiler run

**Entry point**:
A configured starting point from which Rspack begins discovering modules for a compilation.
_Avoid_: Entry file, app root

**Make phase**:
The compilation phase where modules are built and the module graph is constructed.
_Avoid_: Graph build, dependency collection

**Seal phase**:
The compilation phase where no more modules are added and the compilation is finalized for optimization and output planning.
_Avoid_: Finalize phase, close phase

**Emit phase**:
The compilation phase where generated assets are written to the output file system.
_Avoid_: Write phase, output phase

### Module graph

**Module**:
A source or generated unit that Rspack can parse, transform, connect to other modules, and include in generated output.
_Avoid_: File, script

**Normal module**:
A regular JavaScript, TypeScript, CSS, or asset module processed through the normal module pipeline.
_Avoid_: Regular module, source module

**Context module**:
A module created to represent a dynamic require context, such as a directory pattern discovered from `require.context`.
_Avoid_: Dynamic module, directory module

**External module**:
A module reference that Rspack leaves outside the bundle and expects the runtime environment to provide.
_Avoid_: Excluded module, CDN module

**Runtime module**:
A generated module that provides bundler runtime behavior such as module loading, chunk loading, or Hot Module Replacement support.
_Avoid_: Runtime code, bootstrap code

**Dependency**:
A relationship discovered from an entry point or module to another requested resource.
_Avoid_: Edge, import

**Module graph connection**:
The resolved link in the module graph that connects a dependency from its origin to the module it resolves to.
_Avoid_: Dependency edge, resolved dependency

**Module graph**:
The graph of modules, dependencies, and module graph connections built during compilation.
_Avoid_: Dependency graph, import graph

### Chunk and asset model

**Chunk**:
A group of modules that Rspack plans and emits together as part of the output loading strategy.
_Avoid_: Bundle, output file

**Entry chunk**:
A chunk generated from an entry point and loaded as part of starting the application.
_Avoid_: Main bundle, entry bundle

**Async chunk**:
A chunk loaded on demand, commonly produced by dynamic import or code splitting.
_Avoid_: Lazy bundle, split bundle

**Runtime chunk**:
A chunk that carries runtime modules separately from application modules.
_Avoid_: Runtime bundle, bootstrap bundle

**Chunk graph**:
The graph that records chunk relationships and which modules belong to which chunks.
_Avoid_: Bundle graph, output graph

**Asset**:
An output artifact produced by compilation, such as JavaScript, CSS, images, source maps, or generated HTML.
_Avoid_: File, emitted file

**Asset info**:
Metadata attached to an asset, such as size, related assets, or output hints.
_Avoid_: Asset metadata, file metadata

### Extension model

**Plugin**:
An extension that hooks into the compiler or compilation lifecycle to observe or modify bundling behavior.
_Avoid_: Extension, addon

**Builtin plugin**:
A core plugin shipped by Rspack as part of its built-in behavior.
_Avoid_: Internal plugin, native plugin

**Hook**:
A named lifecycle point where plugins can tap callbacks to observe or change compiler or compilation behavior.
_Avoid_: Event, callback point

**Tap**:
The act of registering a plugin callback on a hook.
_Avoid_: Listener registration, callback registration

**Loader**:
A transform that processes a module resource before Rspack parses it and adds it to the module graph.
_Avoid_: Transformer, preprocessor

**Loader chain**:
The ordered sequence of loaders applied to a module resource.
_Avoid_: Loader pipeline, transform chain

**Loader context**:
The object exposed to a loader that provides metadata, options, dependency registration, and other loader utilities.
_Avoid_: Loader this, loader API object

**Pitch loader**:
A loader behavior that runs in the pitch phase before the normal loader chain executes.
_Avoid_: Pre-loader, early loader

### Resolution and optimization

**Module resolution**:
The process of turning a module request into the concrete module or external reference used by the compilation.
_Avoid_: Path lookup, import resolution

**Tree shaking**:
The optimization that removes unused exports by analyzing how modules import and export values.
_Avoid_: Dead code elimination

**Code splitting**:
The optimization that divides code into multiple chunks to control loading and caching behavior.
_Avoid_: Bundle splitting, chunk splitting

**Scope hoisting**:
The optimization that concatenates compatible modules into a shared scope to reduce runtime overhead.
_Avoid_: Module concatenation

**Persistent cache**:
The disk-backed cache that preserves reusable compilation work across process runs.
_Avoid_: Disk cache, build cache

### Development behavior

**Hot Module Replacement**:
Development behavior that updates affected modules in a running application without a full page reload.
_Avoid_: Hot reload, live reload

**Watch mode**:
Development behavior where Rspack monitors input changes and starts new compilations automatically.
_Avoid_: File watching, watch build

**Dev server**:
The development server that serves generated output and coordinates development features such as Hot Module Replacement.
_Avoid_: Local server, development server
