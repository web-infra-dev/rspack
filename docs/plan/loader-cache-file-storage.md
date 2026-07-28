# Simple persistent loader cache

## Scope

Keep the existing compiler-local `FxDashMap` loader cache and add an optional
file backend. Do not reuse `rspack_storage`, add a new public option, or change
the loader-runner state machine.

The feature is guarded by `experiments.loaderCache`, which defaults to
`false`. When the experiment is disabled, `Rule.use.cache` is ignored for
loader execution: no internal cache loader is inserted and no cache storage is
created. Enabling the experiment is required for both memory and persistent
loader caching.

The backend is enabled only for `cache.type = "persistent"`. Its root is the
configured compiler cache directory:

```text
<cache.storage.directory>/loader-cache/v1/<key-hash>.json
```

There is no directory sharding in v1.

## Responsibilities

`LoaderCacheService` remains responsible for cache identity, resource
validation, serialization, L1 lookup, and dependency replay.
`LoaderCacheFileStore` only provides:

```text
get(hash) -> bytes?
put(hash, bytes)
remove(hash)
```

All file-store failures become misses or ignored writes. They must not fail a
compilation.

## Entry and validity

The persisted JSON payload contains:

- format version and complete cache identity;
- compiler scope, including compiler path/name/mode/context and persistent
  cache version;
- resource `mtime_ms` and file size;
- content, source map, and supported dependency data;

The key hash is only a path lookup. The complete identity is checked after
reading the file.

A hit requires:

```text
current mtime_ms == stored mtime_ms
current size     == stored size
```

As in `cache-loader`, a miss records its start time. The result is not cached
when the resource mtime falls in the same second as that start time or later:

```text
resource_mtime_ms / 1000 >= cache_start_ms / 1000
```

This avoids trusting a coarse filesystem timestamp when the resource may have
changed during loader execution. A resource stamp change between pitch and
store also prevents the candidate from being written. Missing files, parse
errors, version mismatches, and identity mismatches are misses.

## Atomic write and lock

The writer creates a sibling lock file with `create_new` and waits briefly when
another process owns it. A timed-out lock attempt skips the cache write; it
does not fail compilation.

Writes use:

```text
create parent directory
write <hash>.json.tmp.<pid>.<sequence>
flush and close
rename temporary file to <hash>.json
remove lock
```

Readers see either the old complete file or the new complete file. They never
parse a file while it is being written. Corrupt files are removed after a
failed read.

## Integration

- Add `experiments.loaderCache?: boolean` to the public TypeScript options,
  defaulting to `false`, and pass it through raw options into Rust
  `Experiments.loader_cache`.
- The NormalModuleFactory checks this experiment before inserting the internal
  cache loader. Disabled experiments retain ordinary loader behavior.
- The compiler derives the loader-cache root from the existing persistent
  filesystem cache options and constructs one compiler-local service.
- `MemoryCache` and `DisableCache` continue using memory-only loader cache.
- Keep the plugin/compiler ownership and compiler isolation unchanged.
- File writes are immediate; compiler close is not required for durability.

## Limits of v1

- No TTL/LRU or size budget.
- No remote backend or cross-process generation protocol beyond per-key lock
  and atomic rename.
- Arbitrary `AdditionalData` stays memory-only when it cannot be encoded.
- mtime/size validation is intentionally weaker than content hashing and may
  miss externally preserved timestamps. Dependency changes are replayed for
  watch/build bookkeeping but are not additional cache-key inputs in v1.

## Tests

- cold process/compiler writes, then a second compiler hits the JSON entry;
- unchanged mtime/size hits and changed mtime/size misses;
- resources modified in the cache-attempt second are not cached;
- malformed/truncated/version-invalid entries degrade to miss;
- concurrent writers leave one complete valid file;
- write/read/lock failures do not fail compilation;
- temporary files do not become cache hits;
- memory-only cache paths do not touch disk.
