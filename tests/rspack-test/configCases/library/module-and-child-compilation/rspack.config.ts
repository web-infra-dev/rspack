import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  mode: 'production',
  target: 'web',
  output: {
    module: true,
    library: {
      type: 'module',
    },
  },
  module: {
    parser: {
      javascript: {
        exportsPresence: 'error',
      },
    },
    rules: [
      {
        test: /\.custom$/i,
        loader: fileURLToPath(import.meta.resolve('./loader.mjs')),
      },
    ],
  },
});
