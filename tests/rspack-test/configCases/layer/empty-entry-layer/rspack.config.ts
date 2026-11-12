import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  module: {
    rules: [
      {
        issuerLayer: 'dark',
        resolve: {
          conditionNames: ['dark', '...'],
        },
      },
    ],
  },
});
