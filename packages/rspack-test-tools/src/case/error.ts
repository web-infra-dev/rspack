import type fs from 'node:fs';
import path from 'node:path';
import type { Compiler, RspackOptions, StatsError } from '@rspack/core';
import merge from 'webpack-merge';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv } from '../type';

let addedSerializer = false;

const creator = new BasicCaseCreator({
  clean: true,
  describe: true,
  steps: ({ name, caseConfig }) => {
    const config = caseConfig as TErrorCaseConfig;
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

export function createErrorCase(
  name: string,
  src: string,
  dist: string,
  testConfig: string,
) {
  if (!addedSerializer) {
    addedSerializer = true;
  }
  const caseConfigList = require(testConfig);
  function createCase(caseConfig: TErrorCaseConfig) {
    if (caseConfig.skip) {
      it.skip(name, () => {});
      return;
    }
    creator.create(name, src, dist, undefined, {
      caseConfig,
      description: () => caseConfig.description,
    });
  }
  if (Array.isArray(caseConfigList)) {
    for (const caseConfig of caseConfigList) {
      createCase(caseConfig);
    }
  } else {
    createCase(caseConfigList);
  }
}

function options(
  context: ITestContext,
  custom?: (context: ITestContext, options: RspackOptions) => RspackOptions,
): RspackOptions {
  let options = {
    context: path.resolve(__TEST_FIXTURES_PATH__, 'errors'),
    mode: 'none',
    devtool: false,
    optimization: {
      minimize: false,
      moduleIds: 'named',
      chunkIds: 'named',
    },
    otuput: {
      bundlerInfo: {
        force: false,
      },
    },
    experiments: {
      css: true,
    },
  } as RspackOptions;
  if (typeof custom === 'function') {
    options = merge(options, custom(context, options));
  }
  if (options.mode === 'production') {
    if (options.optimization) options.optimization.minimize = true;
    else options.optimization = { minimize: true };
  }
  return options;
}

async function compiler(
  context: ITestContext,
  compiler: Compiler,
  custom?: (context: ITestContext, compiler: Compiler) => Promise<void>,
) {
  if (compiler) {
    compiler.outputFileSystem = {
      // CHANGE: rspack outputFileSystem `mkdirp` uses option `{ recursive: true }`, webpack's second parameter is alway a callback
      mkdir(
        dir: string,
        maybeOptionOrCallback: unknown,
        maybeCallback: unknown,
      ) {
        if (typeof maybeOptionOrCallback === 'function') {
          maybeOptionOrCallback();
        } else if (typeof maybeCallback === 'function') {
          maybeCallback();
        }
      },
      writeFile(file: string, content: string, callback: () => {}) {
        callback();
      },
      stat(file: string, callback: (e: Error) => {}) {
        callback(new Error('ENOENT'));
      },
      mkdirSync() {},
      writeFileSync() {},
    } as unknown as typeof fs;
  }
  await custom?.(context, compiler);
}

class RspackStatsDiagnostics {
  constructor(
    public errors: StatsError[],
    public warnings: StatsError[],
  ) {}
}

async function check(
  env: ITestEnv,
  context: ITestContext,
  name: string,
  check?: (stats: RspackStatsDiagnostics) => Promise<void>,
) {
  if (context.getError().length > 0) {
    await check?.(
      new RspackStatsDiagnostics(context.getError() as StatsError[], []),
    );
    context.clearError();
    return;
  }

  const compiler = context.getCompiler();
  const stats = compiler.getStats();
  env.expect(typeof stats).toBe('object');
  const statsResult = stats!.toJson({ errorDetails: false });
  env.expect(typeof statsResult).toBe('object');
  const { errors, warnings } = statsResult;
  env.expect(Array.isArray(errors)).toBe(true);
  env.expect(Array.isArray(warnings)).toBe(true);

  await check?.(
    new RspackStatsDiagnostics(
      errors as StatsError[],
      warnings as StatsError[],
    ),
  );
}

export type TErrorCaseConfig = {
  description: string;
  skip?: boolean;
  options?: (context: ITestContext) => RspackOptions;
  compiler?: (context: ITestContext, compiler: Compiler) => Promise<void>;
  build?: (context: ITestContext, compiler: Compiler) => Promise<void>;
  check?: (stats: RspackStatsDiagnostics) => Promise<void>;
};
