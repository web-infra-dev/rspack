import * as dep from "./dep.js";

export { dep };

it("should materialize a namespace object for default-only modules", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(dep.default).toBe("default");
  expect(Object.keys(dep)).toEqual(["default"]);
  expect(mod.dep.default).toBe("default");
});
