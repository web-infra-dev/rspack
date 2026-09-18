/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  performance: false,
  stats: {
    assets: true,
    modules: true,
    modulesSpace: Infinity,
    modulesSort: '!name',
  },
};
