import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  externals: {
    fs: 'commonjs fs',
  },
  plugins: [new RslibPlugin()],
});
