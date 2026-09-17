import { HotModuleReplacementPlugin } from '@rspack/core';
export default {
  optimization: {
    runtimeChunk: false,
  },
  plugins: [new HotModuleReplacementPlugin()],
};
