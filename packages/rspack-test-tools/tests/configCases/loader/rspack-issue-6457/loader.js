module.exports = function (code) {
  return `${code};exports.foo = "${this.data.foo}"`;
};

module.exports.pitch = function (_a, _b, data) {
  data.foo = "bar";
};
