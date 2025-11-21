it("should have correct runtime", () => {
  const { entrypoints } = __STATS__;
  expect(entrypoints["e2"].chunks).not.toContain("runtime")
  expect(entrypoints["e2"].chunks).toContain("e2");
});
