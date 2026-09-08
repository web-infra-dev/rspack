# New cache tests

`NewCache.test.js` reuses `createCacheCase` from
`packages/rspack-test-tools/src/case/cache.ts` for three suites:

- `newCacheCases`: dedicated cache cases, using the same fixture format as
  `cacheCases` (`NEXT_HMR`, `NEXT_START`, `NEXT_MOVE_DIR_START`, and `---` file
  update separators).
- `normalCases`: the original normal-case configuration and Node/Web runner.
- `configCases`: the original configuration, including config functions,
  configuration arrays, custom bundle discovery, and Web/ESM runners.

All three enable `cache.type: "persistent"` and `experiments.newCache`. Existing
newCache feature flags are preserved. Each case and each compiler has a private
cache directory under `js/temp`, cleared before its cold build.

After the case's tests finish, cases without a `NEXT_START` automatically restart
once without changing the source. Restarting closes the old compiler to flush
the filesystem cache, creates a fresh compiler, and executes the emitted bundles
again. Explicit restarts keep control of the update steps; a restart with no
remaining file updates simply reuses the current source. `noTests` cases still
perform both builds and cache checks. Bundle hooks are isolated per restart.

Cases that modify build dependencies between compilers can use
`compiler.hooks.beforeCompile.tapPromise`, skipping the initial build.
`NEXT_START` waits for the previous compiler's `close()` to flush its cache
before creating the next compiler, so dependency changes in this hook happen
after the previous build's snapshots are captured. `done` and `afterDone` do not
guarantee this: snapshots can be captured later during idle cache storage.

## Running tests

From `tests/rspack-test`, after `pnpm run build:js` at the repository root:

```sh
# Dedicated fixtures
pnpm test --project base NewCache.test.js

# Compatibility suites
NEW_CACHE_CASES=normalCases pnpm test --project base NewCache.test.js
NEW_CACHE_CASES=configCases pnpm test --project base NewCache.test.js

# Select a compatibility fixture using the usual test filter
NEW_CACHE_CASES=configCases pnpm test --project base NewCache.test.js -t configCases/simple

# All three suites
NEW_CACHE_CASES=all pnpm test --project base NewCache.test.js
```

The default suite is `newCacheCases`. `NEW_CACHE_CASES` also accepts a
comma-separated list of suite names. Compatibility suites are opt-in because
some existing cases still expose newCache serialization gaps, missing
diagnostics, or incomplete cache hits; they are not yet expected to all pass.
Known incompatible fixtures are listed with reasons in `NewCache.test.js`.
Remove an exclusion when its newCache support is fixed. The original Normal,
Config, and Cache suites continue to run these cases with their original settings.

## Cache expectations

Every build writes `cache-stats/N.json` into that case's private directory under
`js/temp`. Reports and cache files stay outside the compilation's source and
output directories so they do not affect context dependencies or asset assertions.
Each array element describes one compiler. Module cache
hits compare cacheable modules against actual `buildModule` calls, since the
stats `built` flag also includes restored modules. Code generation, source-map,
and minimizer hit counts come from the compilation's cache logs. Caches disabled
by the case and caches with no requests are omitted. Percentages use one decimal
place.

Unchanged restarts expect 100% hits for each measured cache. A fixture that
intentionally rebuilds some work can set the expected percentages in
`test.config.js`:

```js
module.exports = {
  cacheHitRate: {
    "module code generation cache": 66.7
  }
};
```

Dedicated cases also check `__snapshots__/cache-stats-N.txt` for every cold build,
HMR update, and restart. This covers invalidation cases whose changed source
should only partially hit the cache. Use `-u` to update these snapshots after
reviewing the expected change. Compatibility runs do not write snapshots into
`normalCases` or `configCases`.

## Legacy cache compatibility

The fixtures imported from `cacheCases` keep their original category names,
configuration, runtime assertions, and file update steps. Their legacy
`rspack.persistentCache` snapshots are replaced by newCache cache-stat snapshots.

Fixtures that cannot pass unchanged remain in this directory and are excluded
by the `cache` list in `NewCache.test.js`, with the observed failure documented
beside each exclusion. These include differences in module restoration and
invalidation, and storage paths expected by legacy tests.
Excluded cases do not keep partial snapshots from failed runs.

To re-enable a case after fixing its compatibility issue, remove its exclusion,
run it with `-u`, review the generated cache-stat snapshots, and run it again
without `-u`. Keep its original runtime assertions when doing so.
