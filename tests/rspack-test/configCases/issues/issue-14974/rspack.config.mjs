import { HotModuleReplacementPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  devtool: false,
  optimization: { usedExports: false, sideEffects: false },
  plugins: [new HotModuleReplacementPlugin()],
};
