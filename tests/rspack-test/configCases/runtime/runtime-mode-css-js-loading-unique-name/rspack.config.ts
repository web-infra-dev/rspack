import { defineConfig } from '@rspack/cli';

export default defineConfig({
  experiments: {
    runtimeMode: 'rspack',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  output: {
    uniqueName: 'runtime-review',
  },
  target: 'web',
});
