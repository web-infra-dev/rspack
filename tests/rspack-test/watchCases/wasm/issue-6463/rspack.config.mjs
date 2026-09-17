import { HotModuleReplacementPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  experiments: {
    asyncWebAssembly: true,
  },
  plugins: [new HotModuleReplacementPlugin()],
};
