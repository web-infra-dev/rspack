import { defineConfig } from '../../../../../dist/index.js';

export default defineConfig({
  devtool: 'eval-source-map',
  optimization: {
    minimize: false,
  },
});
