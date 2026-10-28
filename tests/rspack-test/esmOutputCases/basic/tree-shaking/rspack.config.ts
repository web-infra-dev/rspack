import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    fs: 'module fs',
  },
  optimization: {
    usedExports: true,
  },
});
