import { defineConfig, definePlugin } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  externals: {
    './child.js': 'commonjs ./child.js',
  },
  mode: 'development',
  target: 'web',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.make.tapAsync(
        'JsonpTemplatePluginChildCompilerNewUrlTest',
        (compilation, callback) => {
          const childCompiler = compilation.createChildCompiler(
            'jsonp-template-plugin-child-compiler-new-url',
            {
              filename: 'child.js',
              assetModuleFilename: '[name][ext]',
              publicPath: '',
            },
            [
              new compiler.rspack.web.JsonpTemplatePlugin(),
              new compiler.rspack.library.EnableLibraryPlugin('commonjs'),
              new compiler.rspack.EntryPlugin(
                compiler.context,
                path.join(import.meta.dirname, 'child.js'),
                {
                  name: 'child',
                  library: { type: 'commonjs' },
                },
              ),
            ],
          );

          childCompiler.runAsChild((err, _entries, childCompilation) => {
            if (err) {
              return callback(err);
            }

            const assets = childCompilation
              ?.getAssets()
              .map((asset) => asset.name);
            expect(assets).toContain('child.js');
            expect(assets).toContain('asset.png');

            callback();
          });
        },
      );
    }),
  ],
});
