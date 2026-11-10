it("should merge externals for the issuer layer and preserve the base for other layers", () => {
  expect(require("./consumer")).toEqual([42, 42, 42, 42]);
  expect(require("./app")).toEqual([42, 43, 7, 42]);
  expect(require("./other")).toEqual([42, 42, 42, 42]);
});
