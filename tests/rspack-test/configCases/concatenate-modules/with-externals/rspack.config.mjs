/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  entry: {
    main: './index.js',
  },
  externals: {
    jquery: 'var { version: 1 }',
  },
  externalsPresets: {
    node: true,
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
};
