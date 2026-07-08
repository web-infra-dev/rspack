import path from 'node:path';
import type { RspackOptions, StatsCompilation } from '@rspack/core';
import fs from 'fs-extra';
import merge from 'rspack-merge';
import { ECompilerEvent } from '../compiler';
import { readConfigFile } from '../helper';
import { checkArrayExpectation } from '../helper/legacy/checkArrayExpectation';
import { copyDiff } from '../helper/legacy/copyDiff';
import { WebRunner } from '../runner';
import { BasicCaseCreator } from '../test/creator';
import type {
  IModuleScope,
  ITestContext,
  ITestEnv,
  ITestProcessor,
  ITestRunner,
} from '../type';
import { afterExecute, compiler, findMultiCompilerBundle, run } from './common';

type TWatchContext = {
  currentTriggerFilename: string | null;
  lastHash: string | null;
  step: string;
  tempDir: string;
  nativeWatcher: boolean;
  watchState: Record<string, any>;
};

export function createWatchInitialProcessor(
  name: string,
  tempDir: string,
  step: string,
  watchState: Record<string, any>,
  { incremental = false, nativeWatcher = false } = {},
): ITestProcessor {
  const watchContext: TWatchContext = {
    currentTriggerFilename: null,
    lastHash: null,
    step,
    tempDir,
    nativeWatcher,
    watchState,
  };

  return {
    before: (context: ITestContext) => {
      context.setValue('watchContext', watchContext);
    },
    config: (context: ITestContext) => {
      const testConfig = context.getTestConfig();
      const multiCompilerOptions = [];
      const caseOptions: RspackOptions[] = readConfigFile(
        ['rspack.config.js', 'webpack.config.js'].map((i) =>
          context.getSource(i),
        ),
        context,
        {},
      );

      for (const [index, options] of caseOptions.entries()) {
        const compilerOptions = merge(
          defaultOptions!({
            incremental,
            ignoreNotFriendlyForIncrementalWarnings:
              testConfig.ignoreNotFriendlyForIncrementalWarnings,
          }),
          options,
        );
        overrideOptions(
          index,
          context,
          compilerOptions,
          tempDir,
          nativeWatcher,
        );
        multiCompilerOptions.push(compilerOptions);
      }

      const compilerOptions =
        multiCompilerOptions.length === 1
          ? multiCompilerOptions[0]
          : multiCompilerOptions;
      const compiler = context.getCompiler();
      compiler.setOptions(compilerOptions as any);
      context.setValue('multiCompilerOptions', multiCompilerOptions);
    },
    compiler: async (context: ITestContext) => {
      const c = await compiler(context, name);
      c!.hooks.invalid.tap('WatchTestCasesTest', (filename, mtime) => {
        watchContext.currentTriggerFilename = filename;
      });
    },
    build: async (context: ITestContext) => {
      const compiler = context.getCompiler();
      fs.mkdirSync(watchContext.tempDir, { recursive: true });
      copyDiff(
        path.join(context.getSource(), watchContext.step),
        watchContext.tempDir,
        true,
      );

      const pkgJsonFile = path.join(watchContext.tempDir, 'package.json');
      if (!fs.existsSync(pkgJsonFile)) {
        fs.writeJsonSync(pkgJsonFile, { name, version: '0.0.1' });
        const longTimeAgo = Date.now() - 1000 * 60 * 60 * 24;
        fs.utimesSync(
          pkgJsonFile,
          Date.now() - longTimeAgo,
          Date.now() - longTimeAgo,
        );
      }
      const task = new Promise((resolve, reject) => {
        compiler.getEmitter().once(ECompilerEvent.Build, (e, stats) => {
          if (e) return reject(e);
          resolve(stats);
        });
      });
      compiler.watch();
      await task;
    },
    run: async (env: ITestEnv, context: ITestContext) => {
      await run(env, context, name, (context: ITestContext) =>
        findMultiCompilerBundle(context, name, (index, context, options) =>
          findBundle(index, context, options, step),
        ),
      );
    },
    check: async (env: ITestEnv, context: ITestContext) => {
      const testConfig = context.getTestConfig();
      if (testConfig.noTests) return;

      const errors: Array<{ message: string; stack?: string }> = (
        context.getError() || []
      ).map((e) => ({
        message: e.message,
        stack: e.stack,
      }));
      const warnings: Array<{ message: string; stack?: string }> = [];
      const compiler = context.getCompiler();
      const stats = compiler.getStats();
      const options = compiler.getOptions();
      const checkStats = testConfig.checkStats || (() => true);

      if (stats) {
        if (testConfig.writeStatsOuptut) {
          fs.writeFileSync(
            path.join(context.getDist(), 'stats.txt'),
            stats.toString({
              preset: 'verbose',
              colors: false,
            }),
            'utf-8',
          );
        }

        const getJsonStats = (() => {
          let cached: StatsCompilation | null = null;
          return () => {
            if (!cached) {
              cached = stats.toJson({
                assets: true,
                chunks: true,
                chunkModules: true,
                modules: true,
                entrypoints: true,
                chunkGroups: true,
                errorDetails: true,
              });
            }
            return cached;
          };
        })();
        const getStringStats = (() => {
          let cached: string | null = null;
          return () => {
            if (!cached) {
              cached = stats.toString({
                logging: 'verbose',
              });
            }
            return cached;
          };
        })();
        if (checkStats.length > 1) {
          if (
            !checkStats(watchContext.step, getJsonStats(), getStringStats())
          ) {
            throw new Error('stats check failed');
          }
        } else {
          // @ts-expect-error only one param
          if (!checkStats(watchContext.step)) {
            throw new Error('stats check failed');
          }
        }
        if (testConfig.writeStatsJson) {
          fs.writeFileSync(
            path.join(context.getDist(), 'stats.json'),
            JSON.stringify(getJsonStats(), null, 2),
            'utf-8',
          );
        }
        if (
          fs.existsSync(context.getSource(`${watchContext.step}/errors.js`)) ||
          fs.existsSync(
            context.getSource(`${watchContext.step}/warnings.js`),
          ) ||
          stats.hasErrors() ||
          stats.hasWarnings()
        ) {
          const statsJson = stats.toJson({
            assets: true,
            chunks: true,
            chunkModules: true,
            modules: true,
            entrypoints: true,
            chunkGroups: true,
            errorDetails: true,
          });
          if (statsJson.errors) {
            errors.push(...statsJson.errors);
          }
          if (statsJson.warnings) {
            warnings.push(...statsJson.warnings);
          }
        }
      }
      await checkArrayExpectation(
        path.join(context.getSource(), watchContext.step),
        { errors },
        'error',
        'errors',
        'Error',
        options,
      );

      await checkArrayExpectation(
        path.join(context.getSource(), watchContext.step),
        { warnings },
        'warning',
        'warnings',
        'Warning',
        options,
      );

      // clear error if checked
      if (fs.existsSync(context.getSource('errors.js'))) {
        context.clearError();
      }

      // check hash
      if (testConfig.writeStatsOuptut) {
        fs.renameSync(
          path.join(context.getDist(), 'stats.txt'),
          path.join(context.getDist(), `stats.${watchContext.step}.txt`),
        );
      }
      if (testConfig.writeStatsJson) {
        fs.renameSync(
          path.join(context.getDist(), 'stats.json'),
          path.join(context.getDist(), `stats.${watchContext.step}.json`),
        );
      }
    },
    after: async (context: ITestContext) => {
      await afterExecute(context, name);
    },
  };
}

export function createWatchStepProcessor(
  name: string,
  tempDir: string,
  step: string,
  watchState: Record<string, any>,
  { incremental = false, nativeWatcher = false } = {},
) {
  const processor = createWatchInitialProcessor(
    name,
    tempDir,
    step,
    watchState,
    { incremental },
  );
  processor.compiler = (context: ITestContext) => {
    // do nothing
  };
  processor.build = async (context: ITestContext) => {
    const compiler = context.getCompiler();
    const emitter = compiler.getEmitter();
    const DIAG = !!process.env.WATCH_DIAG;

    // The watcher aggregateTimeout (see `watch()`); the settle window must be
    // larger so a slow/extra rebuild cannot be mistaken for a settled result.
    const settleWindow = Number(process.env.WATCH_SETTLE_MS) || 2000;
    // Native Watcher (notify) needs a moment to be ready to detect the change;
    // 400ms on windows, 100ms elsewhere.
    const readyTimeout =
      nativeWatcher && process.platform === 'win32' ? 400 : 100;

    const buildEvents: number[] = [];
    const copyState = { ts: 0 };

    // Capture the LAST build this step settles on, not merely the first one.
    //
    // Under load the watcher can emit an extra transition rebuild that still
    // carries the previous step's content; taking the first Build would run that
    // stale bundle. We instead keep the latest build and only resolve once no new
    // build has arrived for `settleWindow` (> aggregateTimeout), so the final
    // build always reflects the current sources. The listener is armed before
    // copyDiff so a rebuild the step legitimately relies on that is already
    // pending (e.g. a metadata touch from the previous step) is not missed.
    await new Promise<void>((resolve, reject) => {
      let settleTimer: ReturnType<typeof setTimeout> | undefined;
      let finished = false;
      const finish = (fn: () => void) => {
        if (finished) return;
        finished = true;
        emitter.removeListener(ECompilerEvent.Build, onBuild);
        fn();
      };
      const settle = () => {
        if (settleTimer) clearTimeout(settleTimer);
        settleTimer = setTimeout(() => finish(resolve), settleWindow);
      };
      const onBuild = (e: Error | null) => {
        if (finished) return;
        if (e) return finish(() => reject(e));
        if (DIAG) buildEvents.push(Date.now());
        settle();
      };
      emitter.on(ECompilerEvent.Build, onBuild);

      (async () => {
        await new Promise((r) => setTimeout(r, readyTimeout));
        copyState.ts = Date.now();
        copyDiff(path.join(context.getSource(), step), tempDir, false);
      })();
    });

    if (DIAG) {
      let bundleStep = '?';
      try {
        const bundle = fs.readFileSync(
          path.join(context.getDist(), 'bundle.js'),
          'utf-8',
        );
        const m = bundle.match(/toEqual\("(\d)"\)/);
        if (m) bundleStep = m[1];
      } catch {}
      console.error(
        `[WATCH-DIAG] name=${name} step=${step} builds=${buildEvents.length} ` +
          `buildEventsRelCopy=[${buildEvents.map((t) => t - copyState.ts).join(',')}] ` +
          `bundleStep=${bundleStep} STALE=${bundleStep !== '?' && bundleStep !== step}`,
      );
    }
  };
  return processor;
}

const creator = new BasicCaseCreator({
  clean: true,
  runner: {
    key: getWatchRunnerKey,
    runner: createWatchRunner,
  },
  description: (name, index) => {
    return index === 0
      ? `${name} should compile`
      : `should compile step ${index}`;
  },
  describe: false,
  steps: ({ name, src, temp }) => {
    const watchState = {};
    const runs = fs
      .readdirSync(src)
      .sort()
      .filter((name) => fs.statSync(path.join(src, name)).isDirectory())
      .map((name) => ({ name }));

    return runs.map((run, index) =>
      index === 0
        ? createWatchInitialProcessor(name, temp!, run.name, watchState)
        : createWatchStepProcessor(name, temp!, run.name, watchState),
    );
  },
  concurrent: true,
});

export function createWatchCase(
  name: string,
  src: string,
  dist: string,
  temp: string,
) {
  creator.create(name, src, dist, temp);
}

function overrideOptions(
  index: number,
  context: ITestContext,
  options: RspackOptions,
  tempDir: string,
  nativeWatcher: boolean,
) {
  if (!options.mode) options.mode = 'development';
  if (!options.context) options.context = tempDir;
  if (!options.entry) options.entry = './index.js';
  if (!options.target) options.target = 'async-node';
  if (!options.devtool) options.devtool = false;
  if (!options.output) options.output = {};
  if (!options.output.path) options.output.path = context.getDist();
  if (typeof options.output.pathinfo === 'undefined')
    options.output.pathinfo = false;
  if (!options.output.filename) options.output.filename = 'bundle.js';
  options.optimization ??= {};
  options.experiments ??= {};
  options.module ??= {};
  options.module.defaultRules ??= ['...'];
  options.module.defaultRules.push({
    test: /\.css$/,
    type: 'css/auto',
  });

  if (nativeWatcher) {
    (options as RspackOptions).experiments!.nativeWatcher ??= true;
  }

  (options as RspackOptions).output ??= {};
  (options as RspackOptions).output!.bundlerInfo ??= {};
  (options as RspackOptions).output!.bundlerInfo!.force ??= false;
  // test incremental: "safe" here, we test default incremental in Incremental-*.test.js
  (options as RspackOptions).incremental ??= 'safe';

  if (!global.printLogger) {
    options.infrastructureLogging = {
      level: 'error',
    };
  }
}

function findBundle(
  index: number,
  context: ITestContext,
  options: RspackOptions,
  stepName: string,
) {
  const testConfig = context.getTestConfig();

  if (typeof testConfig.findBundle === 'function') {
    return testConfig.findBundle!(index, options, stepName);
  }
  return './bundle.js';
}

function defaultOptions({
  incremental = false,
  ignoreNotFriendlyForIncrementalWarnings = false,
} = {}): RspackOptions {
  if (incremental) {
    return {
      incremental: 'advance',
      ignoreWarnings: ignoreNotFriendlyForIncrementalWarnings
        ? [/is not friendly for incremental/]
        : undefined,
    };
  }
  return {};
}

export function getWatchRunnerKey(
  context: ITestContext,
  name: string,
  file: string,
): string {
  const watchContext = context.getValue('watchContext') as any;
  const stepName: string | void = watchContext?.step;
  return `${name}-${stepName}`;
}

function cachedWatchStats(
  context: ITestContext,
  name: string,
): () => StatsCompilation {
  const compiler = context.getCompiler();
  const watchContext = context.getValue('watchContext') as any;
  const stepName: string = watchContext?.step;
  const statsGetter = (() => {
    const cached: Record<string, StatsCompilation> = {};
    return () => {
      if (cached[stepName]) {
        return cached[stepName];
      }
      cached[stepName] = compiler.getStats()!.toJson({
        entrypoints: true,
        assets: true,
        chunks: true,
        chunkModules: true,
        modules: true,
        errorDetails: true,
      });
      return cached[stepName];
    };
  })();
  return statsGetter;
}

export function createWatchRunner(
  context: ITestContext,
  name: string,
  file: string,
  env: ITestEnv,
): ITestRunner {
  const compiler = context.getCompiler();
  const compilerOptions = compiler.getOptions() as RspackOptions;
  const watchContext = context.getValue('watchContext') as any;
  const stepName: string | void = watchContext?.step;
  if (!stepName) {
    throw new Error('Can not get watch step name from context');
  }

  const state: Record<string, any> | void = watchContext?.watchState;
  if (!state) {
    throw new Error('Can not get watch state from context');
  }

  const isWeb = Array.isArray(compilerOptions)
    ? compilerOptions.some((option) => {
        return option.target === 'web' || option.target === 'webworker';
      })
    : compilerOptions.target === 'web' ||
      compilerOptions.target === 'webworker';

  const testConfig = context.getTestConfig();
  return new WebRunner({
    env,
    stats: cachedWatchStats(context, name),
    name: name,
    runInNewContext: isWeb,
    cachable: false,
    testConfig: {
      ...(testConfig || {}),
      moduleScope: (
        ms: IModuleScope,
        stats?: StatsCompilation,
        options?: RspackOptions,
      ) => {
        ms.STATE = state;
        ms.WATCH_STEP = stepName;
        if (typeof testConfig.moduleScope === 'function') {
          return testConfig.moduleScope(ms, stats, options);
        }
        return ms;
      },
    },
    source: context.getSource(),
    dist: context.getDist(),
    compilerOptions,
    location: testConfig.location || 'https://test.cases/path/index.html',
  });
}
