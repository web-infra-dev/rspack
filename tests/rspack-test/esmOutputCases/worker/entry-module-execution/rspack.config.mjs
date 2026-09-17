import { rspack } from '@rspack/core';
export default {
  mode: 'development',
  optimization: {
    runtimeChunk: false,
  },
  plugins: [new rspack.experiments.RslibPlugin()],
};
