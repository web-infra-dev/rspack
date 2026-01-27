import path from 'node:path';
import rspack, {
  type RspackOptions,
  type StatsCompilation,
} from '@rspack/core';
import { isJavaScript } from '../helper';
import { HotUpdatePlugin } from '../helper/hot-update';
import checkArrayExpectation from '../helper/legacy/checkArrayExpectation';
import { NodeRunner } from '../runner';
import { BasicCaseCreator } from '../test/creator';
import type {
  IModuleScope,
  ITestContext,
  ITestEnv,
  ITestProcessor,
} from '../type';
import { afterExecute, build, check, compiler, config, run } from './common';
import { cachedStats } from './runner';

type TTarget = RspackOptions['target'];

function createCacheProcessor(
  name: string,
  src: string,
  temp: string,
  target: TTarget,
): ITestProcessor {
  const updatePlugin = new HotUpdatePlugin(src, temp);
  return {
    before: async (context: ITestContext) => {
      await updatePlugin.initialize();
      context.setValue('hotUpdateContext', updatePlugin);
    },
    config: async (context: ITestContext) => {
      const compiler = context.getCompiler();
      const options = await generateOptions(
        context,
        temp,
        target,
        updatePlugin,
      );
      compiler.setOptions(options);
    },
    compiler: async (context: ITestContext) => {
      await compiler(context, name);
    },
    build: async (context: ITestContext) => {
      await build(context, name);
    },
    run: async (env: ITestEnv, context: ITestContext) => {
      await run(env, context, name, (context) =>
        findBundle(name, target, context),
      );
    },
    check: async (env: ITestEnv, context: ITestContext) => {
      await check(env, context, name);
    },
    after: async (context: ITestContext) => {
      await afterExecute(context, name);
    },
    afterAll: async (context: ITestContext) => {
      const updateIndex = updatePlugin.getUpdateIndex();
      const totalUpdates = updatePlugin.getTotalUpdates();
      if (updateIndex + 1 !== totalUpdates) {
        throw new Error(
          `Should run all hot steps (${updateIndex + 1} / ${totalUpdates}): ${name}`,
        );
      }
    },
  } as ITestProcessor;
}

function getCreator(target: TTarget) {
  if (!creators.has(target)) {
    creators.set(
      target,
      new BasicCaseCreator({
        clean: true,
        describe: true,
        target,
        steps: ({ name, src, target, temp }) => [
          createCacheProcessor(name, src, temp!, target as TTarget),
        ],
        runner: {
          key: (context: ITestContext, name: string, file: string) => name,
          runner: createRunner,
        },
        concurrent: true,
      }),
    );
  }
  return creators.get(target)!;
}

export function createCacheCase(
  name: string,
  src: string,
  dist: string,
  target: RspackOptions['target'],
  temp: string,
) {
  const creator = getCreator(target);
  creator.create(name, src, dist, temp);
}

const creators: Map<TTarget, BasicCaseCreator> = new Map();

async function generateOptions(
  context: ITestContext,
  temp: string,
  target: TTarget,
  updatePlugin: HotUpdatePlugin,
): Promise<RspackOptions> {
  let options = {
    context: temp,
    mode: 'production',
    cache: true,
    devtool: false,
    output: {
      path: context.getDist(),
      filename: 'bundle.js',
      chunkFilename: '[name].chunk.[fullhash].js',
      publicPath: 'https://test.cases/path/',
      library: { type: 'commonjs2' },
    },
    optimization: {
      moduleIds: 'named',
      emitOnErrors: true,
    },
    target,
    experiments: {
      css: true,
      rspackFuture: {
        bundlerInfo: {
          force: false,
        },
      },
    },
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
) {
  const compiler = context.getCompiler();
  const options = compiler.getOptions() as RspackOptions;
  let compilerIndex = 0;
  const testConfig = context.getTestConfig();
  const source = context.getSource();
  const dist = context.getDist();
  const updatePlugin = context.getValue<HotUpdatePlugin>('hotUpdateContext')!;
  const getWebRunner = () => {
    return new NodeRunner({
      env,
      stats: cachedStats(context, name),
      cachable: false,
      name: name,
      runInNewContext: false,
      testConfig: {
        ...testConfig,
        moduleScope(
          ms: IModuleScope,
          stats?: StatsCompilation,
          options?: RspackOptions,
        ) {
          const moduleScope =
            typeof testConfig.moduleScope === 'function'
              ? testConfig.moduleScope(ms, stats, options)
              : ms;

          moduleScope.COMPILER_INDEX = compilerIndex;
          moduleScope.NEXT_HMR = nextHmr;
          moduleScope.NEXT_START = nextStart;
          moduleScope.NEXT_MOVE_DIR_START = nextMoveDirStart;
          return moduleScope;
        },
      },
      source,
      dist,
      compilerOptions: options,
    });
  };

  const checkStats = async (stats: StatsCompilation) => {
    const compilerOptions = compiler.getOptions();
    const updateIndex = updatePlugin.getUpdateIndex();
    await checkArrayExpectation(
      source,
      stats,
      'error',
      `errors${updateIndex}`,
      'Error',
      compilerOptions,
    );
    await checkArrayExpectation(
      source,
      stats,
      'warning',
      `warnings${updateIndex}`,
      'Warning',
      compilerOptions,
    );
  };

  const nextHmr = async (m: any, options?: any): Promise<StatsCompilation> => {
    await updatePlugin.goNext();
    const stats = await compiler.build();
    const jsonStats = stats.toJson({
      // errorDetails: true
    });
    await checkStats(jsonStats);

    const updatedModules = await m.hot.check(options || true);
    if (!updatedModules) {
      throw new Error('No update available');
    }

    return jsonStats as StatsCompilation;
  };

  const nextStart = async (): Promise<StatsCompilation> => {
    await compiler.close();

    await updatePlugin.goNext();
    compilerIndex++;

    compiler.createCompiler();
    const stats = await compiler.build();
    const jsonStats = stats.toJson({
      // errorDetails: true
    });
    await checkStats(jsonStats);

    env.it(`NEXT_START run with compilerIndex==${compilerIndex}`, async () => {
      return getWebRunner().run(file);
    });
    return jsonStats;
  };

  const nextMoveDirStart = async (): Promise<StatsCompilation> => {
    await compiler.close();

    const tempDir = await updatePlugin.moveTempDir();
    const options = await generateOptions(
      context,
      tempDir,
      compiler.getOptions().target,
      updatePlugin,
    );
    compiler.setOptions(options);
    await updatePlugin.goNext();
    compilerIndex++;

    compiler.createCompiler();
    const stats = await compiler.build();
    const jsonStats = stats.toJson({
      // errorDetails: true
    });

    await checkStats(jsonStats);

    env.it(
      `NEXT_MOVE_DIR_START run with compilerIndex==${compilerIndex}`,
      async () => {
        return getWebRunner().run(file);
      },
    );
    return jsonStats as StatsCompilation;
  };

  return getWebRunner();
}
