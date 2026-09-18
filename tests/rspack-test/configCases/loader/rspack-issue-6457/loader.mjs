export default function (code) {
  return `${code};exports.foo = "${this.data.foo}"`;
};

export const pitch = function (_a, _b, data) {
  data.foo = "bar";
};
