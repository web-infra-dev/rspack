import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

export default defineConfig({
  target: 'node',
  mode: 'development',
  devtool: false,
  entry: {
    main: './index.js',
    success: ['./sync.js', './first.js', './last.js'],
    failure: ['./reject-first.js', './reject-last.js'],
  },
  output: {
    module: true,
    filename: '[name].mjs',
  },
  optimization: { runtimeChunk: 'single' },
  externalsType: 'import',
  externals: { failure: './failure.mjs' },
  plugins: [
    new container.ModuleFederationPlugin({
      name: 'async_entry',
      experiments: { asyncStartup: true },
    }),
  ],
});
