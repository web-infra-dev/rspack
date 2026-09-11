const assert = require('node:assert/strict');
module.exports = async function (data, options) {
  assert.equal(require('node:worker_threads').isMainThread, false);
  if (data.request === './ignored') return false;
  if (data.request !== options.from) return;
  if (options.mainThread) assert.equal(options.mainThread(), true);
  if (options.nested) assert.equal(await options.nested('./step-'), options.to);
  data.request = options.to;
  data.fileDependencies.push(__filename);
};
