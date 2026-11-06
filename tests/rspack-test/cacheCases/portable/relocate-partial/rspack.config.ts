import { DefinePlugin } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  plugins: [
    new DefinePlugin({ WRITER_ROOT: JSON.stringify(import.meta.dirname) }),
  ],
});
