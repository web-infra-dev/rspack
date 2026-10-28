import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js/,
        resolve: {
          conditionNames: ['server'],
        },
      },
      {
        test: /reexports\.js/,
        resolve: {
          alias: {
            'server-lib': 'lib2',
          },
        },
      },
    ],
  },
  resolve: {
    modules: ['modules'],
  },
});
