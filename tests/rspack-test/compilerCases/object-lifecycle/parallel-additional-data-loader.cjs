module.exports = function parallelAdditionalDataLoader(source) {
  this.callback(null, source, null, { owner: "worker" });
};
