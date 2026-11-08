import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      javascript: {
        exportsPresence: 'warn',
      },
    },
  },
});
