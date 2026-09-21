import { defineConfig } from '@rspack/cli';
import TestApplyEntryOptionPlugin from './TestApplyEntryOptionPlugin.ts';

export default defineConfig({
  entry: {
    parent: './parent',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new TestApplyEntryOptionPlugin({
      entry: {
        child: './child',
      },
    }),
  ],
  stats: {
    assets: true,
    modules: true,
    children: true,
    entrypoints: true,
  },
});
