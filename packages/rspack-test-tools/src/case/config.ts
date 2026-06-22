import path from 'node:path';
import type { RspackOptions } from '@rspack/core';
import fs from 'fs-extra';
import { parseResource } from '../helper/legacy/parseResource';
import {
  BasicCaseCreator,
  type IBasicCaseCreatorOptions,
} from '../test/creator';
import type {
  ITestContext,
  ITestEnv,
  ITestProcessor,
  TTestConfig,
} from '../type';
import {
  afterExecute,
  build,
  check,
  compiler,
  configMultiCompiler,
  findMultiCompilerBundle,
  run,
} from './common';
import { applyRuntimeModeTestDefines } from './runtime-mode';
import { createMultiCompilerRunner, getMultiCompilerRunnerKey } from './runner';

export type TConfigCaseConfig = TTestConfig;
type TConfigCaseOptions = Partial<IBasicCaseCreatorOptions> & {
  rspackOptions?: RspackOptions;
};

export function createConfigProcessor(
  name: string,
  rspackOptions?: RspackOptions,
): ITestProcessor {
  return {
    config: (context: ITestContext) => {
      // Isolated cases compile from a private copy of the case dir, so the
      // config (and any fixtures it writes via __dirname) live under <dist>/src.
      if (context.getTestConfig().isolateSource) {
        fs.copySync(context.getSource(), context.getCompileSource());
      }
      configMultiCompiler(
        context,
        name,
        ['rspack.config.cjs', 'rspack.config.js', 'webpack.config.js'],
        defaultOptions,
        (index, context, options) => {
          overrideOptions(index, context, options);
          mergeRspackOptions(options, rspackOptions);
          applyRuntimeModeTestDefines(options);
        },
      );
    },
    compiler: async (context: ITestContext) => {
      await compiler(context, name);
    },
    build: async (context: ITestContext) => {
      await build(context, name);
    },
    run: async (env: ITestEnv, context: ITestContext) => {
      await run(env, context, name, (context: ITestContext) =>
        findMultiCompilerBundle(context, name, findBundle),
      );
    },
    check: async (env: ITestEnv, context: ITestContext) => {
      await check(env, context, name);
    },
    after: async (context: ITestContext) => {
      await afterExecute(context, name);
    },
  };
}

const creator = new BasicCaseCreator({
  clean: true,
  describe: false,
  testConfig: (testConfig) => {
    const oldModuleScope = testConfig.moduleScope;
    testConfig.moduleScope = (ms, stats, compilerOptions) => {
      let res = ms;
      // TODO: modify runner module scope based on stats here
      if (typeof oldModuleScope === 'function') {
        res = oldModuleScope(ms, stats, compilerOptions);
      }
      return res;
    };
  },
  steps: ({ name }) => [createConfigProcessor(name)],
  runner: {
    key: getMultiCompilerRunnerKey,
    runner: createMultiCompilerRunner,
  },
  concurrent: true,
});

export function createConfigCase(
  name: string,
  src: string,
  dist: string,
  rspackOptions?: RspackOptions,
) {
  creator.create(name, src, dist, undefined, {
    rspackOptions,
    steps: ({ name, rspackOptions }) => [
      createConfigProcessor(name, rspackOptions as RspackOptions | undefined),
    ],
  } satisfies TConfigCaseOptions);
}

export function defaultOptions(
  index: number,
  context: ITestContext,
): RspackOptions {
  return {
    context: context.getCompileSource(),
    mode: 'production',
    target: 'async-node',
    devtool: false,
    cache: false,
    output: {
      path: context.getCompileDist(),
      bundlerInfo: {
        force: false,
      },
    },
    optimization: {
      minimize: false,
    },
  };
}

export function enableEsmLibraryPlugin(options: RspackOptions): boolean {
  return (
    options.output?.library === 'modern-module' ||
    (typeof options.output?.library === 'object' &&
      (options.output?.library as { type: string }).type === 'modern-module')
  );
}

export function overrideOptions(
  index: number,
  context: ITestContext,
  options: RspackOptions,
) {
  if (!options.entry) {
    options.entry = './index.js';
  }
  if (options.amd === undefined) {
    options.amd = {};
  }
  if (!options.output?.filename) {
    const runtimeChunkForModernModule =
      options.optimization?.runtimeChunk === undefined &&
      enableEsmLibraryPlugin(options);
    const outputModule =
      options.output?.module || enableEsmLibraryPlugin(options);
    options.output ??= {};
    options.output.filename = `${runtimeChunkForModernModule ? `[name]${outputModule ? '.mjs' : '.js'}` : `bundle${index}${outputModule ? '.mjs' : '.js'}`}`;
  }
  if (enableEsmLibraryPlugin(options)) {
    options.optimization ??= {};
    options.optimization.runtimeChunk ??= { name: `runtime~${index}` };
  }

  if (options.cache === undefined) options.cache = false;
  if (!global.printLogger) {
    options.infrastructureLogging = {
      level: 'error',
    };
  }
}

function mergeRspackOptions(options: RspackOptions, override?: RspackOptions) {
  if (!override) return;

  const { experiments, ...rest } = override;
  Object.assign(options, rest);
  if (experiments) {
    options.experiments = {
      ...options.experiments,
      ...experiments,
    };
  }
}

export function findBundle(
  index: number,
  context: ITestContext,
  options: RspackOptions,
) {
  const testConfig = context.getTestConfig();

  if (typeof testConfig.findBundle === 'function') {
    return testConfig.findBundle!(index, options);
  }

  const ext = path.extname(parseResource(options.output?.filename).path);
  const bundlePath = [];
  if (
    options.output?.path &&
    fs.existsSync(path.join(options.output.path!, `bundle${index}${ext}`))
  ) {
    const cssOutputPath = path.join(
      options.output.path!,
      (typeof options.output?.cssFilename === 'string' &&
        options.output?.cssFilename) ||
        `bundle${index}.css`,
    );
    if (fs.existsSync(cssOutputPath)) {
      bundlePath.push(path.relative(options.output.path!, cssOutputPath));
    }

    bundlePath.push(`./bundle${index}${ext}`);
  }
  return bundlePath;
}
