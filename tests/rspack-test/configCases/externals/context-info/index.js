import e1 from "external-pkg";
import e2 from "./other-layer";

it("should have the correct value", () => {
  expect(e1).toBe(1);
  expect(e2).toBe(2);
});
