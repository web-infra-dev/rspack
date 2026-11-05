import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

export default defineConfig({
  context: import.meta.dirname,
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
        generator: {
          filename: '[name][ext]',
        },
      },
    ],
  },
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.thisCompilation.tap('PLUGIN', (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'PLUGIN',
            stage:
              compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE,
            // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
            additionalAssets: true,
          },
          (assets) => {
            const { RawSource, SourceMapSource } = compiler.rspack.sources;

            expect(assets['img.png']).toBeInstanceOf(RawSource);
            expect(assets['bundle0.js']).toBeInstanceOf(SourceMapSource);
          },
        );
      });
    },
  ],
});
