import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: ['./strict'],
  module: {
    parser: {
      javascript: {
        overrideStrict: 'strict',
      },
    },
  },
});
