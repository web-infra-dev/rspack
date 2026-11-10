import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default [false, true].map((minimize) =>
  defineConfig({
    mode: 'production',
    devtool: 'source-map',
    externals: ['fs', 'source-map'],
    externalsType: 'commonjs',
    module: {
      rules: [
        {
          test: /mapped\.js$/,
          loader: 'builtin:swc-loader',
          options: { sourceMaps: true, jsc: { target: 'es2015' } },
        },
      ],
    },
    optimization: {
      concatenateModules: false,
      minimize,
      minimizer: [
        new rspack.SwcJsMinimizerRspackPlugin({
          minimizerOptions: { compress: false, mangle: false },
        }),
      ],
    },
  }),
);
