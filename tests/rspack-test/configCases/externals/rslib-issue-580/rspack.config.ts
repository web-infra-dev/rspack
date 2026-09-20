import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig((_env, { testPath: _testPath }) => {
  return {
    externals: [/.*foo.*/],
    externalsType: 'module',
    output: {
      module: true,
      chunkFormat: 'module',
      filename: '[name].mjs',
    },
    optimization: {
      minimize: true,
      concatenateModules: true,
    },
    plugins: [
      new rspack.CopyRspackPlugin({
        patterns: ['./a/**/*', './_a/**/*'],
      }),
    ],
  };
});
