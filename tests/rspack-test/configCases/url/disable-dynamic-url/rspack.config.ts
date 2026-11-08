import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  module: {
    parser: {
      javascript: {
        dynamicUrl: false,
      },
    },
  },
});
