import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  node: {
    __filename: false,
    __dirname: false,
  },
});
