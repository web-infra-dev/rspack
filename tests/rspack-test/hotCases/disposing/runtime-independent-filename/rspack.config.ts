import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    hotUpdateMainFilename: '[hash].main-filename.json',
  },
});
