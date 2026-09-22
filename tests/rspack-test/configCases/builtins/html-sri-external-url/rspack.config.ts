import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  mode: 'production',
  target: 'web',
  output: {
    crossOriginLoading: 'anonymous',
  },
  node: {
    __dirname: false,
  },
  plugins: [
    new rspack.HtmlRspackPlugin({
      filename: 'index.html',
    }),
    new rspack.SubresourceIntegrityPlugin({
      hashFuncNames: ['sha384'],
      enabled: true,
    }),
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          rspack.HtmlRspackPlugin.getCompilationHooks(
            compilation,
          ).beforeAssetTagGeneration.tap('TestPlugin', (data) => {
            data.assets.js.push(
              'https://cdn.jsdelivr.net/npm/react@18/umd/react.production.min.js',
            );
            return data;
          });
        });
      },
    }),
  ],
});
