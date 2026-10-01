import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    library: { type: 'system' },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
});
