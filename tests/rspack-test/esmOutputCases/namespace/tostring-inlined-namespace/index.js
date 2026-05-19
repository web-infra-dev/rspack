import * as foo from "./foo.js";

export default {
  tag: foo[Symbol.toStringTag],
  text: Object.prototype.toString.call(foo),
  bar: foo.bar,
};

it("should preserve module toStringTag on local namespace objects", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toEqual({
    tag: "Module",
    text: "[object Module]",
    bar: 42,
  });
});
