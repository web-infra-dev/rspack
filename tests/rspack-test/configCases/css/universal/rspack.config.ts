import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    module: true,
  },
  target: ['web', 'node'],
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
