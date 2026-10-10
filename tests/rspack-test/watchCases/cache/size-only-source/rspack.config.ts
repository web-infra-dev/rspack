import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  cache: true,
  devtool: 'source-map',
  entry: { main: './index.js', stable: './stable.js' },
  output: { filename: '[name].js', clean: false },
  plugins: [
    definePlugin({
      apply(compiler) {
        const sizes = new Map<string, number>();
        compiler.hooks.emit.tap('SizeOnlySource', (compilation) => {
          sizes.clear();
          for (const { name, source } of compilation.getAssets()) {
            sizes.set(name, source.buffer().length);
          }
        });
        compiler.hooks.afterEmit.tap('SizeOnlySource', (compilation) => {
          for (const { name, source } of compilation.getAssets()) {
            expect(source).toBeInstanceOf(
              compiler.rspack.sources.SizeOnlySource,
            );
            expect(source.size()).toBe(sizes.get(name));
          }
        });
      },
    }),
  ],
});
