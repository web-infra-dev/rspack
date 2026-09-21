import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new rspack.HtmlRspackPlugin({
      templateContent:
        '<!DOCTYPE html><html><body><div><%= env %></div></body></html>',
      templateParameters: {
        env: 'production',
      },
      filename: 'index.html',
      customOptions: {
        property: 'value',
      },
    }),
    definePlugin({
      apply: (compiler) => {
        compiler.hooks.thisCompilation.tap(
          'HtmlRspackPlugin',
          (compilation) => {
            const hooks =
              rspack.HtmlRspackPlugin.getCompilationHooks(compilation);
            hooks.beforeEmit.tap('HtmlRspackPlugin', (htmlPluginData) => {
              expect(htmlPluginData.plugin.options.customOptions.property).toBe(
                'value',
              );
              return htmlPluginData;
            });
          },
        );
      },
    }),
  ],
});
