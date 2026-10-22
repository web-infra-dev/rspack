import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    bundlerInfo: {
      force: ['version'],
    },
  },
});
