import path from 'node:path';
import type {
  Compilation,
  Compiler,
  OutputFileSystem,
  RspackOptions,
  Stats,
  StatsCompilation,
  WatchFileSystem,
} from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv, ITestProcessor } from '../type';

function createMultiCompilerProcessor(
  name: string,
  caseConfig: TMultiCompilerCaseConfig,
) {
  return {
    config: async (context: ITestContext) => {
      const compiler = context.getCompiler();
      const options = Object.assign(
        [
          {
            name: 'a',
            context: path.join(__dirname, 'fixtures'),
            entry: './a.js',
          },
          {
            name: 'b',
            context: path.join(__dirname, 'fixtures'),
            entry: './b.js',
          },
        ],
        caseConfig.options?.(context) || {},
      );
      compiler.setOptions(options);
    },
    compiler: async (context: ITestContext) => {
      const compiler = context.getCompiler();
      if (caseConfig.compilerCallback) {
        compiler.createCompilerWithCallback(caseConfig.compilerCallback);
      } else {
        compiler.createCompiler();
      }
      const c = compiler.getCompiler()!;
      c.outputFileSystem = createFsFromVolume(new Volume()) as OutputFileSystem;
      c.watchFileSystem = {
        watch() {
          // watch should return a watcher instance
          // watcher instance should have close, pause and getInfo methods
          return {
            close: () => {},
            pause: () => {},
            getInfo: () => {
              return {
                changes: new Set(),
                removals: new Set(),
                fileTimeInfoEntries: new Map(),
                directoryTimeInfoEntries: new Map(),
              };
            },
          };
        },
      } as unknown as WatchFileSystem;
      await caseConfig.compiler?.(context, c);
    },
    build: async (context: ITestContext) => {
      const compiler = context.getCompiler();
      if (typeof caseConfig.build === 'function') {
        await caseConfig.build?.(context, compiler.getCompiler()!);
      } else {
        await compiler.build();
      }
    },
    run: async (env: ITestEnv, context: ITestContext) => {},
    check: async (env: ITestEnv, context: ITestContext) => {},
  } as ITestProcessor;
}

const creator = new BasicCaseCreator({
  clean: true,
  describe: false,
  steps: ({ name, caseConfig }) => {
    return [
      createMultiCompilerProcessor(
        name,
        caseConfig as TMultiCompilerCaseConfig,
      ),
    ];
  },
  concurrent: false,
});

export function createMultiCompilerCase(
  name: string,
  src: string,
  dist: string,
  testConfig: string,
) {
  let caseConfigList:
    | TMultiCompilerCaseConfig
    | TMultiCompilerCaseConfig[] = require(testConfig);
  if (!Array.isArray(caseConfigList)) {
    caseConfigList = [caseConfigList];
  }
  for (let i = 0; i < caseConfigList.length; i++) {
    const caseConfig = caseConfigList[i];
    if (caseConfig.skip) {
      it.skip(`${name}[${i}]`, () => {});
      continue;
    }
    creator.create(`${name}[${i}]`, src, dist, undefined, {
      caseConfig,
      description: () => caseConfig.description,
    });
  }
}

export type TMultiCompilerCaseConfig = {
  description: string;
  error?: boolean;
  skip?: boolean;
  options?: (context: ITestContext) => RspackOptions;
  compiler?: (context: ITestContext, compiler: Compiler) => Promise<void>;
  build?: (context: ITestContext, compiler: Compiler) => Promise<void>;
  check?: ({
    context,
    stats,
    files,
    compiler,
    compilation,
  }: {
    context: ITestContext;
    stats?: StatsCompilation;
    files?: Record<string, string>;
    compiler: Compiler;
    compilation?: Compilation;
  }) => Promise<void>;
  compilerCallback?: (error: Error | null, stats: Stats | null) => void;
};
