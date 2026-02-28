
import * as c from "./enum";

it("should have correct value", async () => {
  await import(/*webpackChunkName: "lib"*/ "./lib");
  expect(c).toHaveProperty("A.A", 0);
  expect(c).toHaveProperty("B.B", 0);
});
