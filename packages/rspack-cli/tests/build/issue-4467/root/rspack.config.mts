import { defineConfig } from '../../../../dist/index.js';
import { rspack } from '@rspack/core';
import path from 'node:path';

const config = defineConfig({
  entry: path.resolve(import.meta.dirname, './index.js'),
  plugins: [
    new rspack.BannerPlugin({
      banner: '',
    }),
  ],
});

export default config;
