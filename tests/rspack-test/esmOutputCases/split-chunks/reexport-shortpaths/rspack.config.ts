import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    main: './main.js',
    other: './other.js',
    third: './third.js',
  },
});
