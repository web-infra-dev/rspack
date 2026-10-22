import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    concatenateModules: true,
    inlineExports: true,
    mangleExports: false,
    minimize: false,
    providedExports: true,
    usedExports: true,
  },
});
