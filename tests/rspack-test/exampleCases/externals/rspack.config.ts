import { defineConfig } from '@rspack/cli';

export default defineConfig({
  // mode: "development" || "production",
  output: {
    library: { type: 'umd' },
  },
  externals: [
    'add',
    {
      subtract: {
        root: 'subtract',
        commonjs2: './subtract',
        commonjs: ['./math', 'subtract'],
        amd: 'subtract',
      },
    },
  ],
});
