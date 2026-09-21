import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  target: 'node',
  entry: {
    index: {
      import: './main.js',
      filename: 'index.mjs',
    },
    test: {
      import: './index.js',
      filename: 'bundle.mjs',
    },
  },
  experiments: {
    asyncWebAssembly: true,
    outputModule: true,
    sourceImport: true,
  },
  output: {
    module: true,
    chunkFilename: '[name].mjs',
    webassemblyModuleFilename: '[id].[hash].wasm',
    library: {
      type: 'modern-module',
    },
  },
  module: {
    rules: [
      {
        test: /\.wat$/,
        loader: 'wast-loader',
        type: 'webassembly/async',
      },
    ],
  },
  plugins: [new RslibPlugin()],
  optimization: {
    concatenateModules: true,
    minimize: false,
    runtimeChunk: false,
  },
});
