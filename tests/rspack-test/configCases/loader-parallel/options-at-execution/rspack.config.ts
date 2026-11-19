import { threadId } from 'node:worker_threads';
import { defineConfig } from '@rspack/cli';

const options = { value: 'at registration' };

export default defineConfig({
  context: import.meta.dirname,
  cache: false,
  module: {
    rules: [
      {
        test: /lib\.js$/,
        use: [
          { loader: './loader.mjs', parallel: true, options },
          {
            loader: './loader.mjs',
            parallel: true,
            options: { getCompilerThread: () => threadId },
          },
        ],
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('UpdateLoaderOptions', () => {
          options.value = 'at execution';
        });
        compiler.hooks.finishMake.tap('ReadWorkerBuildInfo', (compilation) => {
          const { NormalModule } = compiler.rspack;
          const module = [...compilation.modules].find(
            (module) =>
              module instanceof NormalModule &&
              module.resource?.endsWith('lib.js'),
          )!;
          expect(module.buildInfo.workerOption).toBe('at execution');
        });
      },
    },
  ],
});
