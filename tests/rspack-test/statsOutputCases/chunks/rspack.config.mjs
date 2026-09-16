/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  profile: true,
  stats: {
    assets: true,
    reasons: true,
    chunks: true,
    chunkModules: true,
    dependentModules: true,
    chunkRelations: true,
    chunkOrigins: true,
    modules: false,
    publicPath: true,
  },
};
