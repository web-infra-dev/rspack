import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: {
      import: './index.js',
      runtime: false,
    },
  },
  output: {
    filename: '[name].js',
  },
});
