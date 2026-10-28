import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  entry: {
    main: './main.js',
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
});
