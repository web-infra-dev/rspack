import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: {
    main: './index.js',
  },
  externals: {
    jquery: 'var { version: 1 }',
  },
  externalsPresets: {
    node: true,
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
});
