import { defineConfig } from '@rspack/cli';

export default defineConfig({
  lazyCompilation: {
    entries: false,
    imports: true,
  },
  module: {
    rules: [
      {
        test: /\.json$/,
        oneOf: [
          { with: { type: 'json' }, type: 'json' },
          { use: './plain-json-loader.mjs' },
        ],
      },
    ],
  },
});
