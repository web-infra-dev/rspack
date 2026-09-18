/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  cache: true,
  module: {},
  externals: {
    external: 'var 123',
  },
};
