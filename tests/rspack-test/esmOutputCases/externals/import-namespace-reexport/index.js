import * as ext from "fs";

export { ext };

it("should re-export an imported external namespace", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(typeof mod.ext.readFile).toBe("function");
  expect(typeof mod.ext.readFileSync).toBe("function");
});
