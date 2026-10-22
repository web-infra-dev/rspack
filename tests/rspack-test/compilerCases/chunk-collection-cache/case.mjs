import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import {
  closeCompiler,
  createGCTracker,
  forceGC,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

export default async function run() {
  const context = mkdtempSync(
    path.join(os.tmpdir(), 'rspack-chunk-collections-'),
  );
  const tracker = createGCTracker();
  let compiler;
  let retainedChunk;
  let retainedGroup;
  let retainedFiles;
  let retainedGroups;
  let retainedRuntime;
  let completed = 0;
  const groupsBeforeSplit = new Map();

  function checkSharedArray(read) {
    const first = read();
    assert.equal(read(), first);
    assert(Array.isArray(first));
    assert(!Object.isFrozen(first));
    const copy = [...first];
    copy.push('caller-owned');
    assert.equal(first.includes('caller-owned'), false);
    return first;
  }

  function checkCollections(object, properties) {
    for (const property of properties) {
      const first = object[property];
      const expected = [...first];
      const second = object[property];
      if (
        ['files', 'runtime', 'auxiliaryFiles', 'groupsIterable'].includes(
          property,
        )
      ) {
        assert.equal(first, second, property);
        assert.deepEqual([...second], expected, property);
        for (const method of ['add', 'delete', 'clear']) {
          assert.equal(first[method], undefined);
        }
        continue;
      }
      assert.equal(first, second, property);
      assert.deepEqual([...second], expected, property);
      checkSharedArray(() => object[property]);
      assert.deepEqual([...object[property]], expected, property);
    }
  }

  try {
    writeFileSync(
      path.join(context, 'index.js'),
      "import './shared.js'; import('./async.js'); import logo from './logo.svg'; console.log(logo);",
    );
    writeFileSync(
      path.join(context, 'async.js'),
      "export default () => import('./nested.js');",
    );
    writeFileSync(path.join(context, 'nested.js'), 'export default 42;');
    writeFileSync(
      path.join(context, 'other.js'),
      "import './shared.js'; console.log('other');",
    );
    writeFileSync(path.join(context, 'shared.js'), "console.log('shared');");
    writeFileSync(path.join(context, 'logo.svg'), '<svg></svg>');
    compiler = rspack({
      context,
      entry: { main: './index.js', other: './other.js' },
      optimization: {
        splitChunks: {
          cacheGroups: {
            shared: {
              test(module, { chunkGraph }) {
                if (!/shared\.js$/.test(module.nameForCondition() ?? '')) {
                  return false;
                }
                // Capture real graph state before SplitChunks inserts shared.
                for (const chunk of chunkGraph.getModuleChunksIterable(module)) {
                  for (const group of chunk.groupsIterable) {
                    if (!groupsBeforeSplit.has(group)) {
                      groupsBeforeSplit.set(group, {
                        array: group.chunks,
                        snapshot: [...group.chunks],
                        files: group.getFiles(),
                        origins: group.origins,
                        originSnapshot: [...group.origins],
                      });
                    }
                  }
                }
                return true;
              },
              name: 'shared',
              chunks: 'all',
              enforce: true,
            },
          },
        },
      },
      mode: 'development',
      cache: false,
      incremental: false,
      devtool: 'source-map',
      module: { rules: [{ test: /\.svg$/, type: 'asset/resource' }] },
      output: {
        path: path.join(context, 'dist'),
        filename: '[name].js',
        assetModuleFilename: '[name][ext]',
      },
    });
    compiler.outputFileSystem = createFsFromVolume(new Volume());
    compiler.hooks.thisCompilation.tap(
      'ChunkCollectionCache',
      (compilation) => {
        compilation.hooks.optimizeTree.tap('ChunkCollectionCache', () => {
          for (const chunk of compilation.chunks) {
            assert.deepEqual([...chunk.files], []);
            checkCollections(chunk, [
              'files',
              'runtime',
              'auxiliaryFiles',
              'groupsIterable',
            ]);
          }
        });
        compilation.hooks.processAssets.tapPromise(
          { name: 'ChunkCollectionCache', stage: 6000 },
          async () => {
            const entry = compilation.entrypoints.get('main');
            const chunk = entry.getEntrypointChunk();
            if (retainedChunk) {
              for (const property of [
                'files',
                'runtime',
                'auxiliaryFiles',
                'groupsIterable',
              ]) {
                assert.throws(
                  () => retainedChunk[property],
                  /Unable to access/,
                );
              }
              for (const property of ['chunks', 'childrenIterable', 'origins']) {
                assert.throws(
                  () => retainedGroup[property],
                  /Unable to access/,
                );
              }
              assert.throws(() => retainedGroup.getParents(), /Unable to access/);
              assert.throws(() => retainedGroup.getFiles(), /Unable to access/);
            }
            for (const current of compilation.chunks) {
              checkCollections(current, [
                'files',
                'runtime',
                'auxiliaryFiles',
                'groupsIterable',
              ]);
              assert.deepEqual([...current.files], [...current.files].sort());
            }
            for (const group of compilation.chunkGroups) {
              checkCollections(group, ['chunks', 'childrenIterable', 'origins']);
              const parents = checkSharedArray(() => group.getParents());
              const files = checkSharedArray(() => group.getFiles());
              for (const [symbolName, value] of [
                ['rspack.chunkGroup.files', files],
                ['rspack.chunkGroup.origins', group.origins],
              ]) {
                const descriptor = Object.getOwnPropertyDescriptor(
                  group, Symbol.for(symbolName),
                );
                assert.equal(descriptor.value, value);
                assert.equal(descriptor.enumerable, false);
              }
              assert.deepEqual(
                [...files].sort(),
                group.chunks.flatMap((chunk) => [...chunk.files]).sort(),
              );
              for (const [i, origin] of group.origins.entries()) {
                assert.equal(group.origins[i], origin);
                assert.equal(group.origins[i].loc, origin.loc);
                assert.equal(typeof origin.request, 'string');
                if (origin.module) {
                  assert([...compilation.modules].includes(origin.module));
                }
              }
              for (const parent of parents) {
                assert(parent.childrenIterable.includes(group));
              }
              for (const child of group.childrenIterable) {
                assert(child.getParents().includes(group));
              }
              assert(
                group.chunks.every((item) => compilation.chunks.has(item)),
              );
            }
            const asyncOrigin = compilation.chunkGroups
              .flatMap((group) => [...group.origins])
              .find((origin) => origin.request === './async.js');
            assert(asyncOrigin);
            assert(asyncOrigin.module.identifier().endsWith('index.js'));
            assert.equal(asyncOrigin.loc.start.line, 1);
            const shared = compilation.namedChunks.get('shared');
            assert(shared);
            assert(groupsBeforeSplit.size > 0);
            for (const [group, before] of groupsBeforeSplit) {
              assert(!before.snapshot.includes(shared));
              assert(group.chunks.includes(shared));
              assert.equal(group.chunks, before.array);
              assert(before.array.includes(shared));
              assert.equal(group.getFiles(), before.files);
              assert(before.files.includes('shared.js'));
              assert.equal(group.origins, before.origins);
              assert.deepEqual(group.origins, before.originSnapshot);
            }
            groupsBeforeSplit.clear();
            assert.deepEqual([...shared.runtime].sort(), ['main', 'other']);
            const groups = [...shared.groupsIterable];
            assert.equal(groups.length, 2);
            assert.deepEqual(
              groups.map((group) => group.index),
              groups.map((group) => group.index).sort((a, b) => a - b),
            );
            for (const group of groups) {
              assert(compilation.chunkGroups.includes(group));
              assert(group.chunks.includes(shared));
            }
            if (retainedGroups) {
              assert.throws(
                () => [...retainedGroups][0].index,
                /Unable to access/,
              );
              assert.deepEqual([...retainedRuntime].sort(), ['main', 'other']);
            }
            const entryFiles = entry.getFiles();
            const entryFileCount = entryFiles.length;
            const originalFiles = chunk.files;
            assert(originalFiles.has('main.js'));
            compilation.renameAsset('main.js', 'renamed.js');
            assert.deepEqual([...chunk.files], ['renamed.js']);
            assert.equal(chunk.files, originalFiles);
            assert(originalFiles.has('renamed.js'));
            assert(!originalFiles.has('main.js'));
            assert.equal(entry.getFiles(), entryFiles);
            assert(entryFiles.includes('renamed.js'));
            assert(!entryFiles.includes('main.js'));
            compilation.deleteAsset('renamed.js');
            assert.equal(chunk.files.size, 0);
            assert.equal(entry.getFiles(), entryFiles);
            assert.equal(entryFiles.length, entryFileCount - 1);
            assert(!entryFiles.includes('renamed.js'));

            // Asset modules and source maps populate auxiliaryFiles after graph creation.
            const auxiliaryView = chunk.auxiliaryFiles;
            const auxiliary = [...auxiliaryView];
            assert(auxiliary.length > 0);
            const original = auxiliary[0];
            compilation.renameAsset(original, `renamed-${original}`);
            assert.equal(chunk.auxiliaryFiles, auxiliaryView);
            assert.equal(chunk.auxiliaryFiles, chunk.auxiliaryFiles);
            assert(!auxiliaryView.has(original));
            assert(auxiliaryView.has(`renamed-${original}`));
            assert(!chunk.auxiliaryFiles.has(original));
            assert(chunk.auxiliaryFiles.has(`renamed-${original}`));
            compilation.deleteAsset(`renamed-${original}`);
            assert(!chunk.auxiliaryFiles.has(`renamed-${original}`));
            retainedFiles = originalFiles;
            retainedGroups = shared.groupsIterable;
            retainedRuntime = shared.runtime;
            retainedChunk = chunk;
            retainedGroup = entry;
            if (completed === 0) {
              tracker.track(chunk, 'previous compilation chunk');
              tracker.track(entry, 'previous compilation group');
              tracker.track(entry.chunks, 'previous chunks array');
              tracker.track(entry.getFiles(), 'previous group files array');
              tracker.track(entry.origins, 'previous origins array');
              tracker.track(entry.origins[0], 'previous origin object');
              tracker.track(entry.childrenIterable, 'previous children array');
              tracker.track(
                entry.childrenIterable[0].getParents(), 'previous parents array',
              );
            } else {
              tracker.track(entry.chunks, 'chunks array');
              tracker.track(entry.getFiles(), 'group files array');
              tracker.track(entry.origins, 'origins array');
              tracker.track(entry.origins[0], 'origin object');
              tracker.track(entry.childrenIterable, 'children array');
              tracker.track(entry.childrenIterable[0].getParents(), 'parents array');
            }
            // Keep only owners and weak handles across GC: each owner must keep
            // its cached arrays (and ReadonlySet facades) alive on the JS side.
            const cachedReads = [];
            for (const current of compilation.chunks) {
              for (const property of [
                'files',
                'runtime',
                'auxiliaryFiles',
                'groupsIterable',
              ]) {
                cachedReads.push([
                  () => current[property],
                  new WeakRef(current[property]),
                ]);
              }
            }
            for (const group of compilation.chunkGroups) {
              for (const property of ['chunks', 'childrenIterable', 'origins']) {
                cachedReads.push([
                  () => group[property],
                  new WeakRef(group[property]),
                ]);
              }
              cachedReads.push([
                () => group.getParents(),
                new WeakRef(group.getParents()),
              ]);
              cachedReads.push([
                () => group.getFiles(),
                new WeakRef(group.getFiles()),
              ]);
            }
            await forceGC(3, 5);
            for (const [read, weak] of cachedReads) {
              const cached = weak.deref();
              assert(cached, 'a live owner must retain its collection');
              assert.equal(read(), cached);
            }
            completed++;
          },
        );
      },
    );
    await runCompiler(compiler);
    await runCompiler(compiler);
    assert.equal(completed, 2);
    await tracker.waitForCollection('previous compilation chunk');
    await tracker.waitForCollection('previous compilation group');
    await tracker.waitForCollection('previous chunks array');
    await tracker.waitForCollection('previous children array');
    await tracker.waitForCollection('previous parents array');
    await tracker.waitForCollection('previous group files array');
    await tracker.waitForCollection('previous origins array');
    await tracker.waitForCollection('previous origin object');
    tracker.track(compiler, 'compiler');
    tracker.track(retainedChunk, 'chunk');
    tracker.track(retainedGroup, 'chunk group');
    tracker.track(retainedFiles, 'files view');
    tracker.track(retainedGroups, 'groups view');
    tracker.track([...retainedGroups][0], 'cached group');
  } finally {
    if (compiler) {
      await closeCompiler(compiler);
    }
    rmSync(context, { recursive: true, force: true });
  }
  compiler = null;
  await tracker.waitForCollection('compiler');
  assert.equal(retainedFiles.size, 0);
  assert.throws(() => retainedChunk.files, /Unable to access/);
  assert.throws(() => retainedGroup.chunks, /Unable to access/);
  assert.throws(() => retainedGroup.childrenIterable, /Unable to access/);
  assert.throws(() => retainedGroup.origins, /Unable to access/);
  assert.throws(() => retainedGroup.getParents(), /Unable to access/);
  assert.throws(() => retainedGroup.getFiles(), /Unable to access/);
  retainedChunk = null;
  retainedGroup = null;
  assert.throws(() => [...retainedGroups][0].index, /Unable to access/);
  retainedGroups = null;
  retainedRuntime = null;
  await tracker.waitForCollection('chunk');
  await tracker.waitForCollection('chunk group');
  retainedFiles = null;
  await tracker.waitForCollection('files view');
  await tracker.waitForCollection('groups view');
  await tracker.waitForCollection('cached group');
  await tracker.waitForCollection('chunks array');
  await tracker.waitForCollection('children array');
  await tracker.waitForCollection('parents array');
  await tracker.waitForCollection('group files array');
  await tracker.waitForCollection('origins array');
  await tracker.waitForCollection('origin object');
}
