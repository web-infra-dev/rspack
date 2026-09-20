import { defineConfig } from '@rspack/cli';

import { IgnorePlugin } from '@rspack/core';

export default defineConfig({
  entry: './test.js',
  plugins: [
    new IgnorePlugin({
      checkResource(resource, context) {
        return /ignored-module/.test(resource) && /folder-b/.test(context);
      },
    }),
  ],
});
