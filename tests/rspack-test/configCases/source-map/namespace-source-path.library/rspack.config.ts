import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    library: 'mylibrary',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'cheap-source-map',
});
