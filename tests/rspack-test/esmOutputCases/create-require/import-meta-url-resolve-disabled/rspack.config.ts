import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      javascript: {
        createRequire: true,
        requireResolve: false,
      },
    },
    rules: [
      {
        test: /preserve-import-meta\.js$/,
        parser: {
          importMeta: false,
        },
      },
    ],
  },
});
