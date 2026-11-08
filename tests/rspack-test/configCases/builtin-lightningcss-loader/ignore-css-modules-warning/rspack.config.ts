import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.done.tap('PLUGIN', (stats) => {
      const json = stats.toJson();
      expect(json.warnings).toHaveLength(0);
    });
  }
}

export default defineConfig({
  module: {
    parser: {
      'css/auto': {
        namedExports: true,
      },
    },
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: 'builtin:lightningcss-loader',
            /** @type {import("@rspack/core").LightningcssLoaderOptions} */
            options: {
              unusedSymbols: ['unused'],
              targets: '> 0.2%',
            },
          },
        ],
        type: 'css/auto',
      },
    ],
  },

  plugins: [new Plugin()],
});
