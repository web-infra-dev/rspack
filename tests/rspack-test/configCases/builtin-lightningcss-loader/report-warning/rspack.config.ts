import assert from 'node:assert/strict';
import type { Compiler, LightningcssLoaderOptions } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.done.tap('PLUGIN', (stats) => {
      const json = stats.toJson();
      expect(json.warnings).toHaveLength(1);
      assert(json.warnings);
      expect(json.warnings[0].message).toMatch(
        /LightningCSS parse warning: Unexpected end of input at/,
      );
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
            options: {
              unusedSymbols: ['unused'],
              targets: '> 0.2%',
            } satisfies LightningcssLoaderOptions,
          },
        ],
        type: 'css/auto',
      },
    ],
  },

  plugins: [new Plugin()],
});
