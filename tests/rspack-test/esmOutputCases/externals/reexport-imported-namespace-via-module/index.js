export { fsNs } from "./dep.js";

it("should re-export an imported external namespace through an intermediate module", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(typeof mod.fsNs.readFile).toBe("function");
  expect(typeof mod.fsNs.readFileSync).toBe("function");
});
