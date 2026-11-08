import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  optimization: {
    concatenateModules: true,
  },
  plugins: [
    new rspack.DllReferencePlugin({
      name: "function(id) { return {default: 'ok'}; }",
      scope: 'dll',
      content: {
        './module': {
          id: 1,
          exports: ['default'],
          buildMeta: {
            exportsType: 'namespace',
          },
        },
      },
    }),
  ],
  ignoreWarnings: [/is not friendly for incremental/],
});
