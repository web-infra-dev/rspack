import type fs from 'node:fs';
import type { Compiler, RspackOptions, Stats } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv } from '../type';

let addedSerializer = false;

export type TStatsAPICaseConfig = {
  description: string;
  options?: (context: ITestContext) => RspackOptions;
  snapshotName?: string;
  compiler?: (context: ITestContext, compiler: Compiler) => Promise<void>;
  build?: (context: ITestContext, compiler: Compiler) => Promise<void>;
  check?: (stats: Stats, compiler: Compiler) => Promise<void>;
};

const creator = new BasicCaseCreator({
  clean: true,
  describe: true,
  steps: ({ name, caseConfig }) => {
    const config = caseConfig as TStatsAPICaseConfig;
    return [
      {
        config: async (context: ITestContext) => {
          const compiler = context.getCompiler();
          compiler.setOptions(options(context, config.options));
        },
        compiler: async (context: ITestContext) => {
          const compilerManager = context.getCompiler();
          compilerManager.createCompiler();
          compiler(context, compilerManager.getCompiler()!, config.compiler);
        },
        build: async (context: ITestContext) => {
          const compiler = context.getCompiler();
          if (typeof config.build === 'function') {
            await config.build(context, compiler.getCompiler()!);
          } else {
            await compiler.build();
          }
        },
        run: async (env: ITestEnv, context: ITestContext) => {
          // no need to run, just check the snapshot of diagnostics
        },
        check: async (env: ITestEnv, context: ITestContext) => {
          await check(env, context, name, config.check);
        },
      },
    ];
  },
  concurrent: true,
});

export function createStatsAPICase(
  name: string,
  src: string,
  dist: string,
  testConfig: string,
) {
  if (!addedSerializer) {
    addedSerializer = true;
  }
  const caseConfig: TStatsAPICaseConfig = require(testConfig);
  creator.create(name, src, dist, undefined, {
    caseConfig,
    description: () => caseConfig.description,
  });
}

function options(
  context: ITestContext,
  custom?: (context: ITestContext) => RspackOptions,
) {
  const res = (custom?.(context) || {}) as RspackOptions;
  res.experiments ??= {};
  res.experiments!.css ??= true;
  res.experiments!.rspackFuture ??= {};
  res.experiments!.rspackFuture!.bundlerInfo ??= {};
  res.experiments!.rspackFuture!.bundlerInfo!.force ??= false;
  if (!global.printLogger) {
    res.infrastructureLogging = {
      level: 'error',
    };
  }
  return res;
}

async function compiler(
  context: ITestContext,
  compiler: Compiler,
  custom?: (context: ITestContext, compiler: Compiler) => Promise<void>,
) {
  if (custom) {
    await custom(context, compiler);
  }
  if (compiler) {
    compiler.outputFileSystem = createFsFromVolume(
      new Volume(),
    ) as unknown as typeof fs;
  }
}

async function check(
  env: ITestEnv,
  context: ITestContext,
  name: string,
  custom?: (stats: Stats, compiler: Compiler) => Promise<void>,
) {
  const manager = context.getCompiler();
  const stats = manager.getStats()! as Stats;
  env.expect(typeof stats).toBe('object');
  await custom?.(stats, manager.getCompiler()!);
}
