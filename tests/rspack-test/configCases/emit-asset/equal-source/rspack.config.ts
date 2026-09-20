import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.txt$/,
        use: {
          loader: 'file-loader',
          options: {
            name: 'same-name.txt',
          },
        },
      },
    ],
  },
});
