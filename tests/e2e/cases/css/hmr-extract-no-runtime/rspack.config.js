import { rspack } from '@rspack/core';

/** @type { import('@rspack/core').RspackOptions } */
export default {
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('RewriteCssUrls', (compilation) => {
          rspack.HtmlRspackPlugin.getCompilationHooks(
            compilation,
          ).beforeAssetTagGeneration.tap('RewriteCssUrls', (data) => {
            data.assets.css = data.assets.css.map((url) => `${url}?from=hook`);
            return data;
          });
        });
      },
    },
    new rspack.HtmlRspackPlugin({
      template: './src/index.html',
      inject: 'body',
    }),
    new rspack.CssExtractRspackPlugin({ runtime: false }),
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
};
