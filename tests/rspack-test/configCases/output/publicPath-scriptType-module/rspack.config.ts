import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'none',
  target: 'electron-renderer',
  output: {
    assetModuleFilename: '[name][ext]',
    importMetaName: 'pseudoImport.meta',
    scriptType: 'module',
    filename: 'index.mjs',
  },
  module: {
    rules: [
      {
        test: /\.jpg$/,
        type: 'asset/resource',
      },
    ],
  },
});
