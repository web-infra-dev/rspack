const expectedOptions = require('./main-options');

module.exports = function () {
  if (!this.hookMainThread || this.hookTransform(this.hookValue) !== 'hook:context')
    throw new Error('Main loader hook should still run');
  return `module.exports = ${this.getOptions() === expectedOptions}`;
};
