export * as fsNs from "./reexport-external.js";

it("should merge local exports with external star reexports inside exported namespace", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.fsNs.readFile).toBe(42);
  expect(typeof mod.fsNs.readFileSync).toBe("function");
  expect(mod.fsNs.marker).toBe("marker");
});
