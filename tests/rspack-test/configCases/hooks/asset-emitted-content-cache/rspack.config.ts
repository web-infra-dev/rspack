import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { strict as assert } from 'node:assert';
import type { Source } from 'webpack-sources';

const pluginName = 'AssetEmittedContentCache';
const retry = 'cache-retry.txt';
const expected = new Map<string, Buffer>([
  ['cache-text.txt', Buffer.from('synthetic é')],
  ['cache-binary.dat', Buffer.from([0, 255, 128, 1])],
  ['cache-empty.txt', Buffer.alloc(0)],
  [retry, Buffer.from('retry')],
]);
const untouched = 'cache-untouched.txt';
const filenames = new Set([...expected.keys(), untouched]);

class Plugin {
  apply(compiler: Compiler) {
    const getAssetCalls = new Map<string, number>();
    const firstContents = new Map<string, Buffer>();
    const firstSources = new Map<string, Source>();
    const firstSeen = new Set<string>();
    const secondSeen = new Set<string>();

    compiler.hooks.thisCompilation.tap(pluginName, (compilation) => {
      compilation.hooks.processAssets.tap(pluginName, () => {
        const { RawSource } = compiler.rspack.sources;
        compilation.emitAsset('cache-text.txt', new RawSource('synthetic é'));
        compilation.emitAsset(
          'cache-binary.dat',
          new RawSource(Buffer.from([0, 255, 128, 1])),
        );
        compilation.emitAsset('cache-empty.txt', new RawSource(''));
        compilation.emitAsset(retry, new RawSource('retry'));
        compilation.emitAsset(untouched, new RawSource('untouched'));
      });

      // Count real lookups before emission, including any eager materialization.
      compiler.hooks.emit.tap(pluginName, () => {
        const getAsset = compilation.getAsset;
        compilation.getAsset = function (filename) {
          if (filenames.has(filename)) {
            getAssetCalls.set(filename, (getAssetCalls.get(filename) ?? 0) + 1);
            // Simulate a missing asset once; subsequent lookups use the real asset.
            if (filename === retry && getAssetCalls.get(filename) === 1) {
              return undefined;
            }
          }
          return getAsset.call(this, filename);
        };
      });
    });

    compiler.hooks.assetEmitted.tap(pluginName, (filename, info) => {
      if (!filenames.has(filename)) return;
      firstSeen.add(filename);
      assert.equal(getAssetCalls.get(filename) ?? 0, 0);
      if (filename === untouched) return;
      if (filename === retry) {
        assert.throws(() => info.source, {
          message: `Asset ${filename} not found`,
        });
        // A failed read must not prevent this second source access from succeeding.
        assert(info.source);
        assert.equal(getAssetCalls.get(filename), 2);
      }

      const first = info.content;
      assert(Buffer.isBuffer(first));
      assert.deepEqual(first, expected.get(filename));
      assert.strictEqual(info.content, first);
      assert.strictEqual(info.content, first);
      const source = info.source;
      assert.strictEqual(info.source, source);
      assert.deepEqual(first, source.buffer());
      assert.equal(getAssetCalls.get(filename), filename === retry ? 2 : 1);
      firstContents.set(filename, first);
      firstSources.set(filename, source);
    });

    // Both taps use the default stage and share the same info object.
    compiler.hooks.assetEmitted.tap(`${pluginName}Second`, (filename, info) => {
      if (!filenames.has(filename)) return;
      secondSeen.add(filename);
      if (filename === untouched) {
        assert.equal(getAssetCalls.get(filename) ?? 0, 0);
        return;
      }
      assert.strictEqual(info.content, firstContents.get(filename));
      assert.strictEqual(info.source, firstSources.get(filename));
      assert.equal(getAssetCalls.get(filename), filename === retry ? 2 : 1);
    });

    compiler.hooks.done.tap(pluginName, () => {
      assert.deepEqual(firstSeen, filenames);
      assert.deepEqual(secondSeen, filenames);
      assert.equal(getAssetCalls.get(untouched) ?? 0, 0);
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  devtool: false,
  plugins: [new Plugin()],
});
