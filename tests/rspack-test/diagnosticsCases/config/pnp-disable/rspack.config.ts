import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: './index.js',
  },
  resolve: {
    pnp: false,
  },
});
