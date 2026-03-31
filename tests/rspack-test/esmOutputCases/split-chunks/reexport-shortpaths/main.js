import dep from "./dep1.js";

export default dep();

it("should avoid duplicating short reexport paths across entries", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  const other = await import(/* webpackIgnore: true */ "./other.mjs");
  const third = await import(/* webpackIgnore: true */ "./third.mjs");

  expect(mod.default).toBe("dep2");
  expect(other.default).toBe("other");
  expect(third.default).toBe("third");
  expect(globalThis.__reexport_shortpaths_count__).toBe(1);
});
