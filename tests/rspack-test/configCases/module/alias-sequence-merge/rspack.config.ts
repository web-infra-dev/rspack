import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        resolve: {
          alias: {
            foo: './not-exist',
          },
        },
      },
      {
        resolve: {
          alias: {
            foo: './exist',
          },
        },
      },
    ],
  },
});
