import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.source\.js$/,
        type: 'asset/source',
        as: '*.txt',
        oneOf: [
          {
            test: /\.js$/,
            use: {
              loader: './prefix-loader.mjs',
              options: { prefix: 'wrong:' },
            },
          },
          {
            test: /\.txt$/,
            use: {
              loader: './prefix-loader.mjs',
              options: { prefix: 'nested:' },
            },
          },
        ],
      },
      {
        test: /\.txt$/,
        use: { loader: './prefix-loader.mjs', options: { prefix: 'next:' } },
      },
      {
        test: (resource: string) => /\.txt(?:[?#]|$)/.test(resource),
        use: { loader: './prefix-loader.mjs', options: { prefix: 'async:' } },
      },
    ],
  },
});
