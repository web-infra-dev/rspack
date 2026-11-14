import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: {
    lazy: './index.js',
    asset: './asset.mjs',
    async: './async.mjs',
    shared: './shared.mjs',
  },
  externalsType: 'module',
  externals: ['./first.mjs', './second.mjs'],
  output: { assetModuleFilename: '[name][ext]' },
  optimization: { minimize: false, runtimeChunk: false, splitChunks: false },
});
