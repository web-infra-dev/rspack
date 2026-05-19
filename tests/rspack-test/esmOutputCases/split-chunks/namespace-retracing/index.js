import * as otherNs from "./other.js";
import * as fooNs from "./foo.js";

export default [
  otherNs.callBrokenFromOther(),
  otherNs.Other.doSomething(),
  fooNs.broken(),
];

it("should retrace namespace reexports through split chunks", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toEqual(["broken", "other", "broken"]);
});
