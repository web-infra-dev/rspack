import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    environment: {
      arrowFunction: false,
      const: false,
    },
  },
});
