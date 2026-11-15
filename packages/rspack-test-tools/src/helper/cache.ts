import path from 'node:path';
import type { Compiler, MultiStats, RspackOptions, Stats } from '@rspack/core';
import fs from 'fs-extra';
import type { ITestContext } from '../type';
import { checkArrayExpectation } from './legacy/checkArrayExpectation';

export type TCacheTestOptions = {
  directory: string;
};

// Share the marker between ESM and CommonJS imports of this helper.
const CACHE_TEST_PHASE = Symbol.for('@rspack/test-tools/cache-test-phase');
type CacheTestCompiler = Compiler & {
  [CACHE_TEST_PHASE]?: 'cold' | 'warm';
};

/** The runner's cache round, or `normal` outside the persistent-cache check. */
export function getCacheTestPhase(compiler: Compiler) {
  return (compiler as CacheTestCompiler)[CACHE_TEST_PHASE] ?? 'normal';
}

export function getCacheStats(stats: Stats | MultiStats) {
  return ('stats' in stats ? stats.stats : [stats]).map(
    (stats) => stats.toJson({ all: false, cacheInfo: true }).cacheInfo,
  );
}

// Like webpack's cache suites, use fresh compilers with suite-level cache
// options overriding the case's options. Run this after the normal bundle tests.
export async function checkPersistentCache(
  context: ITestContext,
  cache: TCacheTestOptions,
) {
  const manager = context.getCompiler();
  if (!manager.getStats()) return;

  await manager.close();
  const options = manager.getOptions();
  const cacheDirectory = cache.directory;
  fs.removeSync(cacheDirectory);
  const withCache = (
    options: RspackOptions,
    index: number,
    phase: 'cold' | 'warm',
  ): RspackOptions => ({
    ...options,
    plugins: [
      (compiler: Compiler) => {
        (compiler as CacheTestCompiler)[CACHE_TEST_PHASE] = phase;
      },
      ...(options.plugins ?? []),
    ],
    cache: {
      type: 'persistent',
      storage: {
        type: 'filesystem',
        directory: path.join(cacheDirectory, String(index)),
      },
    },
    experiments: {
      ...options.experiments,
      newCache: {
        codeGeneration: true,
        module: true,
        devtool: true,
        loader: true,
        minimize: true,
        resolver: true,
      },
    },
  });
  let coldCacheStats: ReturnType<typeof getCacheStats> = [];
  for (let buildIndex = 0; buildIndex < 2; buildIndex++) {
    // close() must finish writing the disk cache before the next compiler.
    // The last compiler is closed by the harness teardown, including on failure.
    if (buildIndex > 0) await manager.close();
    const phase = buildIndex === 0 ? 'cold' : 'warm';
    manager.setOptions(
      (Array.isArray(options)
        ? options.map((options, index) => withCache(options, index, phase))
        : withCache(options, 0, phase)) as RspackOptions,
    );
    manager.createCompiler();
    await manager.build();
    const stats = manager.getStats()!;
    const cacheStats = getCacheStats(stats);
    fs.outputJsonSync(
      path.join(cacheDirectory, 'stats', `${buildIndex}.json`),
      cacheStats,
      { spaces: 2 },
    );
    const diagnostics = stats.toJson({
      all: false,
      errors: true,
      errorsCount: true,
      warnings: true,
      errorDetails: true,
      moduleTrace: true,
    });
    for (const kind of ['error', 'warning'] as const) {
      await checkArrayExpectation(
        context.getSource(),
        diagnostics,
        kind,
        `${kind}s`,
        kind === 'error' ? 'Error' : 'Warning',
        manager.getOptions(),
      );
    }
    if (buildIndex === 0) {
      coldCacheStats = cacheStats;
      continue;
    }

    // Like webpack's ConfigTestCases, only check cache reuse when the whole
    // build, including all MultiCompiler children, has no errors.
    if (diagnostics.errorsCount !== 0) continue;

    for (const [index, info] of cacheStats.entries()) {
      if (!info || Object.keys(info).length === 0) {
        throw new Error(`Compiler ${index} did not report cacheInfo`);
      }
      for (const [namespace, counters] of Object.entries(info)) {
        const lookups = counters.hits + counters.misses;
        if (lookups === 0) continue;
        if (counters.reused !== lookups) {
          const reuseRate = counters.reused / lookups;
          throw new Error(
            `Compiler ${index}: ${namespace} cache reuse rate is ${(reuseRate * 100).toFixed(2)}%, expected 100%:\n${JSON.stringify(
              {
                cold: coldCacheStats[index]?.[namespace],
                restored: counters,
              },
              null,
              2,
            )}`,
          );
        }
      }
    }
  }
}
