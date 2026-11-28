import value from "value";

it("should preserve base externals with an undefined layer selector", () => {
  expect(value).toBe(42);
});
