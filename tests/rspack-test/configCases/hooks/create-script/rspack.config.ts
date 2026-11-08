import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  output: {
    chunkLoading: 'jsonp',
    crossOriginLoading: 'anonymous',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const RuntimePlugin = compiler.rspack.RuntimePlugin;
        compiler.hooks.compilation.tap('mock-plugin', (compilation) => {
          const hooks = RuntimePlugin.getCompilationHooks(compilation);
          hooks.createScript.tap('mock-plugin', (code) => {
            return `${code}\nscript.setAttribute("data-create-script-injected", "true");`;
          });
        });
      },
    }),
  ],
});
