import path from 'node:path';
import type { RspackOptions } from '@rspack/core';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv, ITestProcessor } from '../type';
import { build, compiler, configMultiCompiler } from './common';

function overrideOptions(
  index: number,
  context: ITestContext,
  options: RspackOptions,
) {
  options.context = context.getSource();
  options.output = options.output || {};
  options.output.pathinfo = true;
  options.output.path = context.getDist();
  options.output.publicPath = 'dist/';
  if (!options.entry) options.entry = './example.js';
  if (!options.plugins) options.plugins = [];
}

function createExampleProcessor(name: string): ITestProcessor {
  return {
    config: async (context: ITestContext) => {
      configMultiCompiler(
        context,
        name,
        ['rspack.config.js', 'webpack.config.js'],
        () => ({}),
        overrideOptions,
      );
    },
    compiler: async (context: ITestContext) => {
      await compiler(context, name);
    },
    build: async (context: ITestContext) => {
      await build(context, name);
    },
    run: async (env: ITestEnv, context: ITestContext) => {
      // no need to run, just check the building
    },
    check: async (env: ITestEnv, context: ITestContext) => {
      const compiler = context.getCompiler();
      const stats = compiler.getStats();
      if (stats?.hasErrors()) {
        console.log(
          stats.toString({
            all: false,
            errors: true,
            errorDetails: true,
            errorStack: true,
          }),
        );
      }
      expect(stats?.hasErrors()).toBe(false);
    },
  };
}

const creator = new BasicCaseCreator({
  clean: true,
  steps: ({ name }) => [createExampleProcessor(name)],
  concurrent: true,
});

export function createExampleCase(name: string, src: string) {
  creator.create(name, src, path.join(src, 'dist'));
}
