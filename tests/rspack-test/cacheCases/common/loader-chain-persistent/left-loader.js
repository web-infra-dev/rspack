module.exports = function (source) {
  this.addDependency(`${this.rootContext}/trigger.js`);
  return source;
};
