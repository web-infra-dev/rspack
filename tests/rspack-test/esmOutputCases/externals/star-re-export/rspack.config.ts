import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  externals: {
    fs: 'module fs',
    path: 'module path',
  },
  plugins: [new RslibPlugin()],
});
