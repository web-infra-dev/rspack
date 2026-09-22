import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: './index',
  },
  node: {
    __dirname: 'mock',
    __filename: 'mock',
  },
});
