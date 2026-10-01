import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  target: 'web',
  output: {
    crossOriginLoading: 'anonymous',
  },
  plugins: [new rspack.SubresourceIntegrityPlugin()],
});
