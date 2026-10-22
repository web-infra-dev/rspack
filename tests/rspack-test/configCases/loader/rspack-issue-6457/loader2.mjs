export default function (code) {
  return `${code};exports.bar = "${this.data.bar}"`;
};

export const pitch = function (_a, _b, data) {
  data.bar = "baz";
};
