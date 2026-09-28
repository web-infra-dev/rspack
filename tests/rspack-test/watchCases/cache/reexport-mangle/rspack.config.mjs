/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  cache: true,
  output: {
    pathinfo: true,
  },
  optimization: {
    minimize: false,
    concatenateModules: false,
  },
  experiments: {
    cache: { type: 'memory' }, // rspack uses different config
  },
};
