import value from "@foo/index";

it("alias should have right order", () => {
  expect(value).toBe("b");
});