import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  resolve: {
    byDependency: {
      esm: {
        extensions: ['.bar', '...'],
      },
    },
  },
});
