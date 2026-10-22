import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new rspack.DefinePlugin({
      PROPERTY: JSON.stringify('foo'),
    }),
  ],
});
