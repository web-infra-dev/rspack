import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  optimization: {
    mangleExports: true,
    usedExports: true,
    providedExports: true,
    concatenateModules: false,
  },
});
