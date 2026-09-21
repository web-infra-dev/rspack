import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  optimization: {
    innerGraph: true,
  },
  target: 'web',
  module: {
    rules: [
      {
        test: /\.[cm]?js$/,
        parser: {
          worker: ['*audioContext.audioWorklet.addModule()', '...'],
        },
      },
    ],
  },
});
