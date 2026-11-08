import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: {
    main: './index.js',
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  stats: {
    modules: true,
  },
});
