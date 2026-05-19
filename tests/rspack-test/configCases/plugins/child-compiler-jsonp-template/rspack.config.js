const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
    (compiler) => {
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
                path.join(__dirname, 'child.js'),
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
              .getAssets()
              .map((asset) => asset.name);
            expect(assets).toContain('child.js');
            expect(assets).toContain('asset.png');

            callback();
          });
        },
      );
    },
  ],
};
