import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  output: {
    module: true,
    chunkFormat: 'module',
  },
});
