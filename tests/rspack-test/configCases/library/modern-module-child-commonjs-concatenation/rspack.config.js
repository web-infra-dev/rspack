const path = require('path');

class ModernModuleChildCompilerPlugin {
  /** @param {import("@rspack/core").Compiler} compiler */
  apply(compiler) {
    compiler.hooks.make.tapAsync(
      'ModernModuleChildCompilerPlugin',
      (compilation, callback) => {
        const { EntryPlugin, library } = compiler.rspack;
        const childCompiler = compilation.createChildCompiler(
          'modern-module-child',
          {
            filename: 'child.mjs',
            library: { type: 'modern-module' },
          },
          [
            new EntryPlugin(
              compiler.context,
              path.resolve(__dirname, 'child-entry.js'),
              { name: 'child' },
            ),
            new library.EnableLibraryPlugin('modern-module'),
          ],
        );

        childCompiler.runAsChild((error, _entries, childCompilation) => {
          if (error) {
            callback(error);
            return;
          }
          if (!childCompilation) {
            callback(new Error('Child compilation was not created'));
            return;
          }
          if (childCompilation.errors.length > 0) {
            callback(childCompilation.errors[0]);
            return;
          }

          try {
            const asset = childCompilation.getAsset('child.mjs');
            expect(asset).toBeDefined();
            const source = asset.source.source().toString();
            expect(source).toContain('RSPACK_CJS_EXPORTS');
            expect(source).not.toContain('__webpack_modules__');
            callback();
          } catch (assertionError) {
            callback(assertionError);
          }
        });
      },
    );
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  devtool: false,
  output: {
    filename: 'main.mjs',
    library: { type: 'modern-module' },
    module: true,
  },
  optimization: {
    concatenateModules: { commonjs: true },
    minimize: false,
  },
  plugins: [new ModernModuleChildCompilerPlugin()],
};
