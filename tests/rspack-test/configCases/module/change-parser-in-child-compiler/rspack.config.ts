import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  externals: {
    './__child-main.js': 'commonjs ./__child-main.js',
  },
  plugins: [
    definePlugin(function (compiler) {
      compiler.hooks.make.tapAsync('test', (compilation, callback) => {
        const child = compilation.createChildCompiler(
          'test',
          {
            filename: '__child-[name].js',
            publicPath: '',
          },
          [
            new compiler.rspack.library.EnableLibraryPlugin('commonjs'),
            new compiler.rspack.EntryPlugin(
              compilation.options.context!,
              './child-entry.js',
              {
                name: 'main',
                library: {
                  type: 'commonjs',
                },
              },
            ),
          ],
        );
        child.options.module.parser.javascript!.url = 'relative';
        child.runAsChild((error) => callback(error));
      });
    }),
  ],
});
