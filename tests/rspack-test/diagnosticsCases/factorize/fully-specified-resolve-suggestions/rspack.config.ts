import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js$/,
        resolve: {
          fullySpecified: true,
        },
        type: 'javascript/esm',
      },
    ],
  },
});
