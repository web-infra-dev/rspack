import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import { type Compiler, rspack } from '@rspack/core';

const oldHash = 'deadbeef';
const payload = Buffer.from([0xff, 0, 0x78]);

export default defineConfig({
  mode: 'production',
  target: 'web',
  devtool: 'source-map',
  output: {
    filename: `binary.${oldHash}.js`,
    hashFunction: 'xxhash64',
    crossOriginLoading: 'anonymous',
  },
  optimization: { minimize: false, realContentHash: true },
  plugins: [
    {
      apply(compiler: Compiler) {
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
          const asset = compilation.getAsset(filename)!;
          expect(asset.info.contenthash).toContain(
            `sha384-${createHash('sha384').update(payload).digest('base64')}`,
          );
          expect(asset.info.related?.sourceMap).toBeUndefined();
          expect(Buffer.isBuffer(asset.source.source())).toBe(true);
          expect(compilation.getAsset(`${filename}.map`)).toBeUndefined();
          expect(
            readFileSync(path.join(compiler.outputPath, filename)),
          ).toEqual(payload);
        });
      },
    },
    new rspack.SubresourceIntegrityPlugin(),
  ],
});
