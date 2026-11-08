import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    './chunk-with-nonce.web.js': 'commonjs ./chunk-with-nonce.web.js',
  },
  target: 'web',
  output: {
    chunkFilename: 'chunk-with-nonce.web.js',
    crossOriginLoading: 'anonymous',
    trustedTypes: true,
  },
  optimization: {
    minimize: false,
  },
});
