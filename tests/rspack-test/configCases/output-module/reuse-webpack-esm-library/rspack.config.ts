import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  mode: 'development',
  devtool: 'eval',
  optimization: {
    concatenateModules: false,
  },
  resolve: {
    alias: {
      react: path.resolve(import.meta.dirname, 'react'),
    },
  },
});
