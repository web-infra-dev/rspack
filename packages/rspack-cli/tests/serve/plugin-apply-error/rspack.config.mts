import { defineConfig } from '../../../dist/index.js';

export default defineConfig({
  mode: 'development',
  devtool: false,
  plugins: [
    {
      name: 'test-plugin',
      apply() {
        throw new Error('error in plugin');
      },
    },
  ],
  devServer: {},
});
