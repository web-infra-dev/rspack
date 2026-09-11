import path from 'node:path';
import rspack, {
  type Compilation,
  type Compiler,
  type MultiStats,
  type RspackOptions,
  type Stats,
  type StatsCompilation,
  type StatsModule,
} from '@rspack/core';
import fs from 'fs-extra';
import { isJavaScript } from '../helper';
import { normalizePlaceholder } from '../helper/expect/placeholder';
import { HotUpdatePlugin } from '../helper/hot-update';
import { checkArrayExpectation } from '../helper/legacy/checkArrayExpectation';
import { NodeRunner } from '../runner';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv, ITestProcessor } from '../type';
import { afterExecute, build, check, compiler, config, run } from './common';
import { createConfigProcessor } from './config';
import { createNormalProcessor } from './normal';
import {
  cachedStats,
  createMultiCompilerRunner,
  createRunner as createCaseRunner,
  getMultiCompilerRunnerKey,
} from './runner';

type TTarget = RspackOptions['target'];

export type TCacheCaseOptions = {
  /** Reuse the configuration and runner of the original case suite. */
  type?: 'cache' | 'normal' | 'config';
  /** Enable persistent newCache, restart unchanged cases, and check cache hits. */
  newCache?: boolean;
};

type CacheStats = Record<
  string,
  { hits: number; total: number; hitRate: number }
>;

const builtModulesByCompilation = new WeakMap<Compilation, Set<string>>();

interface CacheState {
  compilerIndex: number;
  nextStart: (advance?: boolean) => Promise<StatsCompilation>;
  nextHmr: (module: any, options?: any) => Promise<StatsCompilation>;
}

function createCacheProcessor(
  name: string,
  src: string,
  temp: string,
  target: TTarget,
  caseOptions: TCacheCaseOptions,
): ITestProcessor {
  const type = caseOptions.type || 'cache';
  const compileTemp = caseOptions.newCache ? path.join(temp, 'src') : temp;
  const updatePlugin =
    type === 'cache' ? new HotUpdatePlugin(src, compileTemp) : undefined;
  const base: ITestProcessor =
    type === 'normal'
      ? createNormalProcessor(name)
      : type === 'config'
        ? createConfigProcessor(name)
        : {
            config: async (context) => {
              context
                .getCompiler()
                .setOptions(
                  await generateOptions(
                    context,
                    compileTemp,
                    target,
                    updatePlugin!,
                  ),
                );
            },
            compiler: async (context) => {
              await compiler(context, name);
            },
            build: async (context) => {
              await build(context, name);
            },
            run: async (env, context) => {
              await run(env, context, name, (context) =>
                findBundle(name, target, context),
              );
            },
            check: async (env, context) => {
              await check(env, context, name);
            },
            after: async (context) => {
              await afterExecute(context, name);
            },
          };
  let buildIndex = 0;
  let testEnv: ITestEnv;

  const checkCache = (
    env: ITestEnv,
    context: ITestContext,
    unchanged = false,
  ) => {
    const stats = context.getCompiler().getStats()!;
    if (!caseOptions.newCache) {
      if (!unchanged && updatePlugin)
        matchStatsSnapshot(env, src, updatePlugin!.getUpdateIndex(), stats);
      return;
    }
    const cacheStats = getCacheStats(stats);
    fs.outputJsonSync(
      context.getTemp(`cache-stats/${buildIndex}.json`)!,
      cacheStats,
      { spaces: 2 },
    );
    if (type === 'cache') {
      env
        .expect(JSON.stringify(cacheStats, null, 2))
        .toMatchFileSnapshotSync(
          path.join(src, '__snapshots__', `cache-stats-${buildIndex}.txt`),
        );
    }
    if (unchanged && !stats.hasErrors()) {
      const expected = context.getTestConfig().cacheHitRate || {};
      for (const [index, compilerStats] of cacheStats.entries()) {
        for (const [label, { hitRate }] of Object.entries(compilerStats)) {
          env
            .expect(hitRate, `compiler ${index}: ${label} hit rate`)
            .toBe(expected[label] ?? 100);
        }
      }
    }
  };

  return {
    before: async (context) => {
      await updatePlugin?.initialize();
      const state: CacheState = {
        compilerIndex: 0,
        nextStart: async (advance = true) => {
          const env = testEnv;
          if (state.compilerIndex >= 100) {
            throw new Error('NEXT_START has been called more than 100 times');
          }
          const manager = context.getCompiler();
          // close() flushes the filesystem cache before a fresh compiler reads it.
          await manager.close();
          const advanceUpdate = !!(
            advance &&
            updatePlugin &&
            (!caseOptions.newCache ||
              updatePlugin.getUpdateIndex() + 1 <
                updatePlugin.getTotalUpdates())
          );
          if (advanceUpdate) await updatePlugin!.goNext();
          state.compilerIndex++;
          buildIndex++;
          manager.createCompiler();
          await manager.build();
          const jsonStats = await checkRestart(env, context, advanceUpdate);
          checkCache(env, context, !advanceUpdate);

          // Every restart needs fresh runners, including every child of a MultiCompiler.
          context.setValue('modules', []);
          context.setValue('runned', new Set<string>());
          context.setValue('multiFileIndexMap', {});
          env.it(
            `NEXT_START run with compilerIndex==${state.compilerIndex}`,
            async () => {
              await base.run(createRuntimeEnv(env), context);
            },
          );
          return jsonStats;
        },
        nextHmr: async (module, options) => {
          const env = testEnv;
          if (!updatePlugin)
            throw new Error('NEXT_HMR is only supported by cache cases');
          await updatePlugin.goNext();
          buildIndex++;
          await context.getCompiler().build();
          const jsonStats = await checkRestart(env, context, true);
          checkCache(env, context);
          const updatedModules = await module.hot.check(options || true);
          if (!updatedModules) throw new Error('No update available');
          return jsonStats;
        },
      };
      context.setValue('cacheState', state);
      await base.before?.(context);
    },
    config: async (context) => {
      await base.config(context);
      if (caseOptions.newCache) enableNewCache(context, temp);
    },
    compiler: base.compiler,
    build: base.build,
    run: async (env, context) => {
      testEnv = env;
      await base.run(createRuntimeEnv(env), context);
    },
    check: async (env, context) => {
      await base.check(env, context);
      testEnv = env;
      if (!context.getCompiler().getStats()) return;
      checkCache(env, context);
      if (caseOptions.newCache) {
        // Bundle tests run after processor.check(). Queue this after their tests
        // so explicit NEXT_START calls take precedence over the automatic restart.
        env.it('should restore the persistent cache on restart', async () => {
          const state = context.getValue<CacheState>('cacheState')!;
          if (state.compilerIndex === 0) await state.nextStart(false);
        });
      }
    },
    after: base.after,
    afterAll: () => {
      if (!updatePlugin) return;
      const updateIndex = updatePlugin.getUpdateIndex();
      const totalUpdates = updatePlugin.getTotalUpdates();
      if (updateIndex + 1 !== totalUpdates) {
        throw new Error(
          `Should run all hot steps (${updateIndex + 1} / ${totalUpdates}): ${name}`,
        );
      }
    },
  };

  async function checkRestart(
    env: ITestEnv,
    context: ITestContext,
    advance: boolean,
  ) {
    const manager = context.getCompiler();
    const stats = manager.getStats()!;
    if (type !== 'cache' || !advance) {
      await base.check(env, context);
    }
    const jsonStats = stats.toJson({
      assets: true,
      chunks: true,
      chunkModules: true,
      modules: true,
      entrypoints: true,
      chunkGroups: true,
    });
    if (type === 'cache' && advance) {
      const updateIndex = updatePlugin!.getUpdateIndex();
      for (const kind of ['error', 'warning'] as const) {
        await checkArrayExpectation(
          src,
          jsonStats,
          kind,
          `${kind}s${updateIndex}`,
          kind === 'error' ? 'Error' : 'Warning',
          manager.getOptions(),
        );
      }
    }
    return jsonStats;
  }
}

const creators = new Map<string, BasicCaseCreator>();

export function createCacheCase(
  name: string,
  src: string,
  dist: string,
  target: TTarget,
  temp: string,
  options: TCacheCaseOptions = {},
) {
  const key = JSON.stringify([target, options]);
  let creator = creators.get(key);
  if (!creator) {
    creator = new BasicCaseCreator({
      clean: true,
      describe: true,
      target,
      newCache: options.newCache,
      steps: ({ name, src, temp }) => [
        createCacheProcessor(name, src, temp!, target, options),
      ],
      runner: {
        key: (context, name, file) => {
          const index =
            context.getValue<CacheState>('cacheState')!.compilerIndex;
          const key =
            options.type === 'config'
              ? getMultiCompilerRunnerKey(context, name, file)
              : name;
          return `${key}:${index}`;
        },
        runner: (context, name, file, env) =>
          createRunner(context, name, file, env, options),
      },
      concurrent: true,
    });
    creators.set(key, creator);
  }
  creator.create(name, src, dist, temp);
}

function enableNewCache(context: ITestContext, temp: string) {
  const options = context.getCompiler().getOptions();
  for (const [index, config] of (Array.isArray(options)
    ? options
    : [options]
  ).entries()) {
    config.experiments = {
      ...config.experiments,
      newCache: config.experiments?.newCache || true,
    };
    const cache = typeof config.cache === 'object' ? config.cache : {};
    const storage = cache.type === 'persistent' ? cache.storage : undefined;
    config.cache = {
      ...cache,
      type: 'persistent',
      storage:
        storage?.directory !== undefined || storage?.location !== undefined
          ? storage
          : {
              type: 'filesystem',
              directory: path.join(temp, '.cache', String(index)),
            },
    };
    config.plugins ??= [];
    config.plugins.push({
      apply(compiler: Compiler) {
        compiler.hooks.thisCompilation.tap(
          'NewCacheStatsPlugin',
          (compilation) => {
            const builtModules = new Set<string>();
            builtModulesByCompilation.set(compilation, builtModules);
            compilation.hooks.buildModule.tap(
              'NewCacheStatsPlugin',
              (module) => {
                builtModules.add(module.identifier());
              },
            );
          },
        );
      },
    });
  }
}

function getCacheStats(stats: Stats | MultiStats): CacheStats[] {
  return ('stats' in stats ? stats.stats : [stats]).map((stats) => {
    const newCache = stats.compilation.options.experiments.newCache;
    const enabledCaches: Record<string, boolean | undefined> = {
      'module code generation cache': newCache && newCache.codeGeneration,
      'minimize persistent cache': newCache && newCache.minimize,
      'source map persistent cache': newCache && newCache.devtool,
    };
    const json = stats.toJson({
      all: false,
      modules: true,
      cachedModules: true,
      nestedModules: true,
      orphanModules: true,
      runtimeModules: false,
      groupModulesByAttributes: false,
      groupModulesByCacheStatus: false,
      groupModulesByExtension: false,
      groupModulesByLayer: false,
      groupModulesByPath: false,
      groupModulesByType: false,
      modulesSpace: Infinity,
      nestedModulesSpace: Infinity,
      logging: 'verbose',
      loggingDebug: true,
    });
    const result: CacheStats = {};
    const add = (label: string, hits: number, total: number) => {
      if (label in enabledCaches && !enabledCaches[label]) return;
      if (total)
        result[label] = {
          hits,
          total,
          hitRate: Number(((hits / total) * 100).toFixed(1)),
        };
    };
    if (newCache && newCache.module) {
      // Stats.built includes modules restored while creating a new module graph.
      // Compare cacheable modules against actual buildModule calls instead.
      const built = builtModulesByCompilation.get(stats.compilation)!;
      const seen = new Set<string>();
      let hits = 0;
      const visitModules = (modules: StatsModule[]) => {
        for (const module of modules) {
          if (module.modules) {
            visitModules(module.modules);
          } else if (
            module.cacheable &&
            module.nameForCondition &&
            module.identifier &&
            !seen.has(module.identifier)
          ) {
            seen.add(module.identifier);
            if (!built.has(module.identifier)) hits++;
          }
        }
      };
      visitModules(json.modules || []);
      add('module build cache', hits, seen.size);
    }
    const visitLogs = (
      entries: NonNullable<StatsCompilation['logging']>[string]['entries'],
    ) => {
      for (const entry of entries) {
        if (entry.type === 'cache') {
          const match = entry.message.match(/^(.*): [\d.]+% \((\d+)\/(\d+)\)$/);
          if (!match)
            throw new Error(`Invalid cache statistics: ${entry.message}`);
          add(match[1], Number(match[2]), Number(match[3]));
        }
        if (entry.children) visitLogs(entry.children);
      }
    };
    for (const logging of Object.values(json.logging || {}))
      visitLogs(logging.entries);
    return Object.fromEntries(
      Object.entries(result).sort(([a], [b]) => a.localeCompare(b)),
    );
  });
}

function matchStatsSnapshot(
  env: ITestEnv,
  source: string,
  updateIndex: number,
  stats: Stats | MultiStats,
) {
  const content = normalizePlaceholder(
    stats
      .toString({
        all: false,
        logging: false,
        loggingDebug: /^rspack\.persistentCache$/,
        colors: false,
      })
      .replace(/[0-9]+(\.[0-9]+)? ms/g, 'xx ms'),
  ).trim();

  if (!content.includes('rspack.persistentCache')) {
    return;
  }

  env
    .expect(content)
    .toMatchFileSnapshotSync(
      path.resolve(source, '__snapshots__', `stats-${updateIndex}.txt`),
    );
}

async function generateOptions(
  context: ITestContext,
  temp: string,
  target: TTarget,
  updatePlugin: HotUpdatePlugin,
): Promise<RspackOptions> {
  let options = {
    context: temp,
    cache: true,
    devtool: false,
    output: {
      path: context.getDist(),
      filename: 'bundle.js',
      chunkFilename: '[name].chunk.[fullhash].js',
      publicPath: 'https://test.cases/path/',
      library: { type: 'commonjs2' },
      bundlerInfo: {
        force: false,
      },
    },
    optimization: {
      moduleIds: 'named',
      emitOnErrors: true,
    },
    target,
  } as RspackOptions;

  options.plugins ??= [];
  options.plugins.push(new rspack.HotModuleReplacementPlugin());

  options = await config(
    context,
    'cacheCase',
    ['rspack.config.js', 'webpack.config.js'].map((i) => path.resolve(temp, i)),
    options,
  );

  // overwrite
  if (!options.entry) {
    options.entry = './index.js';
  }

  // rewrite context to temp dir
  options.context = temp;
  options.module ??= {};
  for (const cssModuleType of ['css/auto', 'css/module', 'css'] as const) {
    options.module!.generator ??= {};
    options.module!.generator[cssModuleType] ??= {};
    options.module!.generator[cssModuleType]!.exportsOnly ??=
      target === 'async-node';
  }
  options.plugins ??= [];
  (options as RspackOptions).plugins!.push(updatePlugin);
  if (!global.printLogger) {
    options.infrastructureLogging = {
      level: 'error',
    };
  }

  return options;
}

function findBundle(
  name: string,
  target: TTarget,
  context: ITestContext,
): string[] {
  const files: string[] = [];
  const prefiles: string[] = [];
  const compiler = context.getCompiler();
  if (!compiler) throw new Error('Compiler should exists when find bundle');
  const stats = compiler.getStats();
  if (!stats) throw new Error('Stats should exists when find bundle');
  const info = stats.toJson({
    all: false,
    entrypoints: true,
  }) as StatsCompilation;
  if (target === 'web' || target === 'webworker') {
    for (const file of info.entrypoints!.main.assets!) {
      if (isJavaScript(file.name)) {
        files.push(file.name);
      } else {
        prefiles.push(file.name);
      }
    }
  } else {
    const assets = info.entrypoints!.main.assets!.filter((s) =>
      isJavaScript(s.name),
    );
    files.push(assets[assets.length - 1].name);
  }
  return [...prefiles, ...files];
}

function createRunner(
  context: ITestContext,
  name: string,
  file: string,
  env: ITestEnv,
  options: TCacheCaseOptions,
) {
  const state = context.getValue<CacheState>('cacheState')!;
  const testConfig = context.getTestConfig();
  // Capture the generation for this runner; an old bundle may still finish
  // its assertions after awaiting NEXT_START.
  const compilerIndex = state.compilerIndex;
  const scopedConfig: typeof testConfig = {
    ...testConfig,
    moduleScope(ms, stats, options) {
      const scope = testConfig.moduleScope?.(ms, stats, options) || ms;
      scope.COMPILER_INDEX = compilerIndex;
      scope.NEXT_HMR = state.nextHmr;
      scope.NEXT_START = () => state.nextStart();
      return scope;
    },
  };
  if (options.type === 'config') {
    return createMultiCompilerRunner(context, name, file, env, {
      testConfig: scopedConfig,
      cachable: false,
    });
  }
  if (options.type === 'normal') {
    return createCaseRunner(context, name, file, env, {
      testConfig: scopedConfig,
      cachable: false,
    });
  }
  return new NodeRunner({
    env,
    stats: cachedStats(context, name),
    cachable: false,
    name,
    runInNewContext: false,
    testConfig: scopedConfig,
    source: context.getSource(),
    dist: context.getDist(),
    compilerOptions: context.getCompiler().getOptions(),
  });
}

// Keep bundle hooks local to one compiler generation. Orchestration tests must
// not execute a case's beforeEach/afterEach, or accumulate hooks after a restart.
function createRuntimeEnv(env: ITestEnv): ITestEnv {
  type TestFn = (done?: (error?: Error) => void) => unknown;
  const before: TestFn[] = [];
  const after: TestFn[] = [];
  const execute = async (fn: TestFn) => {
    if (fn.length) {
      await new Promise<void>((resolve, reject) => {
        fn((error) => (error ? reject(error) : resolve()));
      });
    } else {
      await fn();
    }
  };
  return {
    ...env,
    beforeEach: (fn: TestFn) => {
      before.push(fn);
    },
    afterEach: (fn: TestFn) => {
      after.push(fn);
    },
    it: (name: string, fn: TestFn) => {
      env.it(name, async () => {
        for (const hook of before) await execute(hook);
        try {
          await execute(fn);
        } finally {
          for (const hook of after) await execute(hook);
        }
      });
    },
  };
}
