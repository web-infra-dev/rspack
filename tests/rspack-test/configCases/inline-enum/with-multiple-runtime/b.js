import * as c from "./enum"

it("should have correct value", async () => {
  await import(/*webpackChunkName: "lib"*/ "./lib");
  expect(c.A.A).toBe(0);
});
