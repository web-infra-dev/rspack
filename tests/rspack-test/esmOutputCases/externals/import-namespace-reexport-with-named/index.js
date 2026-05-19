export { sep } from "path";

import * as fsNs from "fs";
import { sep as pathSep } from "path";

export { fsNs };

it("should combine a named external re-export with an imported external namespace re-export", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.sep).toBe(pathSep);
  expect(typeof mod.fsNs.readFile).toBe("function");
});
