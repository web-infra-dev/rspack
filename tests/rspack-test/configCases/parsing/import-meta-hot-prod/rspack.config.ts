import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  target: 'web',
  devServer: {
    hot: true,
  },
});
