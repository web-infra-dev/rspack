import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'node14',
  externalsPresets: {
    node: true,
  },
  optimization: {
    concatenateModules: true,
    usedExports: true,
    providedExports: true,
    mangleExports: true,
  },
});
