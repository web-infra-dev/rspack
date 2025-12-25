import type { RspackOptions, StatsCompilation } from '@rspack/core';
import { NodeRunner, WebRunner } from '../runner';
import { DEBUG_SCOPES } from '../test/debug';
import type { ITestContext, ITestEnv, ITestRunner } from '../type';

export type THotStepRuntimeLangData = {
  outdatedModules: string[];
  outdatedDependencies: Record<string, string[]>;

  updatedModules: string[];
  updatedRuntime: string[];

  acceptedModules: string[];
  disposedModules: string[];
};

export type THotStepRuntimeData = {
  javascript: THotStepRuntimeLangData;
  css: THotStepRuntimeLangData;
  statusPath: string[];
};

export function cachedStats(
  context: ITestContext,
  name: string,
): () => StatsCompilation {
  const compiler = context.getCompiler();
  const statsGetter = (() => {
    let cached: StatsCompilation | null = null;
    return () => {
      if (cached) {
        return cached;
      }
      cached = compiler.getStats()!.toJson({
        errorDetails: true,
      });
      return cached;
    };
  })();
  return statsGetter;
}

export function createRunner(
  context: ITestContext,
  name: string,
  file: string,
  env: ITestEnv,
): ITestRunner {
  const compiler = context.getCompiler();
  const testConfig = context.getTestConfig();
  const compilerOptions = compiler.getOptions() as RspackOptions;
  const runnerOptions = {
    runInNewContext: false,
    cachable: true,
    env,
    stats: cachedStats(context, name),
    name,
    testConfig: context.getTestConfig(),
    source: context.getSource(),
    dist: context.getDist(),
    compilerOptions,
  };
  if (
    compilerOptions.target === 'web' ||
    compilerOptions.target === 'webworker'
  ) {
    return new WebRunner({
      ...runnerOptions,
      runInNewContext: true,
      location: testConfig.location || 'https://test.cases/path/index.html',
    });
  }
  return new NodeRunner(runnerOptions);
}

function getFileIndexHandler(
  context: ITestContext,
  name: string,
  file: string,
) {
  const multiFileIndexMap: Record<string, number[]> =
    context.getValue('multiFileIndexMap') || {};
  const runned = (context.getValue('runned') as Set<string>) || new Set();
  if (typeof multiFileIndexMap[file] === 'undefined') {
    throw new Error('Unexpect file in multiple runner');
  }
  const indexList = multiFileIndexMap[file];
  const seq = indexList.findIndex(
    (index, n) => !runned.has(`${name}:${file}[${n}]`),
  );
  if (seq === -1) {
    throw new Error(`File ${file} should run only ${indexList.length} times`);
  }
  const getIndex = () => [indexList[seq], seq];
  const flagIndex = () => runned.add(`${name}:${file}[${seq}]`);
  context.setValue('runned', runned);
  return { getIndex, flagIndex };
}

export function getMultiCompilerRunnerKey(
  context: ITestContext,
  name: string,
  file: string,
): string {
  const { getIndex } = getFileIndexHandler(context, name, file);
  const [index, seq] = getIndex();
  return `${name}-${index}[${seq}]`;
}

export function createMultiCompilerRunner(
  context: ITestContext,
  name: string,
  file: string,
  env: ITestEnv,
): ITestRunner {
  const testConfig = context.getTestConfig();
  const { getIndex, flagIndex } = getFileIndexHandler(context, name, file);
  const multiCompilerOptions: RspackOptions[] =
    context.getValue('multiCompilerOptions') || [];
  const [index] = getIndex();
  const compilerOptions = multiCompilerOptions[index];
  const logs = context.getValue(DEBUG_SCOPES.RunLogs) as string[] | undefined;
  const errors = context.getValue(DEBUG_SCOPES.RunErrors) as
    | Error[]
    | undefined;
  let runner;
  const runnerOptions = {
    runInNewContext: false,
    cachable: true,
    env,
    stats: () => {
      const s = cachedStats(context, name)();
      if (s.children?.length && s.children.length > 1) {
        s.__index__ = index;
        return s;
      }
      return s.children![index];
    },
    name,
    testConfig: context.getTestConfig(),
    source: context.getSource(),
    dist: context.getDist(),
    compilerOptions,
    logs,
    errors,
  };
  if (
    compilerOptions.target === 'web' ||
    compilerOptions.target === 'webworker'
  ) {
    runner = new WebRunner({
      ...runnerOptions,
      runInNewContext: true,
      location: testConfig.location || 'https://test.cases/path/index.html',
    });
  } else {
    runner = new NodeRunner(runnerOptions);
  }
  flagIndex();
  return runner;
}
