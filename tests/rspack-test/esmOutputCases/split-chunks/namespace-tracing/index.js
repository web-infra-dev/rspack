import * as fooNs from "./foo.js";
import * as barNs from "./bar.js";

export default [fooNs.foo(), fooNs.broken(), barNs.bar(), barNs.broken()];

it("should trace namespace reexports across split chunks", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toEqual(["foo", "broken", "bar", "broken"]);
});
