/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css',
      },
    ],
    parser: {
      css: {
        import: false,
      },
    },
    generator: {
      css: {
        exportsOnly: false,
      },
    },
  },
};
