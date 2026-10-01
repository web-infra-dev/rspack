import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    parser: {
      javascript: {
        wrappedContextRegExp: /.*1/,
      },
    },
  },
});
