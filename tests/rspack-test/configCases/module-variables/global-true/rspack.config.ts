import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: './index',
  },
  output: {
    environment: { globalThis: true },
  },
  node: {
    global: true,
  },
});
