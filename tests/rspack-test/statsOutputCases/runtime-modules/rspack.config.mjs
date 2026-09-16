/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index',
  mode: 'development',
  stats: {
    all: false,
    modules: true,
    runtimeModules: true,
  },
};
