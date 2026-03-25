export * as fsNs from "./reexport-external.js";

it("should export a namespace for an internal module that star-reexports an external", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(typeof mod.fsNs.readFile).toBe("function");
  expect(typeof mod.fsNs.readFileSync).toBe("function");
});
