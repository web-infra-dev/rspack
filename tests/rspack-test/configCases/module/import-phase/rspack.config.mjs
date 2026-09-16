/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'async-node',
  optimization: {
    concatenateModules: false,
  },
  experiments: {
    deferImport: true,
    sourceImport: true,
  },
  module: {
    rules: [
      {
        test: /module\.js$/,
        phase: 'defer',
        loader: './phase-loader.js',
        options: {
          phase: 'defer',
        },
      },
      {
        test: /module\.js$/,
        phase: 'source',
        loader: './phase-loader.js',
        options: {
          phase: 'source',
        },
      },
      {
        test: /module\.js$/,
        phase: 'evaluation',
        loader: './phase-loader.js',
        options: {
          phase: 'evaluation',
        },
      },
    ],
  },
};
