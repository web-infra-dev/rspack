it("should use the default branch only when the issuer layer has no matching branch", () => {
  expect(require("./consumer")).toEqual([42, 43]);
  expect(require("./app")).toEqual([42, 42]);
});
