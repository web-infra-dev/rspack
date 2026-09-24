import { defineConfig, definePlugin } from '@rspack/cli';

const plugin = definePlugin({
  apply(compiler) {
    let count = 0;
    compiler.hooks.shouldEmit.tap('should-emit-should-works', (compilation) => {
      expect(compilation).toBeDefined();
      expect(compilation.hooks).toBeDefined();
      count += 1;
    });
    compiler.hooks.shouldEmit.tap('check-next-tap', () => {
      count += 1;
    });

    compiler.hooks.done.tap('check', () => {
      expect(count).toBe(2);
    });
  },
});

export default defineConfig({
  context: import.meta.dirname,
  plugins: [plugin],
});
