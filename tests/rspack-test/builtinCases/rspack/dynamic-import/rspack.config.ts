import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: ['node'],
  entry: {
    main: {
      import: ['./index.js'],
    },
  },
});
