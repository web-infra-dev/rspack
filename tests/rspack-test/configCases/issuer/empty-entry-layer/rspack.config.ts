import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  module: {
    rules: [
      {
        issuer: /dark/,
        resolve: {
          conditionNames: ['dark', '...'],
        },
      },
    ],
  },
});
