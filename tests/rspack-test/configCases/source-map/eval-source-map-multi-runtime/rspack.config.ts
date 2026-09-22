import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  entry: {
    a: './a.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
  },
  devtool: 'eval-source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  optimization: {
    concatenateModules: false,
    // inlineExports will inline lib.js into a.js, so the sourceFiles check will fail
    inlineExports: false,
  },
});
