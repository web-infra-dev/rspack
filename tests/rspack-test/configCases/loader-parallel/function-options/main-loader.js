const expectedOptions = require('./main-options');

module.exports = function () {
  return `module.exports = ${this.getOptions() === expectedOptions}`;
};
