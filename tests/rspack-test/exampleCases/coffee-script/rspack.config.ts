import { defineConfig } from '@rspack/cli';

export default defineConfig({
  // mode: "development" || "production",
  module: {
    rules: [
      {
        test: /\.coffee$/,
        loader: 'coffee-loader',
      },
    ],
  },
  resolve: {
    extensions: ['.web.coffee', '.web.js', '.coffee', '.js'],
  },
});
