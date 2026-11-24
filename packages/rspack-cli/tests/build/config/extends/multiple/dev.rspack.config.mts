import { defineConfig } from '../../../../../dist/index.js';

export default defineConfig({
  extends: './shared.config.json',
  devtool: 'eval-source-map',
  optimization: {
    minimize: false,
  },
});
