import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  cache: true,
  optimization: {
    inlineExports: true,
    usedExports: true,
  },
});
