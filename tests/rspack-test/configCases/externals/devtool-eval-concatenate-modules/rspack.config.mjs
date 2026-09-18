/** @type {import("@rspack/core").Configuration} */
export default {
  devtool: 'eval',
  module: {
    parser: {
      javascript: {
        importMeta: false,
      },
    },
  },
  entry: {
    main: './index.js',
    imported: {
      import: './imported.js',
      library: {
        type: 'module',
      },
    },
  },
  target: 'node14',
  output: {
    filename: '[name].mjs',
    library: {
      type: 'module',
    },
  },
  externals: './imported.mjs',
  externalsType: 'module',
  optimization: {
    concatenateModules: true,
  },
};
