# Draft: app cache consumption by a test compiler

## Goal and API

A completed app build should warm the expensive make computations of a later
test build, even when entry points, output and the test compiler's own cache
version differ. This draft changes only `newCache` and requires filesystem cache.
It does not load an app module graph or incremental artifact into the test build.

The added filesystem options are:

- `readonlyFrom: { cacheLocation, version? }`: one completed producer database.
- `loaderProfile?: string`: explicit compatibility contract for loader results.
- `moduleProfile?: string`: stronger contract for complete NormalModule builds.

The source has an independent validator and is opened read-only. The destination
continues to own its cache, metadata, entry traversal and graph. Existing keys and
namespaces are retained. No new persisted-format version or migration is added.

```text
new consumer module / loader input
  -> private cache lookup + validation
  -> read-only source lookup + profile and snapshot validation
  -> promote valid entry to private cache, or compute on miss
  -> consumer's own graph, optimization, code generation and output
```

## Compatibility boundary

Module identifiers are insufficient: a request may contain an options reference
rather than the actual loader options, and parser hooks are not in the identifier.
The optional module etag combines an explicit profile with effective parser,
generator, resolver, loader, match-resource, context and source-map settings. The
profile must cover compiler-wide inputs and arbitrary plugin/hook behavior that
Rspack cannot infer. Modules with loader filters or non-opted-in loaders are not
eligible for full import. `need_build` still validates file and value dependencies.

Loader sharing uses a separate profile plus existing loader names, options,
versions, input content, inherited dependencies and output-dependency snapshots.
Existing loader-cache eligibility restrictions remain in force. Changing the
loader profile also invalidates eligible local module entries, so a local module
hit cannot silently bypass the new transform contract.

An ordinary app parser and Rstest's parser are different contracts. The intended
first integration uses the same loader profile and no shared module profile:
Rstest consumes compatible app transforms, then reparses with RstestPlugin.
Complete make-result sharing is available for compiler pairs whose parser and
hook behavior really is equivalent. A label alone cannot establish that fact.

## Dependency IDs and invalidation after promotion

Each compiler allocates DependencyIds from its own counter. Directly deserializing
app modules into a partly populated consumer graph can overwrite unrelated edges.
The source codec therefore maps every serialized DependencyId to a fresh consumer
ID. One source-wide map preserves repeated references within and between decoded
entries. Local cache decoding keeps IDs unchanged; normal compiler metadata saves
the new high-water mark for the next process. This is implemented in the shared
codec/context and DependencyId converter, without one cache backend depending on
the other. Child compiler imports are deliberately disabled because their ID
allocator and semantic contract are independent.

Promotion must also retain the source's global build dependencies. The consumer
stores these roots with its own validator, including their resolved dependencies,
so later local hits invalidate if the app's tracked configuration or plugin files
change. The producer database remains unchanged even when its validation fails.
Source version mismatches, unavailable entries and stale snapshots are cache misses.

## Validation

The existing Compiler test runner drives separate Node processes for producer,
consumer and warm consumer. The `cache-readonly-from` case covers:

- Different entry points and cache versions; a prepopulated consumer graph plus
  imported synchronous and asynchronous dependencies; execution of emitted code.
- Local persistence of imported modules and loader results.
- Actual RstestPlugin parsing after app loader hits, including source `__filename`.
- Profile, loader-options, parser-options, source-version, source-file and auxiliary
  dependency changes; missing source; overlapping source/destination rejection.
- Source build-dependency invalidation after promotion, without a source configured.
- Byte-for-byte source database preservation, including invalid-source paths.

Assertions observe loader executions and build/still-valid hooks, not timings.

## Before stabilizing

This is a conservative integration prototype, not an automatic cross-config cache.
It requires the same Rspack binary, absolute paths and a closed producer. It offers
one source, root compiler imports and NormalModule/loader tiers only. Concurrent
producer writes, portable imports, multiple sources, resolver/code-generation reuse
and automatic plugin compatibility need separate design. The module fingerprint
currently serializes effective options; a stable canonical hash and avoiding its
second computation on a miss are possible follow-ups after correctness validation.

Rstest itself still needs configuration plumbing and a way to derive/declare a
loader compatibility profile from app and test settings. Browser app transforms
and Node test transforms may not qualify. Benchmark real projects only after that
contract is established; this prototype makes no performance claim.
