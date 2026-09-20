import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: false,
  entry: {
    main: {
      import: './index.js',
    },
  },
  resolve: {
    mainFields: ['module', 'main'],
    extensionAlias: {
      '.js': ['.ts', '.js'],
    },
  },
});
