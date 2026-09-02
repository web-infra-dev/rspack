# `rspack_sources` Architecture guide

Use this document when changing `crates/rspack_sources` or a hot caller that constructs, maps,
hashes, serializes, or caches `BoxSource` values. It records cross-module contracts and performance
constraints; inspect the current implementation and tests for local details.

If code and this guide disagree, determine whether the code is intentional or a regression, then
update both in the same change.

## 1. Mental model

`rspack_sources` represents generated assets as composable source graphs. A source can provide:

- generated text or bytes; and
- provenance mapping generated positions to original files, positions, and names.

```text
parser / loader / code generator
              |
              v
   RawSource / OriginalSource / SourceMapSource
              |
              v
     ReplaceSource (byte-range edits)
              |
              v
      ConcatSource (ordered composition)
              |
              v
       CachedSource (stable final graph)
              |
              +----> source / buffer / writer ----> emitted asset
              |
              +----> StreamChunks ----> VLQ encoder ----> source map
```

Content requests enter at the root. Leaves emit borrowed text and local mapping events; transforms
rewrite them; compositions shift positions and remap indices. Materialization should happen only
at an observable boundary such as a flat value, final map, JSON, or emitted bytes.

The memory goal is one retained copy of generated code. Source graphs share backing data and pass
borrowed spans so mapping work avoids per-event heap allocation and caches do not keep another
bundle-sized flat string.

The graph can be a DAG because `BoxSource` is `Arc<dyn Source>`. Build mutable `ConcatSource` and
`ReplaceSource` values first, then box/share/cache them. `CachedSource` assumes its inner graph will
not change and has no invalidation mechanism.

This crate owns source representation, transformation, map streaming, encoding/decoding, and
source-local caching. Output filenames, devtool policy, source-map comments, filesystem lookup,
asset cache policy, and JavaScript conversion belong to callers.

## 2. Hard invariants

### Position units

| Value                                | Unit              | Base / interval                        |
| ------------------------------------ | ----------------- | -------------------------------------- |
| replacement and Rust slice offsets   | UTF-8 bytes       | zero-based, half-open `[start, end)`   |
| `Source::size`, buffer/writer length | bytes             | count                                  |
| generated/original lines             | lines             | one-based                              |
| generated/original columns           | UTF-16 code units | zero-based                             |
| source/name indices                  | table entries     | zero-based and local to a child stream |

Preserve these contracts:

1. **Byte offsets and source-map columns are not interchangeable.** Every replacement offset must
   also be a valid UTF-8 boundary. JavaScript string indices and source-map columns are not valid
   replacement offsets for non-ASCII text.
2. **`GeneratedInfo` is the position after the complete output.** Empty input ends at line 1,
   column 0. A trailing newline ends on the next line at column 0.
3. **Source maps are computed as an ordered event stream.** Avoid flattening children, collecting
   all mappings, cloning source/name strings, or allocating per mapping event.
4. **Child source/name indices are local.** `ConcatSource`, `ReplaceSource`, and combined maps must
   remap them exactly once while preserving event order.
5. **Text and binary content are distinct.** `RawBufferSource::buffer()` and `to_writer()` preserve
   arbitrary bytes; text-oriented rope composition may use lossy UTF-8.
6. **Line-only maps are a separate mode.** They retain at most the useful mapping for each generated
   line and omit names. Do not build a full map and discard columns afterward.
7. **Generated-only content is valid.** The common map builder returns `None` when encoded mappings
   are empty; this says nothing about whether content exists.
8. **Borrowed `'static` views must retain their owner.** The lifetime extensions in `SourceMap`,
   `CachedSource`, and `SourceContentLines` depend on explicit owner relationships.
9. **`Source` remains `Send + Sync`; `ObjectPool` does not.** Use one scratch pool per worker or
   thread, never one shared pool behind a global lock.
10. **Hashing and equality describe the canonical source graph, not rendered output.**
    Persistent-cache restoration must reconstruct the same graph, including intentional
    normalization and transparent wrappers.

## 3. Core API and streaming protocol

### `SourceValue`, `Source`, and `BoxSource`

`SourceValue<'a>` is either `String(Cow<'a, str>)` or `Buffer(Cow<'a, [u8]>)`. Leaves usually
borrow; composites allocate only when a flat result is requested. All sizes are byte sizes.

Despite its historical name:

```rust
pub type BoxSource = Arc<dyn Source>;
```

| Method                      | Use                                       | Cost expectations                           |
| --------------------------- | ----------------------------------------- | ------------------------------------------- |
| `source()`                  | text/binary value                         | borrowed for leaves; may flatten composites |
| `rope(on_chunk)`            | incremental text spans                    | preserve borrowing; text-oriented           |
| `buffer()`                  | exact bytes                               | preferred for binary data                   |
| `size()`                    | output byte length                        | must match buffer/writer length             |
| `map(pool, options)`        | map borrowing from `self`                 | direct borrow or one streamed rebuild       |
| `map_static(pool, options)` | map that outlives the local source borrow | retains an `Arc` owner                      |
| `update_hash()`             | structural hash                           | cached by `CachedSource`                    |
| `to_writer()`               | incremental byte output                   | preferred for emission                      |

For every implementation:

- `size() == buffer().len() == bytes written by to_writer()`;
- text returned by `source()` must agree with `rope()`;
- mappings must describe the same generated content;
- `a == b` must imply equal hashes; hashes need a type discriminator, while deliberate field
  omissions are acceptable collisions.

### Mapping events

`StreamChunks::stream_chunks()` returns a `Chunks` handle. `Chunks::stream()` emits:

- `on_chunk(Option<TextSpan>, Mapping)` for a generated span and the mapping active at its start;
- `on_source(index, name, content)` for local source-table entries;
- `on_name(index, name)` for local name-table entries.

`Mapping` uses one-based lines and zero-based UTF-16 columns. The generated span can be `None` when
only mapping metadata is needed, and a mapping can have no original location.

`MapOptions` selects four specialized paths:

| `columns` | `final_source` | Stream requirement                                   |
| --------- | -------------- | ---------------------------------------------------- |
| `true`    | `false`        | spans plus full mappings for an outer transform      |
| `true`    | `true`         | final full mappings; spans can be omitted            |
| `false`   | `false`        | line spans plus useful line mappings for a transform |
| `false`   | `true`         | final line-only mappings                             |

`final_source` is crate-private and means "feeding the terminal map encoder"; it does not describe
mutability. `get_map` sets it to `true`. `ReplaceSource` requests `false` because it must split
actual spans.

A bare `SourceMapSource` without an inner map directly borrows its supplied map and therefore does
not normalize it for `columns: false`. Wrapping or combining it forces the specialized streaming
paths.

### Retained memory and two-stage streaming

webpack-sources' `CachedSource` retains a flattened `_cachedSource`. Rspack instead stores borrowed
rope chunks in `CachedData::chunks`; the generated bytes remain owned by the source graph. Calling
`source()` may build a temporary `String`, but that string is not retained in the shared cache.

Replaying a cached map still requires the generated text: `stream_chunks_of_source_map` uses it to
compute the final position and, for non-final streams, slice the text passed to `on_chunk`. Keeping
a flat `String` in `CachedSource` would make this cheap but would permanently duplicate the source.

Unlike webpack-sources' direct `streamChunks(options, callbacks)`, Rspack first builds a `Chunks`
calculation tree and then executes it:

```text
Source::stream_chunks()
  -> Chunks tree mirroring the Source graph
  -> Chunks::stream(...callbacks)
```

The extra calculation layer exists to align callback lifetimes:

```rust
fn stream<'chunk>(
  &'chunk self,
  // ...
  on_chunk: OnChunk<'_, 'chunk>,
) -> GeneratedInfo;
```

Text passed to `on_chunk` must live for the same `'chunk` lifetime as `&self`.
`CachedSourceChunks` therefore stores the computed source in `source: OnceCell<Cow<str>>`.
`get_or_init_source()` borrows this field through `&'chunk self`, so the resulting `&str` satisfies
the callback lifetime instead of borrowing a local temporary. The source is released with the
calculation handle.

### `TextSpan` and UTF-16

`TextSpan` carries a borrowed `&str` plus an ASCII hint. Preserve the hint across transformations:

- known ASCII uses byte length as UTF-16 length;
- known non-ASCII uses `simd_utf16_len` directly;
- unknown checks ASCII before using SIMD;
- a subspan of known non-ASCII becomes unknown because the slice may be ASCII-only.

`WithUtf16` converts UTF-16 columns back to valid UTF-8 byte boundaries. It builds a pooled byte
index only for non-ASCII lines; four-byte scalars occupy two UTF-16 columns.

### `SourceMap`

Source maps mainly come from parsed JSON or a `Source` graph. `SourceMap` uses `Cow` fields to reuse
strings owned by these inputs instead of cloning them:

- `from_bytes` retains the JSON buffer that parsed fields borrow from;
- computed maps borrow strings from the source graph, and `into_static(owner)` retains that owner;
- `as_borrowed` creates a cheap borrowed view.

## 4. Source types and file map

Derived results are computed lazily. Source maps are built only when `map()` or `Chunks::stream()`
is requested, and content is materialized only through the corresponding content API.
`CachedSource` is the sole cross-call memoization layer; other types may keep only local
representation optimizations.

### Type selection

| Type              | Role                 | Important behavior                                                  |
| ----------------- | -------------------- | ------------------------------------------------------------------- |
| `RawStringSource` | text leaf            | no mappings; use `from_static` for literals                         |
| `RawBufferSource` | binary leaf          | exact bytes; lazy lossy rope view                                   |
| `OriginalSource`  | mapped text leaf     | lazily creates token-like or line mappings                          |
| `SourceMapSource` | existing/nested map  | directly borrows one map or composes outer and inner maps           |
| `ConcatSource`    | ordered composition  | flattens nested concats, merges adjacent raw strings, remaps tables |
| `ReplaceSource`   | byte-range transform | maintains sorted edits and repairs mappings                         |
| `CachedSource`    | memoization wrapper  | owns cross-call memoized state                                      |

Important implementation details:

- `OriginalSource` splits potential JavaScript tokens with `memchr`; it is not an AST tokenizer.
- `SourceMapSource` composition indexes inner mappings by line and binary-searches the closest
  preceding segment. Avoid cloning maps or eagerly splitting all `sourcesContent`.
- `ConcatSource::add` uses `&mut self` and `Mutex::get_mut`; reads lazily optimize children. Mapping
  shifts generated positions and remaps child-local indices into global deduplicated tables.
- `ReplaceSource` ranges are half-open byte ranges. Edits are ordered by
  `(start, end, enforce, insertion_order)`; ordered producers get the append fast path. Mapping
  repair uses lazy source-content identity checks.
- `CachedSource` shares hash, size, ASCII, rope, full-map, and line-map caches across clones.

### Cache placement and the `ReplaceSource` cost model

`ReplaceSource::size()` intentionally computes the exact byte size from the inner size and the
sorted replacement list on each direct call. `ReplaceSource::source()` uses that result to allocate
the final `String` once at the exact capacity, then traverses `rope()` to fill it. This direct,
uncached path therefore scans replacement metadata for `size()` and processes it again while
rendering. Do not treat that fact alone as evidence that `ReplaceSource` needs an internal size
cache: exact preallocation avoids growth reallocations, while a cache would enlarge every mutable
`ReplaceSource`, duplicate `CachedSource` responsibility, and require invalidation on every edit.

Rspack's normal lifecycle places the cache at the stable graph boundary. Module generation,
concatenated-module output, CSS generation, and chunk rendering generally wrap their completed
source graph in `CachedSource`. The relevant first-call paths are:

```text
CachedSource::source() first
  -> get_or_init_chunks()
       -> inner.rope() once; cache borrowed chunks
  -> CachedSource::size()
       -> sum cached chunk lengths; cache size
       -> does not call ReplaceSource::size()
  -> copy cached chunks into one temporary, exactly sized String

CachedSource::size() first
  -> inner.size() once; cache size
  -> a later source() runs inner.rope() once and reuses the cached size
```

Repeated `CachedSource::size()` calls are O(1). Repeated `source()` calls still copy output bytes,
because `CachedSource` intentionally does not retain a second bundle-sized flat `String`, but they
do not reevaluate the replacement list. If profiling finds repeated direct reads from a completed
`ReplaceSource`, first fix the caller to install the standard `CachedSource` boundary.

The large replacement benchmarks deliberately measure uncached primitives in isolation. The
current complex fixture has roughly 6,118 replacements and includes separate `size()` and
`source()` benchmarks. These cases protect primitive performance and provide a worst-case signal;
they do not model the production cache boundary and are not, by themselves, justification for
moving memoization into `ReplaceSource`.

Also do not propose a "batch append, sort once" API solely from the theoretical cost of
out-of-order insertion. `ReplaceSource` maintains a sorted list after every mutation; `rope()`,
`size()`, map generation, hashing, equality, debug output, `replacements()`, and persistent-cache
serialization can all observe that invariant without a finalization step. `add_replacement` is
already O(1) for ordered producers and uses binary search plus `Vec::insert` only for out-of-order
edits. A deferred sort would add lifecycle state and checks to all read paths and could disturb
`enforce`/insertion-order semantics. Consider it only after a real caller profile identifies vector
shifting as material; first prefer making that producer submit edits in sorted order.

## 5. Ownership and concurrency

### Immutable and stable source data

Source graphs have a build phase followed by a shared read phase. Leaf sources own their content in
`Cow<'static, str>`, `Box<str>`, or `Vec<u8>` and do not mutate it through `Source` methods.
`ConcatSource` can only add children, and `ReplaceSource` can only add edits; both mutation paths
require `&mut self`. After boxing into `Arc<dyn Source>`, the graph is read through shared
references.

`ConcatSource` may lazily build an optimized child list, but it does so before returning borrowed
spans. Moving its `Vec<BoxSource>` does not move child data because each child is held by `Arc`.
These rules keep the strings returned by `rope()` immutable and address-stable.

The `'static` extension in `CachedSource::get_or_init_chunks` is safe only because every clone
sharing the cache also retains the same inner `BoxSource`; the inner graph cannot be mutated or
dropped independently. Adding removal or interior content mutation requires redesigning this
cache.

The same owner-retention pattern appears in other narrowly scoped `unsafe` paths:

- `SourceMap::from_bytes` keeps the parsed byte buffer alive;
- `SourceMap::into_static` keeps the owning `BoxSource` alive;
- `SourceContentLines` stores line views while retaining the backing `Cow<str>`;
- hot unchecked string slicing relies on validated UTF-8 byte boundaries.

Any change to these paths must explain:

1. which allocation owns the bytes;
2. why the bytes cannot move or mutate while borrowed; and
3. why drop order and cloning cannot separate the view from its owner.

Completed sources are `Send + Sync`. Lazy shared state uses `OnceLock`; `ConcatSource` uses a
`Mutex` because read methods receive `&self`. Source streaming itself stays ordered and
synchronous. Parallelize at the module/asset level using Rspack's existing parallel abstractions,
not inside one mapping stream.

`ObjectPool` reuses large `Vec<usize>` buffers for UTF-16 indices. It uses `RefCell`, does not pool
requests below 64 elements, and retains peak capacities. Keep one bounded pool per worker/thread.
