const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');

function assertStrict(context) {
  assert.equal(isMainThread, false);
  assert.equal(context.fromLoaderHook, undefined);
  assert.equal(context.parallel, true);
  assert.equal(context.strictFixture, 'worker option');
  assert.equal(context.getOptions().suffix, 'strict');
  assert.equal(context.utils.createHash().update('test').digest('hex').length, 64);
  assert.match(context._module.resource, /value\.js$/);
  assert.equal(context._module.resourceResolveData.path, context.resourcePath);
  assert.ok(context._module.loaders.some(({ loader }) => loader.includes('strict-loader.js')));
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
    assert.throws(access, /parallel: 'strict'/);
  }
  context.getLogger('test').debug('running on a worker');
}

module.exports = function (content, map, data) {
  assertStrict(this);
  assert.equal(data, undefined);
  if (this.getOptions().consume) {
    assert.equal(this._module.buildInfo.strictMetadata(), 'normal');
    const callback = this.async();
    queueMicrotask(() => callback(null, content, map, () => 'async data'));
    return;
  }
  assert.equal(this.data.pitched, true);
  assert.equal(this._module.buildInfo.strictMetadata, undefined);
  this._module.buildInfo.strictMetadata = () => 'normal';
  this.emitFile('strict.txt', Buffer.from('worker asset'), undefined, {
    immutable: true,
  });
  this.callback(null, content, map, { suffix: () => 'strict' });
};

module.exports.pitch = function (_remaining, _previous, data) {
  assertStrict(this);
  if (this.getOptions().consume) {
    this.callback(null, undefined, undefined, () => 'pitch data');
    return;
  }
  data.pitched = true;
  this._module.buildInfo = { strictMetadata: { phase: 'pitch' } };
};
