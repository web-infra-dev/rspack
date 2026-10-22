import { defineConfig } from '@rspack/cli';

export default defineConfig({
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'eval-source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  output: {
    devtoolFallbackModuleFilenameTemplate: 'fallback://[resource-path]?[hash]',
  },
  optimization: {
    moduleIds: 'named',
  },
});
