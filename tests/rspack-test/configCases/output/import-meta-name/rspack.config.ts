import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    importMetaName: 'pseudoImport.meta',
  },
  module: {
    parser: {
      javascript: {
        importMeta: false,
      },
    },
  },
});
