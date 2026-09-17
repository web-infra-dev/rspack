import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  devtool: false,
  plugins: [
    new rspack.EvalSourceMapDevToolPlugin({
      ignoreList: [/ignored\.js/],
    }),
  ],
  optimization: {
    // Ensure the correct `sourceMappingURL` is detected
    concatenateModules: true,
    // Avoid the default export is inlined, which causes the module disappears in the source map
    inlineExports: false,
  },
};
