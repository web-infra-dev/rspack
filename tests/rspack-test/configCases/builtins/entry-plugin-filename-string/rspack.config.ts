import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.EntryPlugin(import.meta.dirname, './a.js', {
      filename: () => 'pages/[name].js',
      name: 'a',
    }),
  ],
});
