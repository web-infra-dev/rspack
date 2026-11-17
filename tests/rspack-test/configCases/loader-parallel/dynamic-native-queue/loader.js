module.exports = function () {
  if (!this.parallel) throw new Error("Expected the native worker queue");
  return "module.exports = true;";
};
