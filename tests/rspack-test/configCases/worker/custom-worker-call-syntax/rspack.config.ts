import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  target: 'web',
  module: {
    rules: [
      {
        test: /\.[cm]?js$/,
        parser: {
          worker: ['MyWorker()', '...'],
        },
      },
      {
        test: /\.[cm]?js$/,
        parser: {
          worker: { alias: ['...'] },
        },
      },
    ],
  },
});
