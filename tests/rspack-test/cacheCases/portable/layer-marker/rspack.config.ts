import { defineConfig, definePlugin } from '@rspack/cli';

const layer = '\0root/\0|\0escape|\0root\\\0|\0root/\0';

export default defineConfig({
  cache: { type: 'persistent', portable: { root: '.' } },
  entry: { main: { import: './index.js', layer } },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.done.tap('LayerMarkerTest', (stats) => {
          const modules = [...stats.compilation.modules].filter((module) =>
            module.identifier().includes('index.js'),
          );
          expect(modules).toHaveLength(1);
          expect(modules[0].layer).toBe(layer);
          expect(modules[0].identifier().endsWith(`|${layer}`)).toBe(true);
        });
      },
    }),
  ],
});
