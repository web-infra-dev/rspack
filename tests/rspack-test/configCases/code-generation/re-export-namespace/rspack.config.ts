import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    concatenateModules: false,
    usedExports: true,
    providedExports: true,
    minimize: false,
    mangleExports: false,
    inlineExports: false,
  },
});
