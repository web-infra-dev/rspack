import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: 'builtin:lightningcss-loader',
            /** @type {import("@rspack/core").LightningcssLoaderOptions} */
            options: {
              unusedSymbols: ['unused'],
              targets: '> 0.2%',
              drafts: 'xx',
            },
          },
        ],
        type: 'css/auto',
      },
    ],
  },
});
