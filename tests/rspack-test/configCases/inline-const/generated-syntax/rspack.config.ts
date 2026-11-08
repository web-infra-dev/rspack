import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    inlineExports: true,
  },
});
