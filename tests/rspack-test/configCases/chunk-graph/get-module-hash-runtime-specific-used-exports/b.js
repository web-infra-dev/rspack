import { getB } from "./shared";

it("should load export b", () => {
  expect(getB()).toBe("b");
});
