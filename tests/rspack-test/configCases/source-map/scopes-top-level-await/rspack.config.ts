import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'node',
  devtool: 'source-map-scopes',
  optimization: { concatenateModules: false, minimize: false },
});
