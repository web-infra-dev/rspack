/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  module: {
    rules: [],
  },

  optimization: {
    sideEffects: true,
  },
  externalsPresets: {
    node: true,
  },
  externals: {
    'react-router-dom': 'Buffer',
  },
};
