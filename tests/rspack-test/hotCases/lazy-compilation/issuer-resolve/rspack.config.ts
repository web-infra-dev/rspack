import { defineConfig } from '@rspack/cli';

export default defineConfig({
  lazyCompilation: {
    entries: false,
    imports: true,
  },
  module: {
    rules: [
      {
        test: /index\.js$/,
        resolve: { extensions: ['...', '.txt'] },
      },
      {
        test: /\.txt$/,
        oneOf: [
          { issuer: /index\.js$/, type: 'asset/source' },
          { use: './no-issuer-loader.mjs' },
        ],
      },
    ],
  },
});
