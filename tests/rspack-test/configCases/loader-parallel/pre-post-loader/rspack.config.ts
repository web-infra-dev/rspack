import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: { loader: './loader1.mjs', parallel: true, options: {} },
      },
      {
        test: /a\.js$/,
        use: { loader: './loader2.mjs', parallel: true, options: {} },
        enforce: 'pre',
      },
      {
        test: /a\.js$/,
        use: {
          loader: './loader3.mjs',
          parallel: true,
          options: {},
        },
        enforce: 'post',
      },
    ],
  },
});
