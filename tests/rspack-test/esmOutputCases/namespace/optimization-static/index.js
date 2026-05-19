import * as foo from "./foo.js";

export default foo.bar.quux.a();

it("should optimize static namespace lookup chains", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toBe("a");
});
