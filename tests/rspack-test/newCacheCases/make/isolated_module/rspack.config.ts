import { defineConfig, definePlugin } from '@rspack/cli';

let index = 0;

export default defineConfig({
  context: import.meta.dirname,
  entry: () => `./entry${index}.js`,
  optimization: {
    minimize: false,
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        index++;
        compiler.hooks.done.tapPromise('PLUGIN', async (stats) => {
          const { modules } = stats.toJson({ modules: true });
          const entry = modules?.filter((item) =>
            /entry[0-9]\.js$/.test(item.identifier ?? ''),
          );
          expect(entry?.length).toBe(1);
          expect(entry?.[0].identifier?.endsWith(`entry${index}.js`)).toBe(
            true,
          );
        });
      },
    }),
  ],
});
