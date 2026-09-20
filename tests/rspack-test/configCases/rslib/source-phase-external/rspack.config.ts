import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  entry: {
    main: './main.js',
  },
  experiments: {
    sourceImport: true,
    outputModule: true,
  },
  output: {
    module: true,
    filename: '[name].js',
    library: {
      type: 'modern-module',
    },
    iife: false,
  },
  externals: {
    './add.wasm': 'module ./add.wasm',
  },
  plugins: [new RslibPlugin()],
  optimization: {
    minimize: false,
  },
});
