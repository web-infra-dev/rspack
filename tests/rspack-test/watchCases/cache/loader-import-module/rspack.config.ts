import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.generate-json\.js$/,
        use: './loader',
        type: 'json',
      },
    ],
  },
});
