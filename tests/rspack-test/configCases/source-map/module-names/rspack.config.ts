import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  output: {
    chunkFilename: '[name].js',
    devtoolModuleFilenameTemplate: 'module',
    devtoolFallbackModuleFilenameTemplate: 'fallback',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
});
