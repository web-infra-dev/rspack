import * as mod from "./lib";

it("should have correct exports", () => {
  expect(mod.foo).toBe(1);
  expect(mod.bar).toBe(2);
});
