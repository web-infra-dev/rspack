import { defineConfig } from '@rspack/cli';
import { SubresourceIntegrityPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'production',
  target: 'web',
  entry: {
    main: './index.js',
  },
  output: {
    crossOriginLoading: 'anonymous',
  },
  plugins: [new SubresourceIntegrityPlugin()],
});
