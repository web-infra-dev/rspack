const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
