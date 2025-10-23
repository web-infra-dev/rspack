import * as lib from "./lib";
import * as c from "./enum";

it("should have correct value", () => {
  lib;
  expect(c).toHaveProperty("A.A", 0);
  expect(c).toHaveProperty("B.B", 0);
});
