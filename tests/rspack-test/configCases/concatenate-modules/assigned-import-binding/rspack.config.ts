import { defineConfig, definePlugin } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

function config(index: number, concatenateModules: boolean): Configuration {
  return {
    mode: 'production',
    devtool: false,
    output: {
      filename: `bundle.${index}.js`,
    },
    plugins: [
      definePlugin(function (compiler) {
        new compiler.rspack.DefinePlugin({
          CONCATENATED: JSON.stringify(concatenateModules),
        }).apply(compiler);
      }),
    ],
    optimization: {
      concatenateModules,
      inlineExports: true,
      minimize: false,
      moduleIds: 'named',
      chunkIds: 'named',
    },
  };
}

export default defineConfig([config(0, true), config(1, false)]);
