import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: {
    main: './index.js',
  },
  optimization: {
    providedExports: true,
    usedExports: true,
    concatenateModules: true,
    minimize: false,
  },
});
