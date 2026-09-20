import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './main.js',
  },
  experiments: {
    sourceImport: true,
    outputModule: true,
  },
  output: {
    module: true,
    filename: '[name].mjs',
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
};
