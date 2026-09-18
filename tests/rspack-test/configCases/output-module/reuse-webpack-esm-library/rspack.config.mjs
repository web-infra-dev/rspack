import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: 'eval',
  optimization: {
    concatenateModules: false,
  },
  resolve: {
    alias: {
      react: path.resolve(import.meta.dirname, 'react'),
    },
  },
};
