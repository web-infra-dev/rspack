/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    concatenateModules: true,
  },
  externals: {
    external: 'this EXTERNAL_TEST_GLOBAL',
  },
};
