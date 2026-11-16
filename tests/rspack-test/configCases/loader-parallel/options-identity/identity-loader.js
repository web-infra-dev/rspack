const assert = require('node:assert/strict');
const { isMainThread } = require('node:worker_threads');
const seen = new Map();
const owners = new WeakSet();
module.exports = function () {
  assert.equal(isMainThread, false);
  const options = this.getOptions();
  // The entry loader gives each compilation its own group, including restarts.
  const generation = this.resourceQuery.split(':')[0];
  const previous = seen.get(generation);
  if (previous) {
    assert.equal(options, previous.options);
    assert.equal(options.nested, previous.options.nested);
    if (options.policy === 'true') {
      assert.equal(this._compiler.options, previous.compilerOptions);
    }
  } else {
    assert.equal(owners.has(options), false);
    owners.add(options);
    seen.set(generation, {
      options,
      compilerOptions: options.policy === 'true' ? this._compiler.options : undefined,
    });
  }
  assert.equal(options.nested.value, 42);
  return `module.exports = ${JSON.stringify({ mode: this.mode, reused: !!previous })};`;
};
