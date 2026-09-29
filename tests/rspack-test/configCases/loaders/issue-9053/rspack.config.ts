import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /c\.js$/,
        use: ['loader2'],
      },
      {
        test: /d\.js$/,
        use: ['loader3'],
      },
    ],
  },
});
