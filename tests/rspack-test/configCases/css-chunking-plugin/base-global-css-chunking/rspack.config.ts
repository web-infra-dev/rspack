import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
        type: 'javascript/auto',
      },
    ],
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      chunkFilename: '[name].css',
    }),
    new rspack.experiments.CssChunkingPlugin({
      strict: false,
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('PLUGIN', (stats) => {
          const json = stats.toJson({ all: false, assets: true });
          // Test scenario:
          // - Two independent pages: `page1` and `page2`
          // - Each page has its own CSS dependencies that don't interfere with each other
          // - In loose mode, CssChunkingPlugin should merge global CSS files
          //   within each page into separate chunks
          // - Expected result: 2 CSS assets (one per page)
          const cssAssets = json.assets?.filter((asset) =>
            asset.name?.endsWith('.css'),
          );
          expect(cssAssets?.length).toBe(2);
        });
      },
    }),
  ],
  experiments: {
    css: false,
  },
});
