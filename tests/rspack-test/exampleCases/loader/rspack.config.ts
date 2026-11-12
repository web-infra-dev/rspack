import { defineConfig } from '@rspack/cli';

export default defineConfig({
  // mode: "development" || "production",
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        loader: 'css-loader',
      },
    ],
  },
});
