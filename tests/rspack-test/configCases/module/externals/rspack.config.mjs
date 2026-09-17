/** @type {import("@rspack/core").Configuration} */
export default {
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
    module: true,
    filename: '[name].mjs',
  },
  externals: './imported.mjs',
  optimization: {
    concatenateModules: true,
  },
};
