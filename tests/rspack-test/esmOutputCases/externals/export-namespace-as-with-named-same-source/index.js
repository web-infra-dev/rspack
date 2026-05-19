export * as fsNs from "fs";
export { readFile, readFileSync } from "fs";

it("should combine export-star-as and named exports for the same external source", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.fsNs.readFile).toBe(mod.readFile);
  expect(mod.fsNs.readFileSync).toBe(mod.readFileSync);
});
