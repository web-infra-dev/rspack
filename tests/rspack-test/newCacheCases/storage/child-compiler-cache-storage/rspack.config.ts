import { defineConfig, definePlugin } from '@rspack/cli';

let buildIndex = 0;
const built = { parent: 0, child: 0 };

export default defineConfig({
  entry: './a.js',
  cache: {
    type: 'persistent',
    maxMemoryGenerations: 0,
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        // The parent plugin is copied into child compilers; only track the root.
        if (compiler.isChild()) return;
        compiler.hooks.thisCompilation.tap('ChildSstTest', (compilation) => {
          compilation.hooks.buildModule.tap(
            'ChildSstTest',
            () => built.parent++,
          );
        });
        compiler.hooks.make.tapAsync(
          'ChildSstTest',
          (compilation, callback) => {
            const child = compilation.createChildCompiler(
              'child',
              { filename: 'child.js' },
              [
                new compiler.rspack.EntryPlugin(compiler.context, './b.js', {
                  name: 'main',
                }),
              ],
            );
            child.hooks.thisCompilation.tap(
              'ChildSstTest',
              (childCompilation) => {
                childCompilation.hooks.buildModule.tap(
                  'ChildSstTest',
                  () => built.child++,
                );
              },
            );
            child.runAsChild((error) => callback(error));
          },
        );
        compiler.hooks.done.tap('ChildSstTest', () => {
          // Reopening the same directory must recover both compilers' data from disk.
          expect(built).toEqual(
            buildIndex === 0
              ? { parent: 1, child: 1 }
              : { parent: 0, child: 0 },
          );
          built.parent = 0;
          built.child = 0;
          buildIndex++;
        });
      },
    }),
  ],
});
