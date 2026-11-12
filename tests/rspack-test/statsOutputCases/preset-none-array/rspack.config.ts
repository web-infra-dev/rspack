import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    mode: 'production',
    entry: './index',
    output: {
      filename: 'a.js',
    },
    stats: 'none',
  },

  {
    mode: 'production',
    entry: './index',
    output: {
      filename: 'b.js',
    },
    stats: 'none',
  },
]);
