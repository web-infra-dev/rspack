import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.never-match$/,
        use: [
          {
            loader: './transform-loader.mjs',
            ident: 'string-filter',
            options: { value: 'string-hit' },
            codeFilter: 'MARKER_STR',
            parallel: true,
          },
          {
            loader: './transform-loader.mjs',
            ident: 'regexp-filter',
            options: { value: 'regexp-hit' },
            codeFilter: /MARKER_REGEXP/,
            parallel: true,
          },
        ],
      },
      {
        test: /\.virtual\.js$/,
        use: [
          './pitch-import-loader.mjs',
          {
            loader: './transform-loader.mjs',
            options: { value: 'pitched-hit' },
            codeFilter: 'MARKER_PITCH',
            parallel: true,
          },
        ],
      },
    ],
  },
});
