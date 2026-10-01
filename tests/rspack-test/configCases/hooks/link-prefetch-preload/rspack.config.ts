import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    chunkFilename: '[name].js',
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const RuntimePlugin = compiler.rspack.RuntimePlugin;
        compiler.hooks.compilation.tap('mock-plugin', (compilation) => {
          const hooks = RuntimePlugin.getCompilationHooks(compilation);
          hooks.linkPrefetch.tap('mock-plugin', (code) => {
            return `${code}\nlink.setAttribute("data-prefetch-injected", "true");`;
          });
          hooks.linkPreload.tap('mock-plugin', (code) => {
            return `${code}\nlink.setAttribute("data-preload-injected", "true");`;
          });
        });
      },
    }),
  ],
});
