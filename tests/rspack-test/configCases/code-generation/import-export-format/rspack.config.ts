import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  optimization: {
    concatenateModules: true,
    usedExports: true,
    providedExports: true,
    minimize: false,
    mangleExports: 'size',
    inlineExports: false,
  },
});
