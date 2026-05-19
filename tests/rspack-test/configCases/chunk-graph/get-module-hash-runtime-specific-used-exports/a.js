import { getA } from "./shared";

it("should load export a", () => {
  expect(getA()).toBe("a");
});
