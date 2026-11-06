module.exports = function (source) {
  const suffix = (this.resourceQuery || "") + (this.resourceFragment || "");
  return suffix ? `export default ${JSON.stringify(suffix)};` : source;
};
