import { defineConfig, definePlugin } from '@rspack/cli';

const options: { files: string[] } = { files: [] };

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /file\.js$/,
        loader: './loader.mjs',
        options,
      },
    ],
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    definePlugin({
      updateIndex: 0,
      apply(compiler) {
        compiler.hooks.done.tap('Test', () => {
          if (this.updateIndex == 0) {
            expect(options.files.length).toBe(1);
          }
          if (this.updateIndex == 1) {
            expect(options.files.length).toBe(1);
          }
          if (this.updateIndex == 2) {
            expect(options.files.length).toBe(0);
          }
          if (this.updateIndex == 3) {
            expect(options.files.length).toBe(0);
          }
          if (this.updateIndex == 4) {
            expect(options.files.length).toBe(1);
          }
          options.files = [];
          this.updateIndex++;
        });
      },
    }),
  ],
});
