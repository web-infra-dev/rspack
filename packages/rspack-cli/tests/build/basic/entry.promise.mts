import { defineConfig } from '../../../dist/index.js';

export default defineConfig(() =>
  Promise.resolve({
    mode: 'development',
    entry: './src/entry.js',
  }),
);
