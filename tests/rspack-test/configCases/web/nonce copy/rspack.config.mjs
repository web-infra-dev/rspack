import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  // plugin that intercepts __webpack_require__
  plugins: [new rspack.HotModuleReplacementPlugin()],
};
