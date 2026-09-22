import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: {
      import: './index.js',
      runtime: 'runtime',
    },
  },
  output: {
    filename: '[name].js',
  },
});
