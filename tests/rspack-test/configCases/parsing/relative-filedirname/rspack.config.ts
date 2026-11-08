import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __filename: true,
    __dirname: true,
  },
});
