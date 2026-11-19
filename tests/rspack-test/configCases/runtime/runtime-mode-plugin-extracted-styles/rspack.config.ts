import { defineConfig, definePlugin } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  target: 'web',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  output: { publicPath: '', chunkFilename: '[name].js' },
  optimization: { chunkIds: 'named' },
  plugins: [
    new CssExtractRspackPlugin({ chunkFilename: '[name].[contenthash:8].css' }),
    definePlugin((compiler) => {
      const { RuntimeGlobals: g, RuntimeModule } = compiler.rspack;
      class IsolatedStyles extends RuntimeModule {
        constructor() {
          super('isolated styles', RuntimeModule.STAGE_ATTACH);
        }
        generate() {
          return `
          __webpack_require__.styleHost = function () { return document.querySelector('#host').shadowRoot; };
          var loadChunk = __webpack_require__.e;
          __webpack_require__.e = function (id) {
            return loadChunk(id).catch(function (error) {
              if (id !== 'second' || __webpack_require__.rbLoadStyleSheet) throw error;
              __webpack_require__.rbLoadStyleSheet = (href, chunkId) => '/backup/' + chunkId + '/' + href;
              return loadChunk(id);
            });
          };`;
        }
      }
      compiler.hooks.thisCompilation.tap('IsolatedStyles', (compilation) => {
        compilation.hooks.additionalTreeRuntimeRequirements.tap(
          'IsolatedStyles',
          (chunk, set) => {
            set.add(g.publicPath);
            compilation.addRuntimeModule(chunk, new IsolatedStyles());
          },
        );
        compilation.hooks.runtimeModule.tap('IsolatedStyles', (mod) => {
          // The extractor calls this "css loading"; built-in CSS calls its module "css_loading".
          // Its identifier prefix and the representation of source can vary between modes.
          if (mod.name !== 'css loading' || !mod.source) return;
          const code = mod.source.source.toString();
          mod.source.source = Buffer.from(
            code
              .replace(
                'var fullhref = ',
                'var fullhref = __webpack_require__.rbLoadStyleSheet ? __webpack_require__.rbLoadStyleSheet(href, chunkId) : ',
              )
              .replace(
                'document.head.appendChild(linkTag)',
                '__webpack_require__.styleHost().appendChild(linkTag)',
              ) +
              `
              // addon-bone can wrap the installed handler without relying on an internal loader name.
              var loadDocumentCss = __webpack_require__.f.miniCss;
              __webpack_require__.f.miniCss = function (id, promises) {
                if (id !== 'isolated') return loadDocumentCss.apply(this, arguments);
                var link = document.createElement('link');
                link.rel = 'stylesheet';
                link.href = __webpack_require__.p + __webpack_require__.miniCssF(id);
                promises.push(new Promise(function (resolve, reject) {
                  link.onload = resolve;
                  link.onerror = reject;
                  __webpack_require__.styleHost().appendChild(link);
                }));
              };`,
          );
        });
      });
    }),
  ],
});
