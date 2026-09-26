import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  mode: 'development',
  output: {
    library: {
      type: 'modern-module',
    },
    filename: '[name].js',
  },
  entry: {
    foo: './foo.js',
    bar: './bar.js',
  },
  externalsType: 'module',
  externals: function ({ request }) {
    if (request?.includes('value')) {
      // make '../modern-module-reexport-star/value' and './value'
      // the same request
      return path.resolve(import.meta.dirname, './value.js');
    }
  },
  optimization: {
    avoidEntryIife: true,
    concatenateModules: true,
    minimize: false,
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('MyPlugin', (compilation) => {
          compilation.hooks.processAssets.tap('MyPlugin', (assets) => {
            const list = Object.keys(assets);
            const js = list.find((item) => item.includes('foo.js'));
            const jsContent = assets[js!].source().toString();
            expect(
              // should make sure no default property access for default ExportsType
              jsContent,
            ).toContain('export * from');
          });
        });
      },
    }),
  ],
});
