module.exports = function (source) {
  const firstType = this.resourceQuery === "?string-first";
  const isString = Number(source.trim()) < 2 ? firstType : !firstType;
  const content = "\uFEFFhello";
  return isString ? content : Buffer.from(content);
};
