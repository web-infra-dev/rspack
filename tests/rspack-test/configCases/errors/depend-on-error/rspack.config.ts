import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    foo: './file-does-not-exist.js',
    bar: {
      import: ['./index.js'],
      dependOn: ['foo'],
    },
  },
  output: {
    filename: '[name].js',
  },
});
