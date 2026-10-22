import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

export default async function run() {
  const context = mkdtempSync(
    path.join(os.tmpdir(), 'rspack-files-versioned-'),
  );
  let compiler;
  let previous;
  let builds = 0;
  try {
    writeFileSync(
      path.join(context, 'index.js'),
      "import './style.css'; console.log(42);",
    );
    writeFileSync(path.join(context, 'style.css'), '.example { color: red }');
    compiler = rspack({
      context,
      entry: './index.js',
      mode: 'development',
      devtool: false,
      cache: false,
      incremental: true,
      experiments: { css: true },
      module: {
        rules: [
          { test: /\.css$/, type: 'css', parser: { exportType: 'link' } },
        ],
      },
      output: {
        path: path.join(context, 'dist'),
        filename: 'main.js',
        cssFilename: 'main.css',
      },
    });
    compiler.outputFileSystem = createFsFromVolume(new Volume());
    compiler.hooks.thisCompilation.tap('VersionedFiles', (compilation) => {
      const earlyFiles = new Map();
      compilation.hooks.optimizeTree.tap('VersionedFiles', () => {
        for (const chunk of compilation.chunks) {
          assert.deepEqual([...chunk.files], []);
          earlyFiles.set(chunk, chunk.files);
        }
      });
      compilation.hooks.processAssets.tap(
        { name: 'VersionedFiles', stage: 6000 },
        () => {
          const chunk = compilation.entrypoints
            .get('main')
            .getEntrypointChunk();
          if (previous) {
            assert.throws(() => previous.files, /Unable to access/);
          }
          const original = chunk.files;
          assert.equal(original, earlyFiles.get(chunk));
          earlyFiles.clear();
          assert.deepEqual([...original], ['main.css', 'main.js']);
          assert.equal(original, chunk.files);
          assert.equal(original.size, 2);
          assert.equal(original.has('main.js'), true);
          assert.equal(original.has('missing.js'), false);
          assert.deepEqual([...original.keys()], [...original]);
          assert.deepEqual([...original.values()], [...original]);
          assert.deepEqual(
            [...original.entries()],
            [
              ['main.css', 'main.css'],
              ['main.js', 'main.js'],
            ],
          );
          assert.equal(
            Object.prototype.toString.call(original),
            '[object Set]',
          );
          // The same implementation supports primitive and object elements.
          const ReadonlySet = original.constructor;
          const object = { name: 'group' };
          const symbol = Symbol('runtime');
          const mixed = new ReadonlySet(
            Object.freeze([NaN, 0, object, symbol, undefined]),
          );
          assert.equal(mixed.size, 5);
          assert(mixed.has(NaN));
          assert(mixed.has(-0));
          assert(mixed.has(object));
          assert(!mixed.has({ name: 'group' }));
          assert(mixed.has(symbol));
          assert(mixed.has(undefined));
          assert.deepEqual([...mixed], [NaN, 0, object, symbol, undefined]);
          assert.deepEqual([...mixed.entries()], [...new Set(mixed).entries()]);
          const empty = new ReadonlySet(Object.freeze([]));
          assert.equal(empty.size, 0);
          assert.deepEqual([...empty], []);
          assert.throws(() => empty.forEach(null), TypeError);
          const iterator = mixed.values();
          assert.deepEqual([...iterator], [...mixed]);
          assert.deepEqual(iterator.next(), { value: undefined, done: true });
          assert.deepEqual(iterator.next(), { value: undefined, done: true });
          const receiver = {};
          const visited = [];
          original.forEach(function (value, key, set) {
            assert.equal(this, receiver);
            assert.equal(set, original);
            assert.equal(key, value);
            visited.push(value);
          }, receiver);
          assert.deepEqual(visited, [...original]);
          assert.throws(() => original.forEach(null), TypeError);
          for (const method of [
            'union',
            'intersection',
            'difference',
            'symmetricDifference',
            'isSubsetOf',
            'isSupersetOf',
            'isDisjointFrom',
          ]) {
            if (typeof Set.prototype[method] === 'function') {
              const other = new Set(['main.js', 'other.js']);
              assert.deepEqual(
                original[method](other),
                new Set(original)[method](other),
              );
            }
          }
          for (const method of ['add', 'delete', 'clear']) {
            assert.equal(original[method], undefined);
          }
          const copy = new Set(original);
          copy.clear();
          copy.add('poison.js');
          assert.equal(chunk.files, original);
          assert.deepEqual([...chunk.files], ['main.css', 'main.js']);
          for (let i = 0; i < 3; i++) {
            const beforeRename = chunk.files;
            compilation.renameAsset('main.js', 'next.js');
            assert.equal(chunk.files, beforeRename);
            assert.equal(chunk.files, chunk.files);
            assert.deepEqual([...beforeRename], ['main.css', 'next.js']);
            assert.deepEqual([...chunk.files], ['main.css', 'next.js']);
            compilation.renameAsset('next.js', 'main.js');
            assert.equal(chunk.files, beforeRename);
            assert.deepEqual([...chunk.files], ['main.css', 'main.js']);
          }
          compilation.emitAsset(
            'unrelated.txt',
            new rspack.sources.RawSource('unrelated'),
          );
          compilation.deleteAsset('unrelated.txt');
          assert.deepEqual([...chunk.files], [...original]);
          compilation.deleteAsset('main.css');
          assert.deepEqual([...chunk.files], ['main.js']);
          compilation.deleteAsset('main.js');
          assert.deepEqual([...chunk.files], []);
          assert.equal(chunk.files, original);
          assert.deepEqual([...original], []);
          assert.equal(original.size, 0);
          previous = chunk;
          builds++;
        },
      );
    });
    await runCompiler(compiler);
    await runCompiler(compiler);
    assert.equal(builds, 2);
  } finally {
    if (compiler) {
      await closeCompiler(compiler);
    }
    rmSync(context, { recursive: true, force: true });
  }
}
