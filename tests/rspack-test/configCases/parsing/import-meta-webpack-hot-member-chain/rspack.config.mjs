import { HotModuleReplacementPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: false,
  target: 'web',
  plugins: [new HotModuleReplacementPlugin()],
};
