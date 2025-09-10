module.exports = function (code) {
  return `${code};exports.bar = "${this.data.bar}"`;
};

module.exports.pitch = function (_a, _b, data) {
  data.bar = "baz";
};
