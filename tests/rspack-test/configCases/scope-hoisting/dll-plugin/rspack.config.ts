import { defineConfig } from '@rspack/cli';

import { DllReferencePlugin } from '@rspack/core';

export default defineConfig({
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new DllReferencePlugin({
      name: "function(id) { return {default: 'ok'}; }",
      scope: 'dll',
      content: {
        './module': {
          id: 1,
          buildMeta: {
            exportsType: 'namespace',
          },
          exports: ['default'],
        },
      },
    }),
  ],
});
