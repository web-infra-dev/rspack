const assert = require('node:assert/strict');
const path = require('node:path');
const { isMainThread } = require('node:worker_threads');

module.exports = async function (source, _map, additionalData) {
  assert.equal(this.fromHook, undefined);
  assert.equal(isMainThread, false);
  for (const access of [
    () => this._compiler,
    () => this._compilation,
    () => this.fs,
    () => this._module.buildInfo.assets,
    () => this.emitWarning(new Error('blocked')),
    () => this.emitError(new Error('blocked')),
    () => this.importModule('./value.js'),
  ]) assert.throws(access, /parallel: 'strict'/);

  assert.equal(additionalData, undefined);
  assert.equal(this._module.buildInfo.mainOnly, undefined);
  assert.equal(this._module.buildInfo.workerOnly(), 'local');
  const expected = path.join(this.context, 'target.custom');
  const resolve = this.getResolve({ extensions: ['.custom'] });
  assert.equal(await resolve(this.context, './target'), expected);
  assert.equal(await resolve(this.context, 'strict-target'), expected);
  assert.equal(await resolve(this.context, 'strict-ignored'), false);
  assert.equal(
    await this.getResolve({ dependencyType: 'esm' })(
      this.context,
      'strict-esm',
    ),
    expected,
  );
  assert.ok(this.getDependencies().includes(expected));
  await assert.rejects(
    resolve(this.context, './does-not-exist'),
    /does-not-exist/,
  );
  await new Promise((done, fail) =>
    resolve(this.context, './target', (error, result) => {
      try {
        assert.ifError(error);
        assert.equal(isMainThread, false);
        assert.equal(result, expected);
        done();
      } catch (error) {
        fail(error);
      }
    }),
  );
  await new Promise((done, fail) =>
    this.resolve(this.context, 'strict-target', (error, result) => {
      try {
        assert.ifError(error);
        assert.equal(result, expected);
        done();
      } catch (error) {
        fail(error);
      }
    }),
  );
  this.emitFile('strict-resolve.txt', Buffer.from('worker asset'));
  this._module.buildInfo.local = () => 'worker';
  assert.equal(this._module.buildInfo.local(), 'worker');
  return source;
};

module.exports.pitch = async function () {
  assert.equal(isMainThread, false);
  assert.equal(
    await this.getResolve()(this.context, 'strict-target'),
    path.join(this.context, 'target.custom'),
  );
};
