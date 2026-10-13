it("should be bundled in the matching layer chunk", () => {
  expect(__filename).toContain("foo.js");
});
