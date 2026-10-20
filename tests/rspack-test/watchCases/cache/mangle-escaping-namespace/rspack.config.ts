import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  cache: true,
  optimization: {
    usedExports: true,
    mangleExports: true,
    inlineExports: true,
    concatenateModules: false,
    minimize: false,
  },
});
