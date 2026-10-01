import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.less$/,
        use: 'less-loader',
        type: 'css/auto',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
