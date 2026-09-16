/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  stats: {
    hash: false,
    timings: false,
    builtAt: false,
    assets: false,
    chunks: true,
    chunkRelations: true,
    chunkModules: true,
    dependentModules: true,
    modules: false,
  },
};
