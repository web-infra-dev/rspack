import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  cache: false,
  plugins: [
    definePlugin({
      name: 'CountBuildsPlugin',
      builds: 0,
      apply(compiler) {
        compiler.hooks.done.tap(this.name, () => this.builds++);
      },
    }),
  ],
});
