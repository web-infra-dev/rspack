import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
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
});
