export * as external1 from "fs";

import * as imported1 from "fs";

export * as external2 from "path";

import { resolve as imported2 } from "path";

it("should re-export used external namespaces without duplicating semantics", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.external1.readFile).toBe(imported1.readFile);
  expect(mod.external2.resolve).toBe(imported2);
});
