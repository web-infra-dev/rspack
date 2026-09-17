import { rspack } from '@rspack/core';
export default {
  mode: 'production',
  target: 'web',
  entry: {
    main: './index.js',
  },
  output: {
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    new rspack.HtmlRspackPlugin(),
    new rspack.SubresourceIntegrityPlugin(),
  ],
};
