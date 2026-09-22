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
  module: {
    parser: {
      javascript: {
        url: 'new-url-relative',
      },
    },
  },
  externals: {
    './mod.js': 'module ./mod.js',
    './nested/other.js': 'module ./nested/other.js',
  },
  plugins: [new RslibPlugin()],
  optimization: {
    minimize: false,
  },
};
