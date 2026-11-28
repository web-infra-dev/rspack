it("should preserve external value replacement and extension semantics", () => {
  expect(require("./app")).toEqual([41, 43, 45, 46, 47, 42]);
});
