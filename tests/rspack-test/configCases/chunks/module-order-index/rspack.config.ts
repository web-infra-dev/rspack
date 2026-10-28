import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  entry: {
    main: './index.js',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('test', (compilation) => {
          compilation.hooks.afterSeal.tap('test', () => {
            let entrypoint = compilation.entrypoints.get('main')!;

            compilation.chunkGraph
              .getChunkModules(entrypoint.chunks[0])
              .forEach((m) => {
                expect(entrypoint.getModulePreOrderIndex(m)).toBeDefined();
                expect(entrypoint.getModulePostOrderIndex(m)).toBeDefined();
              });
          });
        });
      },
    }),
  ],
});
