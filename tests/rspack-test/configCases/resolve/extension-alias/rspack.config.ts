import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  resolve: {
    extensionAlias: {
      '.mjs': ['.mts'],
    },
  },
});
