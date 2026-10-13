import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { createFsFromVolume, Volume } from 'memfs';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

export default async function run() {
  const context = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-normal-module-accessors-')),
  );
  let compiler;
  try {
    fs.writeFileSync(path.join(context, 'index.js'), "import './broken.js';");
    fs.writeFileSync(path.join(context, 'broken.js'), 'export const = ;');
    compiler = rspack({
      context,
      entry: './index.js',
      mode: 'none',
      cache: { type: 'memory' },
      output: { path: path.join(context, 'dist') },
    });
    compiler.outputFileSystem = createFsFromVolume(new Volume());
    let generation = 0;
    let getError;
    compiler.hooks.done.tap('NormalModuleSharedErrors', ({ compilation }) => {
      const modules = [...compilation.modules];
      const entry = modules.find(
        (module) => module.resource === path.join(context, 'index.js'),
      );
      const broken = modules.find(
        (module) => module.resource === path.join(context, 'broken.js'),
      );
      assert(entry && broken);
      getError ||= Object.getOwnPropertyDescriptor(entry, 'error').get;
      assert.equal(
        getError,
        Object.getOwnPropertyDescriptor(broken, 'error').get,
      );
      assert.equal(getError.call(entry), undefined);
      if (generation === 0) {
        assert.equal(typeof getError.call(broken).message, 'string');
        assert.match(getError.call(broken).message, /JavaScript parse error/);
      } else {
        // Sharing functions must not memoize the previous receiver's error.
        assert.equal(getError.call(broken), undefined);
      }
    });
    const failed = await runCompiler(compiler);
    assert.equal(failed.hasErrors(), true);
    fs.writeFileSync(
      path.join(context, 'broken.js'),
      'export const value = 42;',
    );
    compiler.inputFileSystem.purge();
    generation++;
    const fixed = await runCompiler(compiler);
    assert.equal(fixed.hasErrors(), false);
  } finally {
    if (compiler) await closeCompiler(compiler);
    fs.rmSync(context, { recursive: true, force: true });
  }
}
