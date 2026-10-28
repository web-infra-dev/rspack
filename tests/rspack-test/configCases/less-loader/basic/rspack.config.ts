import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.less$/,
        use: [{ loader: 'less-loader' }],
        type: 'css',
      },
    ],
  },
});
