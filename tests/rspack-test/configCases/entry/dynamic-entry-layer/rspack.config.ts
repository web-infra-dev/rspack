import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry() {
    return Promise.resolve({
      bundle0: {
        import: './index.js',
        layer: 'client',
      },
    });
  },
});
