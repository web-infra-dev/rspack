const {
  experiments: { RslibPlugin },
} = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
};
