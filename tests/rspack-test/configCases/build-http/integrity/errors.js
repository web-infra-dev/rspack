module.exports = require('./cases')
  .filter(test => test.error)
  .map(test => new RegExp(test.error));
