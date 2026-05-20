/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
