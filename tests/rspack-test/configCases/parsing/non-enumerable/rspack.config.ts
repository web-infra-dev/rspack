import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  module: {
    parser: {
      javascript: {
        exportsPresence: 'auto',
      },
    },
  },
});
