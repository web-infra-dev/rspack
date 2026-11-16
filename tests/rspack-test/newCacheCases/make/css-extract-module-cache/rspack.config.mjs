import { defineConfig, definePlugin } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

let compilerIndex = 0;

export default defineConfig({
  mode: 'development',
  node: { __dirname: false },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [
    new CssExtractRspackPlugin(),
    definePlugin((compiler) => {
      let cssReused = false;
      compiler.hooks.compilation.tap('CssModuleCacheTest', (compilation) => {
        compilation.hooks.stillValidModule.tap(
          'CssModuleCacheTest',
          (module) => {
            if (module.identifier().startsWith('css|')) cssReused = true;
          },
        );
      });
      compiler.hooks.done.tap('CssModuleCacheTest', () => {
        expect(cssReused).toBe(compilerIndex++ === 1);
      });
    }),
  ],
});
