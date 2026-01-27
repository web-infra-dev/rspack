import path from 'node:path';
import type { Compiler, RspackOptions, Stats } from '@rspack/core';
import fs from 'fs-extra';
import { normalizePlaceholder } from '../helper/expect/placeholder';
import captureStdio from '../helper/legacy/captureStdio';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv } from '../type';
import { build, compiler, configMultiCompiler } from './common';

const REG_ERROR_CASE = /error$/;

export function createStatsProcessor(
  name: string,
  defaultOptions: (index: number, context: ITestContext) => RspackOptions,
  overrideOptions: (
    index: number,
    context: ITestContext,
    options: RspackOptions,
  ) => void,
) {
  const writeStatsOuptut = false;
  const snapshotName = 'stats.txt';
  let stderr: any = null;
  return {
    before: async (context: ITestContext) => {
      stderr = captureStdio(process.stderr, true);
    },
    config: async (context: ITestContext) => {
      configMultiCompiler(
        context,
        name,
        ['rspack.config.js', 'webpack.config.js'],
        defaultOptions,
        overrideOptions,
      );
    },
    compiler: async (context: ITestContext) => {
      const c = await compiler(context, name);
      await statsCompiler(context, c);
    },
    build: async (context: ITestContext) => {
      await build(context, name);
    },
    run: async (env: ITestEnv, context: ITestContext) => {
      // no need to run, just check snapshot
    },
    check: async (env: ITestEnv, context: ITestContext) => {
      await check(env, context, name, writeStatsOuptut, snapshotName, stderr);
    },
    after: async (context: ITestContext) => {
      stderr.restore();
    },
  };
}

const creator = new BasicCaseCreator({
  clean: true,
  describe: false,
  steps: ({ name }) => [
    createStatsProcessor(name, defaultOptions, overrideOptions),
  ],
  description: () => 'should print correct stats for',
});

export function createStatsOutputCase(name: string, src: string, dist: string) {
  creator.create(name, src, dist);
}

function defaultOptions(index: number, context: ITestContext): RspackOptions {
  if (
    fs.existsSync(path.join(context.getSource(), 'rspack.config.js')) ||
    fs.existsSync(path.join(context.getSource(), 'webpack.config.js'))
  ) {
    return {
      experiments: {
        css: true,
        rspackFuture: {
          bundlerInfo: {
            force: false,
          },
        },
      },
    } as RspackOptions;
  }
  return {
    context: context.getSource(),
    mode: 'development',
    entry: './index.js',
    output: {
      filename: 'bundle.js',
      path: context.getDist(),
    },
    optimization: {
      minimize: false,
    },
    experiments: {
      css: true,
      rspackFuture: {
        bundlerInfo: {
          force: false,
        },
      },
    },
  } as RspackOptions;
}

function overrideOptions(
  index: number,
  context: ITestContext,
  options: RspackOptions,
) {
  if (!options.context) options.context = context.getSource();
  if (!options.output) options.output = options.output || {};
  if (!options.output.path) options.output.path = context.getDist();
  if (!options.plugins) options.plugins = [];
  if (!options.optimization) options.optimization = {};
  if (options.optimization.minimize === undefined) {
    options.optimization.minimize = false;
  }
  if (!global.printLogger) {
    options.infrastructureLogging = {
      level: 'error',
    };
  }
}

class RspackStats {
  constructor(public value: string) {}
}

async function check(
  env: ITestEnv,
  context: ITestContext,
  name: string,
  writeStatsOuptut: boolean,
  snapshot: string,
  stderr: any,
) {
  const compiler = context.getCompiler();
  const options = compiler.getOptions() as RspackOptions;
  const stats = compiler.getStats();
  if (!stats || !compiler) return;

  for (const compilation of []
    .concat((stats as any).stats || stats)
    .map((s: any) => s.compilation)) {
    compilation.logging.delete('webpack.Compilation.ModuleProfile');
  }

  if (REG_ERROR_CASE.test(name)) {
    env.expect(stats.hasErrors()).toBe(true);
  } else if (stats.hasErrors()) {
    throw new Error(
      stats.toString({
        all: false,
        errors: true,
        // errorStack: true,
        // errorDetails: true
      }),
    );
  } else if (writeStatsOuptut) {
    fs.writeFileSync(
      path.join(context.getDist(), 'stats.txt'),
      stats.toString({
        preset: 'verbose',
        // context: context.getSource(),
        colors: false,
      }),
      'utf-8',
    );
  }
  let toStringOptions: any = {
    context: context.getSource(),
    colors: false,
  };
  let hasColorSetting = false;
  if (typeof options.stats !== 'undefined') {
    toStringOptions = options.stats;
    if (toStringOptions === null || typeof toStringOptions !== 'object')
      toStringOptions = { preset: toStringOptions };
    if (!toStringOptions.context) toStringOptions.context = context.getSource();
    hasColorSetting = typeof toStringOptions.colors !== 'undefined';
  }

  if (Array.isArray(options) && !toStringOptions.children) {
    toStringOptions.children = options.map((o) => o.stats);
  }

  // mock timestamps
  for (const { compilation: s } of [].concat(
    (stats as any).stats || stats,
  ) as Stats[]) {
    env.expect(s.startTime).toBeGreaterThan(0);
    env.expect(s.endTime).toBeGreaterThan(0);
    s.endTime = new Date('04/20/1970, 12:42:42 PM').getTime();
    s.startTime = s.endTime - 1234;
  }

  let actual = stats.toString(toStringOptions);
  env.expect(typeof actual).toBe('string');
  actual = stderr.toString() + actual;
  if (!hasColorSetting) {
    actual = actual
      .replace(/\u001b\[[0-9;]*m/g, '')
      // CHANGE: The time unit display in Rspack is second
      .replace(/[.0-9]+(\s?s)/g, 'X$1')
      // CHANGE: Replace bundle size, since bundle sizes may differ between platforms
      .replace(/[0-9]+(\.[0-9]+)? KiB/g, 'xx KiB')
      .replace(/[0-9]+(\.[0-9]+)? bytes/g, 'xx bytes')
      .replace(/[0-9]+(\.[0-9]+)? ms/g, 'xx ms');
  }

  actual = actual
    .split('\n')
    .filter((line) => !line.includes('@rstest/core/dist'))
    .join('\n');

  const snapshotPath = path.isAbsolute(snapshot)
    ? snapshot
    : path.resolve(context.getSource(), `./__snapshots__/${snapshot}`);

  env.expect(new RspackStats(actual)).toMatchFileSnapshotSync(snapshotPath);

  const testConfig = context.getTestConfig();
  if (typeof testConfig?.validate === 'function') {
    testConfig.validate(stats, stderr.toString());
  }
}

async function statsCompiler(context: ITestContext, compiler: Compiler) {
  const compilers: Compiler[] = (compiler as any).compilers
    ? (compiler as any).compilers
    : [compiler as any];
  for (const compiler of compilers) {
    if (!compiler.inputFileSystem) {
      continue;
    }
    const ifs = compiler.inputFileSystem;
    const inputFileSystem = Object.create(ifs);
    compiler.inputFileSystem = inputFileSystem;
    inputFileSystem.readFile = (...args: any[]) => {
      const callback = args.pop();
      ifs.readFile.apply(
        ifs,
        args.concat([
          (err: Error, result: Buffer) => {
            if (err) return callback(err);
            if (!/\.(js|json|txt)$/.test(args[0]))
              return callback(null, result);
            callback(null, normalizePlaceholder(result.toString('utf-8')));
          },
        ]) as Parameters<typeof ifs.readFile>,
      );
    };
  }
}
