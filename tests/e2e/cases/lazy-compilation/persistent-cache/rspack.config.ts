import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  entry: {
    main: ['./src/component.js', './src/index.js'],
  },
  stats: 'none',
  mode: 'development',
  plugins: [
    new rspack.HtmlRspackPlugin(),
    {
      apply(compiler: Compiler) {
        compiler.hooks.done.tap('TEST', function (stats) {
          const { modules } = stats.toJson({ modules: true });
          Object.assign(compiler, {
            __modules: modules!.map((item) => item.identifier),
          });
        });
      },
    },
  ],
  lazyCompilation: true,
  cache: {
    type: 'persistent',
  },
  devServer: {
    hot: true,
    client: {
      overlay: {
        // hide warnings for incremental
        warnings: false,
      },
    },
  },
});
