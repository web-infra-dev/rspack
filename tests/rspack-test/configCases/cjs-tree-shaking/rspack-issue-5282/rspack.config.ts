import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    minimize: false,
    moduleIds: 'named',
  },
});
