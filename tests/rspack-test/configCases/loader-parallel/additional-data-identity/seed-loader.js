module.exports = function (content) {
  const value = require('./value');
  value.count = 0;
  this.callback(null, content, null, value);
};
