import assert from 'node:assert/strict';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  ignoreWarnings: [
    {
      module: /module2\.js\?[34]/,
    },
    {
      module: /[13]/,
      message: /homepage/,
    },
    /The 'mode' option has not been set/,
    (warning) => {
      assert(warning.module);
      return warning.module.identifier().endsWith('?2');
    },
  ],
  stats: {
    assets: true,
    modules: true,
  },
});
