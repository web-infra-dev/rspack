module.exports = function () {
  const content = "\ufeffhello";
  return this.getOptions().kind === "string" ? content : Buffer.from(content);
};
