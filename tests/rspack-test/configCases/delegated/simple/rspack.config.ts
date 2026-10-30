import { defineConfig } from '@rspack/cli';
// @ts-expect-error Rspack does not expose DelegatedPlugin yet.
import { DelegatedPlugin } from '@rspack/core';

export default defineConfig({
  plugins: [
    new DelegatedPlugin({
      source: './bundle',
      type: 'require',
      context: import.meta.dirname,
      content: {
        './a.js': {
          id: 0,
        },
        './loader.js!./b.js': {
          id: 1,
        },
        './dir/c.js': {
          id: 2,
        },
      },
    }),
  ],
});
