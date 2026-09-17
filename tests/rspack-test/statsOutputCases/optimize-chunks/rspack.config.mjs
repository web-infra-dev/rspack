/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  stats: {
    assets: true,
    ids: true,
    reasons: false,
    modules: false,
    chunks: true,
    chunkRelations: true,
    chunkModules: true,
    dependentModules: true,
    chunkOrigins: true,
  },
};
