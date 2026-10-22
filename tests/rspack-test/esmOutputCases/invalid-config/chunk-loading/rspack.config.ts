import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    chunkLoading: 'jsonp',
  },
  optimization: {
    concatenateModules: true,
  },
});
