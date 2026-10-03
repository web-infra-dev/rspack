import { defineConfig } from '@rspack/cli';
import { rspack, type Compiler } from '@rspack/core';

export default defineConfig(
  [true, false].map((runtime) => ({
    target: 'web',
    externals: { fs: 'commonjs fs', path: 'commonjs path' },
    node: { __dirname: false },
    mode: 'development',
    output: {
      uniqueName: 'html-css',
      publicPath: '/assets/',
      cssFilename: 'native-[name].css',
    },
    module: {
      rules: [
        { test: /native\.css$/, type: 'css/auto' },
        {
          test: /style\.css$/,
          type: 'javascript/auto',
          use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
        },
      ],
    },
    plugins: [
      {
        apply(compiler: Compiler) {
          compiler.hooks.compilation.tap('RewriteCssUrls', (compilation) => {
            rspack.HtmlRspackPlugin.getCompilationHooks(
              compilation,
            ).beforeAssetTagGeneration.tap('RewriteCssUrls', (data) => {
              data.assets.css = data.assets.css
                .map((url) =>
                  url.replace('/assets/', 'https://cdn.example.com/'),
                )
                .reverse();
              return data;
            });
          });
        },
      },
      new rspack.CssExtractRspackPlugin({
        filename: 'extract-[name].css',
        runtime,
      }),
      new rspack.HtmlRspackPlugin({
        filename: runtime ? 'runtime.html' : 'no-runtime.html',
        minify: false,
        hash: true,
        templateContent:
          '<html><head><link href="framework.css" rel="stylesheet"></head><body></body></html>',
      }),
      new rspack.DefinePlugin({
        HTML_FILE: JSON.stringify(runtime ? 'runtime.html' : 'no-runtime.html'),
      }),
    ],
  })),
);
