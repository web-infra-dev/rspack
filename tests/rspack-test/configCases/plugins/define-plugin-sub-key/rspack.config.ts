import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    new DefinePlugin({
      'foo.bar.baz': '"test"',
    }),
  ],
});
