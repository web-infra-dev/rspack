import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: () => {
          return [
            {
              loader: './loader1.mjs',
            },
          ];
        },
      },
      {
        test: /a\.js$/,
        use: () => {
          return [
            {
              loader: './loader2.mjs',
            },
          ];
        },
        enforce: 'pre',
      },
      {
        test: /a\.js$/,
        use: () => {
          return [
            {
              loader: './loader3.mjs',
            },
          ];
        },
        enforce: 'post',
      },
    ],
  },
});
