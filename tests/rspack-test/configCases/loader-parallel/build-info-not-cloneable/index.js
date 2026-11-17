it("should access function-valued build info through the main object bridge", () => {
  expect(require("./lib.js")).toBe("not cloneable");
});
