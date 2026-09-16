import { rspack } from '@rspack/core';
export default {
  target: 'web',
  output: {
    crossOriginLoading: 'anonymous',
  },
  plugins: [new rspack.SubresourceIntegrityPlugin()],
};
