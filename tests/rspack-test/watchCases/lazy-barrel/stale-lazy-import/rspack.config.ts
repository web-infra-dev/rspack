import { defineConfig } from '@rspack/cli';

export default defineConfig({
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
