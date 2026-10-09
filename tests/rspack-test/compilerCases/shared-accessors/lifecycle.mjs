import assert from 'node:assert/strict';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { rspack } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import {
  closeCompiler,
  createGCTracker,
  forceGC,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

export default async function run() {
  const tracker = createGCTracker();
  const context = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-shared-accessors-')),
  );
  try {
    const entryPath = path.join(context, 'index.js');
    const valuePath = path.join(context, 'value.js');
    const binaryRequest = 'data:application/octet-stream;base64,AP9h';
    fs.writeFileSync(
      entryPath,
      `import './value.js'; import '${binaryRequest}';`,
    );
    fs.writeFileSync(valuePath, 'export default 42;');
    let compiler = rspack({
      context,
      entry: './index.js',
      mode: 'none',
      devtool: 'source-map',
      module: {
        rules: [{ mimetype: 'application/octet-stream', type: 'asset/inline' }],
      },
      cache: false,
      output: { path: path.join(context, 'dist') },
    });
    compiler.outputFileSystem = createFsFromVolume(new Volume());
    let entry, removed, info, stats;
    let originalSource, originalMap, binarySource;
    let generation = 0;
    const fileKey = Symbol.for('rspack.buildInfo.fileDependencies');
    let getContext, getFiles, getMap;
    compiler.hooks.done.tap('SharedAccessorsLifecycle', ({ compilation }) => {
      const modules = [...compilation.modules];
      const current = modules.find((module) => module.resource === entryPath);
      assert(current);
      if (generation === 0) {
        entry = current;
        removed = modules.find((module) => module.resource === valuePath);
        assert(removed);
        originalSource = removed.originalSource();
        getMap = Object.getOwnPropertyDescriptor(
          originalSource,
          '_sourceMapAsString',
        ).get;
        const binaryModule = modules.find(
          (module) => module.resource === binaryRequest,
        );
        assert(binaryModule);
        binarySource = binaryModule.originalSource();
        info = removed.buildInfo;
        getContext = Object.getOwnPropertyDescriptor(entry, 'context').get;
        getFiles = Object.getOwnPropertyDescriptor(info, fileKey).get;
        assert.equal(
          getContext,
          Object.getOwnPropertyDescriptor(removed, 'context').get,
        );
        assert.equal(
          getFiles,
          Object.getOwnPropertyDescriptor(entry.buildInfo, fileKey).get,
        );
        assert(getFiles.call(info).includes(valuePath));
      } else {
        assert.equal(entry.context, context);
        assert.equal(getContext.call(current), context);
        assert.throws(() => removed.context, /removed on the Rust side/);
        assert.match(getFiles.call(info).message, /removed on the Rust side/);
        assert.throws(
          () => info[Symbol.for('rspack.buildInfo.assets')],
          /removed on the Rust side/,
        );
      }
    });
    try {
      stats = await runCompiler(compiler);
      assert.equal(stats.hasErrors(), false);
      await forceGC(3);
      fs.writeFileSync(entryPath, 'export default 43;');
      generation++;
      stats = await runCompiler(compiler);
      assert.equal(stats.hasErrors(), false);
    } finally {
      await closeCompiler(compiler);
    }
    // close keeps the current compilation accessible while its compiler lives.
    assert.equal(entry.context, context);
    tracker.track(compiler, 'compiler');
    tracker.track(entry, 'module');
    tracker.track(removed, 'removed module');
    tracker.track(info, 'build info');
    stats = null;
    compiler = null;
    await tracker.waitForCollection('compiler');
    assert.throws(
      () => getContext.call(entry),
      /Compiler has been garbage collected/,
    );
    entry = null;
    removed = null;
    await tracker.waitForCollection('module');
    await tracker.waitForCollection('removed module');
    assert.match(
      getFiles.call(info).message,
      /Module has been garbage collected/,
    );
    assert.throws(
      () => info[Symbol.for('rspack.buildInfo.assets')],
      /Module has been garbage collected/,
    );
    info = null;
    await tracker.waitForCollection('build info');
    // The unread map is an owned snapshot, even after module removal, close and owner GC.
    assert.equal(originalSource.source(), 'export default 42;');
    originalMap = originalSource.map();
    assert.deepEqual(originalMap.sourcesContent, ['export default 42;']);
    assert.deepEqual(JSON.parse(getMap.call(originalSource)), originalMap);
    tracker.track(originalSource, 'original source');
    originalSource = null;
    await tracker.waitForCollection('original source');
    assert.deepEqual(binarySource.source(), Buffer.from([0, 255, 97]));
    binarySource.source()[0] = 42;
    assert.deepEqual(binarySource.buffer(), Buffer.from([42, 255, 97]));
    // Keeping the accessor functions alive must not retain any of those owners.
    assert.equal(typeof getContext, 'function');
    assert.equal(typeof getFiles, 'function');
    assert.equal(typeof getMap, 'function');
  } finally {
    fs.rmSync(context, { recursive: true, force: true });
  }
}
