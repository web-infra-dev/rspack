import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.a\.js$/,
        use: ['loader1'],
      },
    ],
  },
});
