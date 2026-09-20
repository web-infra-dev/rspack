module.exports = function () {
  if (!this.parallel) throw new Error("loader must run in a worker");
  return `module.exports = ${JSON.stringify(this._module.buildInfo.onDone())}`;
};
