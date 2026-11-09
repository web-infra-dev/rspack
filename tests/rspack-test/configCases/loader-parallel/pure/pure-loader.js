const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

function assertPure(context) {
  assert.equal(isMainThread, false);
  assert.equal(context.fromLoaderHook, undefined);
  assert.equal(context.parallel, true);
  assert.equal(context.pureFixture, 'worker option');
  assert.equal(context.getOptions().suffix, 'pure');
  assert.equal(context.utils.createHash().update('test').digest('hex').length, 64);
  assert.match(context._module.resource, /value\.js$/);
  assert.equal(context._module.resourceResolveData.path, context.resourcePath);
  assert.ok(context._module.loaders.some(({ loader }) => loader.includes('pure-loader.js')));
  assert.deepEqual(context._module.blocks, []);
  assert.deepEqual(context._module.dependencies, []);
  assert.equal(context._module.error, undefined);
  assert.ok(context._module.buildInfo.fileDependencies instanceof Set);
  for (const access of [
    () => context._compiler,
    () => context._compilation,
    () => context._module.buildInfo.assets,
    () => context._module.buildInfo[Symbol.for('rspack.buildInfo.assets')],
    () => context.fs,
    () => context.loaders[0].options,
    () => context.getResolve(),
    () => context.importModule('./value.js'),
    () => context.emitWarning(new Error('unavailable')),
  ]) {
    assert.throws(access, /parallel: 'pure'/);
  }
  context.getLogger('test').debug('running on a worker');
}

module.exports = function (content) {
  assertPure(this);
  assert.equal(this.data.pitched, true);
  assert.deepEqual(this._module.buildInfo.pureMetadata, { phase: 'pitch' });
  this._module.buildInfo.pureMetadata.phase = 'normal';
  this.emitFile('pure.txt', Buffer.from('worker asset'), undefined, {
    immutable: true,
  });
  this.callback(null, content, undefined, { suffix: this.getOptions().suffix });
};

module.exports.pitch = function (_remaining, _previous, data) {
  assertPure(this);
  data.pitched = true;
  this._module.buildInfo = { pureMetadata: { phase: 'pitch' } };
};
