import { defineConfig } from '@rspack/cli';
import { ProvidePlugin } from '@rspack/core';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    moduleIds: 'named',
    inlineExports: true,
  },
  plugins: [
    new ProvidePlugin({
      providedA: ['./constants.js', 'a'],
      providedDefault: ['./constants.js', 'default'],
    }),
  ],
});
