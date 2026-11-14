import { defineConfig } from '@rspack/cli';

export default [false, true].map((esm, index) =>
  defineConfig({
    mode: 'development',
    target: 'node',
    entry: { main: './index.js' },
    devtool: false,
    experiments: { runtimeMode: 'rspack' },
    output: {
      filename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      chunkFilename: `[name]${index}.${esm ? 'mjs' : 'js'}`,
      publicPath: '/initial/',
      ...(esm ? { module: true, library: { type: 'modern-module' } } : {}),
    },
    optimization: {
      minimize: false,
      runtimeChunk: 'single',
    },
  }),
);
