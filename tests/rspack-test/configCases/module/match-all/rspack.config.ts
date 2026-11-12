import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        resourceQuery: /inline/,
        type: 'asset/inline',
      },
    ],
  },
});
