import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.my$/,
        loader: 'regexp-#-loader',
      },
    ],
  },
});
