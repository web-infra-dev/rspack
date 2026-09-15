export * as empty from "./empty";
export * as asyncTarget from "./async-target";
export * as externalTarget from "./external-target";
export * as wrapped from "./wrapped";

it("preserves empty namespaces, top-level await and wrapped live bindings", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");
  expect(Object.keys(mod.empty)).toEqual([]);
  expect(Object.keys(mod.asyncTarget)).toEqual(["default", "value"]);
  expect(mod.asyncTarget.value).toBe(42);
  expect(mod.asyncTarget.default).toBe("ready");
  expect(mod.externalTarget.value).toBe(1);
  mod.externalTarget.increment();
  expect(mod.externalTarget.value).toBe(2);
  expect(Object.keys(mod.wrapped).sort()).toEqual(["increment", "value"]);
  expect(mod.wrapped.value).toBe(1);
  mod.wrapped.increment();
  expect(mod.wrapped.value).toBe(2);
});
