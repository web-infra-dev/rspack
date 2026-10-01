import { defineConfig } from '@rspack/cli';

export default defineConfig({
  name: 'compiler-name',
  module: {
    rules: [
      {
        test: /a\.js$/,
        compiler: 'compiler',
        use: './loader',
      },
      {
        test: /b\.js$/,
        compiler: 'other-compiler',
        use: './loader',
      },
    ],
  },
});
