import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        dependency: 'url',
        type: 'asset',
        generator: {
          dataUrl: {
            encoding: false,
          },
        },
      },
    ],
  },
  target: 'web',
});
