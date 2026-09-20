import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      javascript: {
        exportsPresence: 'auto',
      },
    },
  },
  experiments: {
    asyncWebAssembly: true,
  },
});
