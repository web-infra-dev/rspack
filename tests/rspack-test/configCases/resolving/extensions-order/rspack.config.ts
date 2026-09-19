import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: {
    extensions: ['.js', '', '.json'],
  },
});
