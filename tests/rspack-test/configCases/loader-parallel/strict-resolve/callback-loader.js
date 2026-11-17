const assert = require('node:assert/strict');
module.exports = function (source, map, data) {
  assert.equal(data, undefined);
  this._module.buildInfo = { workerOnly: () => 'local' };
  this.callback(null, source, map, () => 'must not cross strict loaders');
};
module.exports.pitch = function () {
  this.callback(null, undefined, undefined, () => 'must not stop pitching');
};
