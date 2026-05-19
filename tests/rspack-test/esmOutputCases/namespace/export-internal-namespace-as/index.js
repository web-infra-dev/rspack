import * as dep1 from "./dep1.js";
import * as dep2 from "./dep2.js";
import { dep1 as reexportedDep1, dep2 as reexportedDep2 } from "./reexport.js";

it("should export and resolve internal namespaces as names", () => {
  expect(reexportedDep1.foo).toBe(dep1.foo);
  expect(reexportedDep1.bar).toBe(dep1.bar);
  expect(reexportedDep2.foo).toBe(dep2.foo);
  expect(reexportedDep2.bar).toBe(dep2.bar);
});
