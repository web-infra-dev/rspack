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

`LoaderCacheService` remains responsible for cache identity, dependency
validation, serialization, L1 lookup, and L1 replay. `LoaderCacheFileStore`
only provides:

```text
get(hash) -> bytes?
put(hash, bytes)
remove(hash)
```

All file-store failures become misses or ignored writes. They must not fail a
compilation.

## Entry and validity

The persisted JSON envelope contains:

- format version and complete cache identity;
- resource `mtime_ns` and file size;
- cache write timestamp;
- content, source map, and supported dependency data;
- checksum of the encoded payload.

The key hash is only a path lookup. The complete identity is checked after
reading the file.

A hit requires:

```text
current mtime_ns == stored mtime_ns
current size    == stored size
current mtime_ns is not in the future beyond a small tolerance
```

Any timestamp rollback/too-early result, missing file, parse error, checksum
error, identity mismatch, or dependency mismatch is a miss. A resource or
dependency changed between pitch and store, so the candidate is not written.

## Atomic write and lock

The writer creates a sibling lock file with `create_new` and waits briefly when
another process owns it. A stale lock can be removed after a bounded timeout.

Writes use:

```text
create parent directory
write <hash>.json.tmp.<pid>.<random>
flush and close
rename temporary file to <hash>.json
remove lock
```

Readers see either the old complete file or the new complete file. They never
parse a file while it is being written. Corrupt files are removed/quarantined
after a failed read.

## Integration

- Add `experiments.loaderCache?: boolean` to the public TypeScript options,
  defaulting to `false`, and pass it through raw options into Rust
  `Experiments.loader_cache`.
- The NormalModuleFactory checks this experiment before inserting the internal
  cache loader. Disabled experiments retain ordinary loader behavior.
- `PersistentCache` derives the loader-cache root from its existing filesystem
  cache directory and constructs `LoaderCacheFileStore`.
- `MemoryCache` and `DisableCache` continue using memory-only loader cache.
- Remove the loader-specific `StorageRouter`, scope loading, update batching,
  and failed-build cache finalization introduced by the more complex design.
- Keep the plugin/compiler ownership and compiler isolation unchanged.
- File writes are immediate; compiler close is not required for durability.

## Limits of v1

- No TTL/LRU or size budget; versioned directories are the cleanup boundary.
- No remote backend or cross-process generation protocol beyond per-key lock
  and atomic rename.
- Arbitrary `AdditionalData` stays memory-only when it cannot be encoded.
- mtime/size validation is intentionally weaker than content hashing and may
  miss changes hidden by a coarse filesystem timestamp; dependency snapshots
  remain the stronger validation path where available.

## Tests

- cold process/compiler writes, then a second compiler hits the JSON entry;
- unchanged mtime/size hits and changed mtime/size misses;
- timestamps earlier than the stored observation are rejected;
- malformed/truncated/checksum-invalid entries degrade to miss;
- concurrent writers leave one complete valid file;
- write/read/lock failures do not fail compilation;
- temporary files and stale locks are recoverable;
- memory-only cache paths do not touch disk.
