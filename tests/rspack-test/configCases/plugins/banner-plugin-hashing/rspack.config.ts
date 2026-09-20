import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  entry: {
    'dist/banner': ['./index.js'],
    vendors: ['./vendors.js'],
  },
  output: {
    filename: '[name].js?value',
  },
  plugins: [
    new rspack.BannerPlugin({
      banner:
        'fullhash:[fullhash], chunkhash:[chunkhash], name:[name], base:[base], query:[query], file:[file], path:[path], ext:[ext]',
    }),
  ],
});
