import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'node',
  optimization: {
    concatenateModules: true,
    minimize: true,
  },
});
