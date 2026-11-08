import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].js',
    library: {
      name: 'MyLibraryRuntimeChunk',
      type: 'assign',
    },
  },
  target: 'web',
  optimization: {
    runtimeChunk: true,
  },
});
