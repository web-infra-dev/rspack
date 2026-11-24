import { defineConfig } from '@rspack/cli';

export default defineConfig({
  cache: {
    type: 'persistent',
  },
  module: {
    parser: {
      javascript: {
        exportsPresence: 'error',
      },
    },
    rules: [
      {
        test: /pkg/,
        sideEffects: false,
      },
    ],
  },
});
