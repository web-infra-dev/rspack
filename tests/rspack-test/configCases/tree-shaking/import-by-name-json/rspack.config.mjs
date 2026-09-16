/**@type {import("@rspack/core").Configuration}*/
export default {
  mode: 'production',
  context: import.meta.dirname,
  optimization: {
    moduleIds: 'named',
    minimize: false,
  },
  externalsPresets: {
    node: true,
  },
};
