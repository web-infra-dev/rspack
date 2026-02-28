it("should have correct runtime", () => {
  const { entrypoints } = __STATS__;
  expect(entrypoints["e1"].chunks).toContain("runtime")
  expect(entrypoints["e1"].chunks).toContain("e1")
});
