import { readFileSync } from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const oldHash = 'deadbeef';
const payload = Buffer.from([0xff, 0, 0x78]);

export default defineConfig({
  mode: 'production',
  target: 'web',
  devtool: false,
  output: {
    filename: `binary.${oldHash}.js`,
    hashFunction: 'xxhash64',
    crossOriginLoading: 'anonymous',
  },
  optimization: { minimize: false, realContentHash: true },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('Test', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'Test',
              stage:
                compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
            },
            () => {
              compilation.updateAsset(
                `binary.${oldHash}.js`,
                new compiler.webpack.sources.RawSource(payload),
                { contenthash: oldHash },
              );
            },
          );
        });
        compiler.hooks.afterEmit.tap('Test', (compilation) => {
          // SRI wraps the chunk in a native ReplaceSource. With no integrity
          // placeholders it stays unedited, so its buffer remains binary.
          const hash = compiler.webpack.util.createHash('xxhash64');
          hash.update(payload);
          const expectedHash = hash.digest('hex').slice(0, oldHash.length);
          const filename = `binary.${expectedHash}.js`;
          expect(compilation.getAsset(filename)).toBeDefined();
          expect(
            readFileSync(path.join(compiler.outputPath, filename)),
          ).toEqual(payload);
        });
      },
    },
    new rspack.SubresourceIntegrityPlugin(),
  ],
});
