export * as external from "fs";

import * as indirect from "fs";

export { indirect };

it("should re-export the same external namespace through direct and indirect forms", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(typeof mod.external.readFile).toBe("function");
  expect(mod.external.readFile).toBe(mod.indirect.readFile);
});
