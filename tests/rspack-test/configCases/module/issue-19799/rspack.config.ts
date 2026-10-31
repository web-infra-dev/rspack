import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import path from 'node:path';
import { rspack } from '@rspack/core';

export default defineConfig((_env, { testPath }) => [
  {
    output: {
      filename: 'lib.mjs',
      module: true,
      library: {
        type: 'module',
      },
    },
    plugins: [
      {
        apply(compiler: Compiler) {
          compiler.hooks.compilation.tap('MyPlugin', (compilation) => {
            const hooks =
              rspack.javascript.JavascriptModulesPlugin.getCompilationHooks(
                compilation,
              );
            // @ts-expect-error Rspack does not expose inlineInRuntimeBailout yet.
            hooks.inlineInRuntimeBailout.tap('test', () => 'test bailout');
          });
        },
      },
    ],
  },
  {
    name: 'test-output',
    entry: './test.js',
    output: {
      filename: 'test.mjs',
      module: true,
    },
    experiments: {},
    externals: {
      lib: path.resolve(testPath, './lib.mjs'),
    },
    externalsType: 'module-import',
  },
]);
