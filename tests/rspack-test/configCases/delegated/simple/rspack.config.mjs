import { DelegatedPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
