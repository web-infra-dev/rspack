import { defineConfig } from '@speedy-js/speedy-core';
import { speedyPluginRspack } from '..';
module.exports = defineConfig({
  input: { main: './index.ts' },
  plugins: [speedyPluginRspack()],
});
