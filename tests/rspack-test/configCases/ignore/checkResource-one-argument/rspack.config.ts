import { defineConfig } from '@rspack/cli';
import { IgnorePlugin } from '@rspack/core';

export default defineConfig({
  entry: './test.js',
  plugins: [
    new IgnorePlugin({
      checkResource(resource) {
        return /ignored-module/.test(resource);
      },
    }),
  ],
});
