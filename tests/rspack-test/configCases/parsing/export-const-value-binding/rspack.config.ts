import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    concatenateModules: false,
    inlineExports: false,
    mangleExports: false,
  },
});
