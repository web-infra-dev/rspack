import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  context: import.meta.dirname,
  optimization: {
    moduleIds: 'named',
    minimize: false,
  },
});
