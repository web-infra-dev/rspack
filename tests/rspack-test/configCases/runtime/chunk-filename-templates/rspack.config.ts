import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [false, true].map((templateLiteral) => ({
    target: 'web',
    output: {
      environment: { templateLiteral },
      chunkFilename: 'chunk-[id].[contenthash].js?`${literal}',
      cssChunkFilename: '[id].[contenthash].css',
    },
    optimization: { minimize: false, chunkIds: 'named' as const },
    module: {
      rules: [{ test: /\.css$/, type: 'css/auto' }],
    },
  })),
);
