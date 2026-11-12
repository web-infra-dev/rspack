import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.tsx',
  module: {
    rules: [
      {
        test: /\.tsx$/,
        use: {
          loader: 'builtin:swc-loader',
        },
      },
    ],
  },
});
