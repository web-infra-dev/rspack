import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  optimization: {
    usedExports: true,
    providedExports: true,
    mangleExports: 'size',
    inlineExports: true,
    concatenateModules: true,
    minimize: false,
  },
});
