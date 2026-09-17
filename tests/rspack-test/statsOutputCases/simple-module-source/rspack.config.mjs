/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index',
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  stats: {
    assets: true,
    modules: true,
    builtAt: false,
    timings: false,
    source: true,
    version: false,
  },
};
