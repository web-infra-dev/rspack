/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  optimization: {
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
  },
};
